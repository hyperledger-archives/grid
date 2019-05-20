// Copyright 2018 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::ApplyError;
        use sabre_sdk::TransactionContext;
        use sabre_sdk::TransactionHandler;
        use sabre_sdk::TpProcessRequest;
        use sabre_sdk::{WasmPtr, execute_entrypoint};
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
        use sawtooth_sdk::processor::handler::TransactionContext;
        use sawtooth_sdk::processor::handler::TransactionHandler;
        use sawtooth_sdk::messages::processor::TpProcessRequest;
    }
}

use grid_sdk::protocol::errors::BuilderError;
use grid_sdk::protocol::schema::state::{PropertyDefinition, PropertyValue};
use grid_sdk::protocol::track_and_trace::payload::{
    Action, AnswerProposalAction, CreateProposalAction, CreateRecordAction, FinalizeRecordAction,
    Response, RevokeReporterAction, TrackAndTracePayload, UpdatePropertiesAction,
};
use grid_sdk::protocol::track_and_trace::state::{
    AssociatedAgentBuilder, PropertyBuilder, PropertyPageBuilder, ProposalBuilder,
    ProposalListBuilder, RecordBuilder, ReportedValueBuilder, ReporterBuilder, Role, Status,
};

use grid_sdk::protos::FromBytes;

use crate::addressing::*;
use crate::payload::validate_payload;
use crate::state::TrackAndTraceState;

const PROPERTY_PAGE_MAX_LENGTH: usize = 256;

pub struct TrackAndTraceTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

impl TrackAndTraceTransactionHandler {
    #[allow(clippy::new_without_default)]
    pub fn new() -> TrackAndTraceTransactionHandler {
        TrackAndTraceTransactionHandler {
            family_name: "grid_track_and_trace".to_string(),
            family_versions: vec!["1.0".to_string()],
            namespaces: vec![
                get_track_and_trace_prefix(),
                get_pike_prefix(),
                get_grid_prefix(),
            ],
        }
    }

    fn _create_record(
        &self,
        payload: &CreateRecordAction,
        state: &mut TrackAndTraceState,
        signer: &str,
        timestamp: u64,
    ) -> Result<(), ApplyError> {
        match state.get_agent(signer)? {
            Some(_) => (),
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Agent is not registered: {}",
                    signer
                )));
            }
        }
        let record_id = payload.record_id();
        if state.get_record(record_id)?.is_some() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Record already exists: {}",
                record_id
            )));
        }

        let schema_name = payload.schema();
        let schema = match state.get_schema(schema_name)? {
            Some(schema) => schema,
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Schema does not exist {}",
                    schema_name
                )));
            }
        };

        let mut type_schemata: HashMap<&str, PropertyDefinition> = HashMap::new();
        let mut required_properties: HashMap<&str, PropertyDefinition> = HashMap::new();
        let mut provided_properties: HashMap<&str, PropertyValue> = HashMap::new();
        for property in schema.properties() {
            type_schemata.insert(property.name(), property.clone());
            if *property.required() {
                required_properties.insert(property.name(), property.clone());
            }
        }

        for property in payload.properties() {
            provided_properties.insert(property.name(), property.clone());
        }

        for name in required_properties.keys() {
            if !provided_properties.contains_key(name) {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Required property {} not provided",
                    name
                )));
            }
        }

        for (provided_name, provided_properties) in provided_properties.clone() {
            let required_type = match type_schemata.get(provided_name) {
                Some(required_type) => required_type.data_type(),
                None => {
                    return Err(ApplyError::InvalidTransaction(format!(
                        "Provided property {} is not in schema",
                        provided_name
                    )));
                }
            };
            let provided_type = provided_properties.data_type();
            if provided_type != required_type {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Value provided for {} is the wrong type",
                    provided_name
                )));
            };
        }

        let owner = AssociatedAgentBuilder::new()
            .with_agent_id(signer.to_string())
            .with_timestamp(timestamp)
            .build()
            .map_err(|err| map_builder_error_to_apply_error(err, "AssociatedAgent"))?;

        let new_record = RecordBuilder::new()
            .with_record_id(record_id.to_string())
            .with_schema(schema_name.to_string())
            .with_field_final(false)
            .with_owners(vec![owner.clone()])
            .with_custodians(vec![owner.clone()])
            .build()
            .map_err(|err| map_builder_error_to_apply_error(err, "Record"))?;

        state.set_record(record_id, new_record)?;

        let reporter = ReporterBuilder::new()
            .with_public_key(signer.to_string())
            .with_authorized(true)
            .with_index(0)
            .build()
            .map_err(|err| map_builder_error_to_apply_error(err, "Reporter"))?;

        for (property_name, property) in type_schemata {
            let new_property = PropertyBuilder::new()
                .with_name(property_name.to_string())
                .with_record_id(record_id.to_string())
                .with_property_definition(property.clone())
                .with_reporters(vec![reporter.clone()])
                .with_current_page(1)
                .with_wrapped(false)
                .build()
                .map_err(|err| map_builder_error_to_apply_error(err, "Property"))?;

            state.set_property(record_id, property_name, new_property.clone())?;

            let mut new_property_page = PropertyPageBuilder::new()
                .with_name(property_name.to_string())
                .with_record_id(record_id.to_string());

            if provided_properties.contains_key(property_name) {
                let provided_property = provided_properties[property_name].clone();
                let reported_value = ReportedValueBuilder::new()
                    .with_reporter_index(0)
                    .with_timestamp(timestamp)
                    .with_value(provided_property)
                    .build()
                    .map_err(|err| map_builder_error_to_apply_error(err, "ReportedValue"))?;

                new_property_page = new_property_page.with_reported_values(vec![reported_value]);
            }

            state.set_property_page(
                record_id,
                property_name,
                1,
                new_property_page
                    .build()
                    .map_err(|err| map_builder_error_to_apply_error(err, "PropertyPage"))?,
            )?;
        }

        Ok(())
    }

    fn _finalize_record(
        &self,
        payload: FinalizeRecordAction,
        mut state: SupplyChainState,
        signer: &str,
    ) -> Result<(), ApplyError> {
        let record_id = payload.get_record_id();
        let final_record = match state.get_record(record_id) {
            Ok(Some(final_record)) => final_record,
            Ok(None) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Record does not exist: {}",
                    record_id
                )));
            }
            Err(err) => return Err(err),
        };
        let owner = match final_record.owners.last() {
            Some(x) => x,
            None => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Owner was not found",
                )));
            }
        };
        let custodian = match final_record.custodians.last() {
            Some(x) => x,
            None => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Custodian was not found",
                )));
            }
        };

        if owner.agent_id != signer || custodian.agent_id != signer {
            return Err(ApplyError::InvalidTransaction(
                "Must be owner and custodian to finalize record".to_string(),
            ));
        }
        if final_record.get_field_final() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Record is already final: {}",
                record_id
            )));
        }

        let mut record_clone = final_record.clone();
        record_clone.set_field_final(true);
        state.set_record(record_id, record_clone)?;

        Ok(())
    }

    fn _update_properties(
        &self,
        payload: UpdatePropertiesAction,
        mut state: SupplyChainState,
        signer: &str,
        timestamp: u64,
    ) -> Result<(), ApplyError> {
        let record_id = payload.get_record_id();
        let update_record = match state.get_record(record_id) {
            Ok(Some(update_record)) => update_record,
            Ok(None) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Record does not exist: {}",
                    record_id
                )));
            }
            Err(err) => return Err(err),
        };

        if update_record.get_field_final() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Record is final: {}",
                record_id
            )));
        }

        let updates = payload.get_properties();

        for update in updates {
            let name = update.get_name();
            let data_type = update.get_data_type();

            let mut prop = match state.get_property(record_id, name) {
                Ok(Some(prop)) => prop,
                Ok(None) => {
                    return Err(ApplyError::InvalidTransaction(format!(
                        "Record does not have provided poperty: {}",
                        name
                    )));
                }
                Err(err) => return Err(err),
            };

            let mut allowed = false;
            let mut reporter_index = 0;
            for reporter in prop.get_reporters() {
                if reporter.get_public_key() == signer && reporter.get_authorized() {
                    allowed = true;
                    reporter_index = reporter.get_index();
                    break;
                }
            }
            if !allowed {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Reporter is not authorized: {}",
                    signer
                )));
            }

            if prop.fixed {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Property is fixed and cannot be updated: {}",
                    prop.name
                )));
            }

            if data_type != prop.data_type {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Update has wrong type: {:?} != {:?}",
                    data_type, prop.data_type
                )));
            }

            let page_number = prop.get_current_page();
            let mut page = match state.get_property_page(record_id, name, page_number) {
                Ok(Some(page)) => page,
                Ok(None) => {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Property page does not exist",
                    )));
                }
                Err(err) => return Err(err),
            };

            let reported_value =
                match self._make_new_reported_value(reporter_index, timestamp, update, &prop) {
                    Ok(reported_value) => reported_value,
                    Err(err) => return Err(err),
                };
            page.reported_values.push(reported_value);
            page.reported_values
                .sort_by_key(|rv| (rv.clone().timestamp, rv.clone().reporter_index));
            state.set_property_page(record_id, name, page_number, page.clone())?;
            if page.reported_values.len() >= PROPERTY_PAGE_MAX_LENGTH {
                let new_page_number = if page_number < PROPERTY_PAGE_MAX_LENGTH as u32 {
                    1
                } else {
                    page_number + 1
                };

                let new_page = match state.get_property_page(record_id, name, new_page_number) {
                    Ok(Some(mut new_page)) => {
                        new_page.set_reported_values(RepeatedField::from_vec(Vec::new()));
                        new_page
                    }
                    Ok(None) => {
                        let mut new_page = PropertyPage::new();
                        new_page.set_name(name.to_string());
                        new_page.set_record_id(record_id.to_string());
                        new_page
                    }
                    Err(err) => return Err(err),
                };
                state.set_property_page(record_id, name, new_page_number, new_page)?;

                prop.set_current_page(new_page_number);
                if new_page_number == 1 && !prop.get_wrapped() {
                    prop.set_wrapped(true);
                }
                state.set_property(record_id, name, prop)?;
            }
        }

        Ok(())
    }

    fn _create_proposal(
        &self,
        payload: CreateProposalAction,
        mut state: SupplyChainState,
        signer: &str,
        timestamp: u64,
    ) -> Result<(), ApplyError> {
        let record_id = payload.record_id;
        let receiving_agent = payload.receiving_agent;
        let role = payload.role;
        let properties = payload.properties;

        match state.get_agent(signer) {
            Ok(Some(agent)) => agent,
            Ok(None) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Issuing agent does not exist: {}",
                    signer
                )));
            }
            Err(err) => return Err(err),
        };

        match state.get_agent(&receiving_agent) {
            Ok(Some(agent)) => agent,
            Ok(None) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Receiving agent does not exist: {}",
                    receiving_agent
                )));
            }
            Err(err) => return Err(err),
        };

        let mut proposals = match state.get_proposal_container(&record_id, &receiving_agent) {
            Ok(Some(proposals)) => proposals,
            Ok(None) => ProposalContainer::new(),
            Err(err) => return Err(err),
        };

        let mut open_proposals = Vec::<Proposal>::new();
        for prop in proposals.get_entries() {
            if prop.status == Proposal_Status::OPEN {
                open_proposals.push(prop.clone());
            }
        }

        for prop in open_proposals {
            if prop.get_receiving_agent() == receiving_agent
                && prop.get_role() == role
                && prop.get_record_id() == record_id
            {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Proposal already exists",
                )));
            }
        }

        let proposal_record = match state.get_record(&record_id) {
            Ok(Some(record)) => record,
            Ok(None) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Record does not exist: {}",
                    record_id
                )));
            }
            Err(err) => return Err(err),
        };

        if proposal_record.get_field_final() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Record is final: {}",
                record_id
            )));
        }

        if role == Proposal_Role::OWNER || role == Proposal_Role::REPORTER {
            let owner = match proposal_record.owners.last() {
                Some(owner) => owner,
                None => {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Owner not found",
                    )));
                }
            };
            if owner.get_agent_id() != signer {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Only the owner can create a proposal to change ownership",
                )));
            }
        }

        if role == Proposal_Role::CUSTODIAN {
            let custodian = match proposal_record.custodians.last() {
                Some(custodian) => custodian,
                None => {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Custodian not found",
                    )));
                }
            };

            if custodian.get_agent_id() != signer {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Only the custodian can create a proposal to change custodianship",
                )));
            }
        }

        let mut new_proposal = Proposal::new();
        new_proposal.set_record_id(record_id.to_string());
        new_proposal.set_timestamp(timestamp);
        new_proposal.set_issuing_agent(signer.to_string());
        new_proposal.set_receiving_agent(receiving_agent.to_string());
        new_proposal.set_role(role);
        new_proposal.set_properties(properties);
        new_proposal.set_status(Proposal_Status::OPEN);

        proposals.entries.push(new_proposal);
        proposals.entries.sort_by_key(|p| {
            (
                p.clone().record_id,
                p.clone().receiving_agent,
                p.clone().timestamp,
            )
        });
        state.set_proposal_container(&record_id, &receiving_agent, proposals)?;

        Ok(())
    }

    fn _answer_proposal(
        &self,
        payload: AnswerProposalAction,
        mut state: SupplyChainState,
        signer: &str,
        timestamp: u64,
    ) -> Result<(), ApplyError> {
        let record_id = payload.get_record_id();
        let receiving_agent = payload.get_receiving_agent();
        let role = payload.get_role();
        let response = payload.get_response();

        let mut proposals = match state.get_proposal_container(record_id, receiving_agent) {
            Ok(Some(proposals)) => proposals,
            Ok(None) => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Proposal does not exist",
                )));
            }
            Err(err) => return Err(err),
        };

        let mut exists = false;
        let mut current_proposal = match proposals.clone().entries.last() {
            Some(current_proposal) => current_proposal.clone(),
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "No open proposals found for record {} for {}",
                    record_id, receiving_agent
                )));
            }
        };

        let mut proposal_index = 0;

        for (i, prop) in proposals.get_entries().iter().enumerate() {
            if prop.get_receiving_agent() == receiving_agent
                && prop.get_role() == role
                && prop.get_record_id() == record_id
                && prop.status == Proposal_Status::OPEN
            {
                current_proposal = prop.clone();
                exists = true;
                proposal_index = i;
                break;
            }
        }

        if !exists {
            return Err(ApplyError::InvalidTransaction(format!(
                "No open proposals found for record {} for {}",
                record_id, receiving_agent
            )));
        }

        match response {
            AnswerProposalAction_Response::CANCEL => {
                if current_proposal.get_issuing_agent() != signer {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Only the issuing agent can cancel a proposal",
                    )));
                }
                current_proposal.status = Proposal_Status::CANCELED;
            }
            AnswerProposalAction_Response::REJECT => {
                if current_proposal.get_receiving_agent() != signer {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Only the receiving agent can reject a proposal",
                    )));
                }
                current_proposal.status = Proposal_Status::REJECTED;
            }
            AnswerProposalAction_Response::ACCEPT => {
                if current_proposal.get_receiving_agent() != signer {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Only the receiving agent can Accept a proposal",
                    )));
                };

                let mut proposal_record = match state.get_record(record_id) {
                    Ok(Some(record)) => record,
                    Ok(None) => {
                        return Err(ApplyError::InvalidTransaction(format!(
                            "Record in proposal does not exist: {}",
                            record_id
                        )));
                    }
                    Err(err) => return Err(err),
                };

                let owner = match proposal_record.clone().owners.last() {
                    Some(owner) => owner.clone(),
                    None => {
                        return Err(ApplyError::InvalidTransaction(String::from(
                            "Owner not found",
                        )));
                    }
                };

                let custodian = match proposal_record.clone().custodians.last() {
                    Some(custodian) => custodian.clone(),
                    None => {
                        return Err(ApplyError::InvalidTransaction(String::from(
                            "Custodian not found",
                        )));
                    }
                };

                match role {
                    Proposal_Role::OWNER => {
                        if owner.get_agent_id() != current_proposal.get_issuing_agent() {
                            current_proposal.status = Proposal_Status::CANCELED;
                            #[cfg(not(target_arch = "wasm32"))]
                            info!("Record owner does not match the issuing agent of the proposal");
                            // remove old proposal and replace with new one
                            proposals.entries.remove(proposal_index);
                            proposals.entries.push(current_proposal);
                            proposals.entries.sort_by_key(|p| {
                                (
                                    p.clone().record_id,
                                    p.clone().receiving_agent,
                                    p.clone().timestamp,
                                )
                            });
                            state.set_proposal_container(
                                &record_id,
                                &receiving_agent,
                                proposals,
                            )?;
                            return Ok(());
                        }

                        let mut new_owner = Record_AssociatedAgent::new();
                        new_owner.set_agent_id(receiving_agent.to_string());
                        new_owner.set_timestamp(timestamp);
                        proposal_record.owners.push(new_owner);
                        state.set_record(record_id, proposal_record.clone())?;

                        let record_type =
                            match state.get_record_type(proposal_record.get_record_type()) {
                                Ok(Some(record_type)) => record_type,
                                Ok(None) => {
                                    return Err(ApplyError::InvalidTransaction(format!(
                                        "RecordType does not exist: {}",
                                        proposal_record.get_record_type()
                                    )));
                                }
                                Err(err) => return Err(err),
                            };

                        for prop_schema in record_type.get_properties() {
                            let mut prop =
                                match state.get_property(record_id, prop_schema.get_name()) {
                                    Ok(Some(prop)) => prop,
                                    Ok(None) => {
                                        return Err(ApplyError::InvalidTransaction(String::from(
                                            "Property does not exist",
                                        )));
                                    }
                                    Err(err) => return Err(err),
                                };

                            let mut authorized = false;
                            let mut new_reporters: Vec<Property_Reporter> = Vec::new();
                            let temp_prob = prop.clone();
                            let reporters = temp_prob.get_reporters();
                            for reporter in reporters {
                                if reporter.get_public_key() == owner.get_agent_id() {
                                    let mut new_reporter = reporter.clone();
                                    new_reporter.set_authorized(false);
                                    new_reporters.push(new_reporter);
                                } else if reporter.get_public_key() == receiving_agent {
                                    let mut new_reporter = reporter.clone();
                                    new_reporter.set_authorized(true);
                                    authorized = true;
                                    new_reporters.push(new_reporter);
                                } else {
                                    new_reporters.push(reporter.clone());
                                }
                            }

                            if !authorized {
                                let mut reporter = Property_Reporter::new();
                                reporter.set_public_key(receiving_agent.to_string());
                                reporter.set_authorized(true);
                                reporter.set_index(prop.reporters.len() as u32);
                                new_reporters.push(reporter);
                            }

                            prop.set_reporters(RepeatedField::from_vec(new_reporters));
                            state.set_property(record_id, prop.get_name(), prop.clone())?;
                        }
                        current_proposal.status = Proposal_Status::ACCEPTED;
                    }
                    Proposal_Role::CUSTODIAN => {
                        if custodian.get_agent_id() != current_proposal.get_issuing_agent() {
                            current_proposal.status = Proposal_Status::CANCELED;
                            #[cfg(not(target_arch = "wasm32"))]
                            info!(
                                "Record custodian does not match the issuing agent of the proposal"
                            );
                            // remove old proposal and replace with new one
                            proposals.entries.remove(proposal_index);
                            proposals.entries.push(current_proposal.clone());
                            proposals.entries.sort_by_key(|p| {
                                (
                                    p.clone().record_id,
                                    p.clone().receiving_agent,
                                    p.clone().timestamp,
                                )
                            });
                            state.set_proposal_container(
                                &record_id,
                                &receiving_agent,
                                proposals.clone(),
                            )?;
                        }

                        let mut new_custodian = Record_AssociatedAgent::new();
                        new_custodian.set_agent_id(receiving_agent.to_string());
                        new_custodian.set_timestamp(timestamp);
                        proposal_record.custodians.push(new_custodian.clone());
                        state.set_record(record_id, proposal_record)?;
                        current_proposal.status = Proposal_Status::ACCEPTED;
                    }
                    Proposal_Role::REPORTER => {
                        if owner.get_agent_id() != current_proposal.get_issuing_agent() {
                            current_proposal.status = Proposal_Status::CANCELED;
                            #[cfg(not(target_arch = "wasm32"))]
                            info!("Record owner does not match the issuing agent of the proposal");
                            // remove old proposal and replace with new one
                            proposals.entries.remove(proposal_index);
                            proposals.entries.push(current_proposal);
                            proposals.entries.sort_by_key(|p| {
                                (
                                    p.clone().record_id,
                                    p.clone().receiving_agent,
                                    p.clone().timestamp,
                                )
                            });
                            state.set_proposal_container(
                                &record_id,
                                &receiving_agent,
                                proposals,
                            )?;
                            return Ok(());
                        }

                        let mut reporter = Property_Reporter::new();
                        reporter.set_public_key(receiving_agent.to_string());
                        reporter.set_authorized(true);

                        for prop_name in current_proposal.get_properties() {
                            let mut prop = match state.get_property(record_id, prop_name) {
                                Ok(Some(prop)) => prop,
                                Ok(None) => {
                                    return Err(ApplyError::InvalidTransaction(String::from(
                                        "Property does not exist",
                                    )));
                                }
                                Err(err) => return Err(err),
                            };
                            reporter.set_index(prop.reporters.len() as u32);
                            prop.reporters.push(reporter.clone());
                            state.set_property(record_id, prop_name, prop)?;
                        }
                        current_proposal.status = Proposal_Status::ACCEPTED;
                    }
                }
            }
        }
        // remove old proposal and replace with new one
        proposals.entries.remove(proposal_index);
        proposals.entries.push(current_proposal.clone());
        proposals.entries.sort_by_key(|p| {
            (
                p.clone().record_id,
                p.clone().receiving_agent,
                p.clone().timestamp,
            )
        });
        state.set_proposal_container(&record_id, &receiving_agent, proposals)?;

        Ok(())
    }

    fn _revoke_reporter(
        &self,
        payload: RevokeReporterAction,
        mut state: SupplyChainState,
        signer: &str,
    ) -> Result<(), ApplyError> {
        let record_id = payload.get_record_id();
        let reporter_id = payload.get_reporter_id();
        let properties = payload.get_properties();

        let revoke_record = match state.get_record(record_id) {
            Ok(Some(record)) => record,
            Ok(None) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Record does not exists: {}",
                    record_id
                )));
            }
            Err(err) => return Err(err),
        };

        let owner = match revoke_record.owners.last() {
            Some(x) => x,
            None => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Owner was not found",
                )));
            }
        };

        if owner.get_agent_id() != signer {
            return Err(ApplyError::InvalidTransaction(
                "Must be owner to revoke reporters".to_string(),
            ));
        }

        if revoke_record.get_field_final() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Record is final: {}",
                record_id
            )));
        }

        for prop_name in properties {
            let mut prop = match state.get_property(record_id, prop_name) {
                Ok(Some(prop)) => prop,
                Ok(None) => {
                    return Err(ApplyError::InvalidTransaction(
                        "Property does not exists".to_string(),
                    ));
                }
                Err(err) => return Err(err),
            };

            let mut new_reporters: Vec<Property_Reporter> = Vec::new();
            let mut revoked = false;
            for reporter in prop.get_reporters() {
                if reporter.get_public_key() == reporter_id {
                    if !reporter.get_authorized() {
                        return Err(ApplyError::InvalidTransaction(
                            "Reporter is already unauthorized.".to_string(),
                        ));
                    }
                    let mut unauthorized_reporter = reporter.clone();
                    unauthorized_reporter.set_authorized(false);
                    revoked = true;
                    new_reporters.push(unauthorized_reporter);
                } else {
                    new_reporters.push(reporter.clone());
                }
            }
            if !revoked {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Reporter cannot be revoked: {}",
                    reporter_id
                )));
            }
            prop.set_reporters(RepeatedField::from_vec(new_reporters));

            state.set_property(record_id, prop_name, prop)?;
        }

        Ok(())
    }

    fn _make_new_reported_value(
        &self,
        reporter_index: u32,
        timestamp: u64,
        value: &TrackAndTracePropertyValue,
        property: &Property,
    ) -> Result<PropertyPage_ReportedValue, ApplyError> {
        let mut reported_value = PropertyPage_ReportedValue::new();
        reported_value.set_reporter_index(reporter_index);
        reported_value.set_timestamp(timestamp);

        match value.get_data_type() {
            PropertySchema_DataType::TYPE_UNSET => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "DataType is not set",
                )));
            }
            PropertySchema_DataType::BYTES => {
                reported_value.set_bytes_value(value.get_bytes_value().to_vec())
            }
            PropertySchema_DataType::BOOLEAN => {
                reported_value.set_boolean_value(value.get_boolean_value())
            }
            PropertySchema_DataType::NUMBER => {
                reported_value.set_number_value(value.get_number_value())
            }
            PropertySchema_DataType::STRING => {
                reported_value.set_string_value(value.get_string_value().to_string())
            }
            PropertySchema_DataType::ENUM => {
                let enum_name = value.get_enum_value().to_string();
                let enum_index = match property
                    .enum_options
                    .iter()
                    .position(|name| name == &enum_name)
                {
                    Some(index) => index,
                    None => {
                        return Err(ApplyError::InvalidTransaction(format!(
                            "Provided enum name is not a valid option: {}",
                            enum_name,
                        )));
                    }
                };
                reported_value.set_enum_value(enum_index as u32)
            }
            PropertySchema_DataType::STRUCT => {
                match self
                    ._validate_struct_values(&value.struct_values, &property.struct_properties)
                {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                }

                let struct_values = RepeatedField::from_vec(value.get_struct_values().to_vec());
                reported_value.set_struct_values(struct_values)
            }
            PropertySchema_DataType::LOCATION => {
                reported_value.set_location_value(value.get_location_value().clone())
            }
        };
        Ok(reported_value)
    }

    fn _validate_struct_values(
        &self,
        struct_values: &RepeatedField<TrackAndTracePropertyValue>,
        schema_values: &RepeatedField<PropertySchema>,
    ) -> Result<(), ApplyError> {
        if struct_values.len() != schema_values.len() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Provided struct does not match schema length: {:?} != {:?}",
                struct_values.len(),
                schema_values.len(),
            )));
        }

        for schema in schema_values.iter() {
            let value = match struct_values.iter().find(|val| val.name == schema.name) {
                Some(val) => val,
                None => {
                    return Err(ApplyError::InvalidTransaction(format!(
                        "Provided struct missing required property from schema: {}",
                        schema.name,
                    )));
                }
            };

            if value.data_type != schema.data_type {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Struct property \"{}\" must have data type: {:?}",
                    schema.name, schema.data_type,
                )));
            }

            if schema.data_type == PropertySchema_DataType::STRUCT {
                match self._validate_struct_values(&value.struct_values, &schema.struct_properties)
                {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(())
    }
fn map_builder_error_to_apply_error(err: BuilderError, protocol_name: &str) -> ApplyError {
    ApplyError::InvalidTransaction(format!(
        "Failed to build {}. {}",
        protocol_name,
        err.to_string()
    ))
}

impl TransactionHandler for TrackAndTraceTransactionHandler {
    fn family_name(&self) -> String {
        self.family_name.clone()
    }

    fn family_versions(&self) -> Vec<String> {
        self.family_versions.clone()
    }

    fn namespaces(&self) -> Vec<String> {
        self.namespaces.clone()
    }

    fn apply(
        &self,
        request: &TpProcessRequest,
        context: &mut dyn TransactionContext,
    ) -> Result<(), ApplyError> {
        let payload = TrackAndTracePayload::from_bytes(request.get_payload()).map_err(|err| {
            ApplyError::InvalidTransaction(format!("Cannot build track and trace payload: {}", err))
        })?;

        validate_payload(&payload)?;

        let signer = request.get_header().get_signer_public_key();
        let mut state = TrackAndTraceState::new(context);

        #[cfg(not(target_arch = "wasm32"))]
        info!(
            "payload: {:?} {} {} {}",
            payload.action(),
            payload.timestamp(),
            request.get_header().get_inputs()[0],
            request.get_header().get_outputs()[0]
        );

        match payload.action() {
            Action::CreateRecord(action_payload) => {
                self._create_record(action_payload, &mut state, signer, *payload.timestamp())?
            }
            }
            Action::FinalizeRecord(finalize_payload) => {
                self._finalize_record(finalize_payload, state, signer)?
            }

            Action::UpdateProperties(update_properties_payload) => self._update_properties(
                update_properties_payload,
                state,
                signer,
                payload.get_timestamp(),
            )?,
            Action::CreateProposal(proposal_payload) => {
                self._create_proposal(proposal_payload, state, signer, payload.get_timestamp())?
            }
            Action::AnswerProposal(answer_proposal_payload) => self._answer_proposal(
                answer_proposal_payload,
                state,
                signer,
                payload.get_timestamp(),
            )?,
            Action::RevokeReporter(revoke_reporter_payload) => {
                self._revoke_reporter(revoke_reporter_payload, state, signer)?
            }
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
// Sabre apply must return a bool
fn apply(request: &TpProcessRequest, context: &mut TransactionContext) -> Result<bool, ApplyError> {
    let handler = TrackAndTraceTransactionHandler::new();
    match handler.apply(request, context) {
        Ok(_) => Ok(true),
        Err(err) => Err(err),
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub unsafe fn entrypoint(payload: WasmPtr, signer: WasmPtr, signature: WasmPtr) -> i32 {
    execute_entrypoint(payload, signer, signature, apply)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::collections::HashMap;

    use grid_sdk::protocol::pike::state::{AgentBuilder, AgentListBuilder};
    use grid_sdk::protocol::schema::state::{
        DataType, PropertyDefinitionBuilder, PropertyValueBuilder, SchemaBuilder, SchemaListBuilder,
    };
    use grid_sdk::protocol::track_and_trace::payload::{
        AnswerProposalActionBuilder, CreateProposalActionBuilder, CreateRecordActionBuilder,
        FinalizeRecordActionBuilder, RevokeReporterActionBuilder, UpdatePropertiesAction,
        UpdatePropertiesActionBuilder,
    };
    use grid_sdk::protocol::track_and_trace::state::{
        Property, PropertyListBuilder, PropertyPage, PropertyPageListBuilder, Proposal, Record,
        RecordListBuilder, Role, Status,
    };
    use grid_sdk::protos::IntoBytes;
    use sawtooth_sdk::processor::handler::{ContextError, TransactionContext};

    const TIMESTAMP: u64 = 1;
    const RECORD_ID: &str = "test_record_action";
    const PUBLIC_KEY: &str = "agent_public_key";
    const OPTIONAL_PROPERTY_NAME: &str = "test_optional";
    const REQUIRED_PROPERTY_NAME: &str = "test_required";
    const SCHEMA_NAME: &str = "test_schema";

    #[derive(Default, Debug)]
    /// A MockTransactionContext that can be used to test TrackAndTraceState
    struct MockTransactionContext {
        state: RefCell<HashMap<String, Vec<u8>>>,
    }

    impl TransactionContext for MockTransactionContext {
        fn get_state_entries(
            &self,
            addresses: &[String],
        ) -> Result<Vec<(String, Vec<u8>)>, ContextError> {
            let mut results = Vec::new();
            for addr in addresses {
                let data = match self.state.borrow().get(addr) {
                    Some(data) => data.clone(),
                    None => Vec::new(),
                };
                results.push((addr.to_string(), data));
            }
            Ok(results)
        }

        fn set_state_entries(&self, entries: Vec<(String, Vec<u8>)>) -> Result<(), ContextError> {
            for (addr, data) in entries {
                self.state.borrow_mut().insert(addr, data);
            }
            Ok(())
        }

        /// this is not needed for these tests
        fn delete_state_entries(&self, _addresses: &[String]) -> Result<Vec<String>, ContextError> {
            unimplemented!()
        }

        /// this is not needed for these tests
        fn add_receipt_data(&self, _data: &[u8]) -> Result<(), ContextError> {
            unimplemented!()
        }

        /// this is not needed for these tests
        fn add_event(
            &self,
            _event_type: String,
            _attributes: Vec<(String, String)>,
            _data: &[u8],
        ) -> Result<(), ContextError> {
            unimplemented!()
        }
    }

    impl MockTransactionContext {
        fn add_agent(&self, public_key: &str) {
            let builder = AgentBuilder::new();
            let agent = builder
                .with_org_id("test_org".to_string())
                .with_public_key(public_key.to_string())
                .with_active(true)
                .with_roles(vec![])
                .build()
                .unwrap();

            let builder = AgentListBuilder::new();
            let agent_list = builder.with_agents(vec![agent.clone()]).build().unwrap();
            let agent_bytes = agent_list.into_bytes().unwrap();
            let agent_address = make_agent_address(public_key);
            self.set_state_entry(agent_address, agent_bytes).unwrap();
        }

        fn add_schema(&self) {
            let builder = SchemaBuilder::new();
            let schema = builder
                .with_name(SCHEMA_NAME.to_string())
                .with_description("Test Schema".to_string())
                .with_owner("test_org".to_string())
                .with_properties(vec![
                    optional_property_definition(),
                    required_property_definition(),
                ])
                .build()
                .unwrap();

            let builder = SchemaListBuilder::new();
            let schema_list = builder.with_schemas(vec![schema]).build().unwrap();
            let schema_bytes = schema_list.into_bytes().unwrap();
            let schema_address = make_schema_address(SCHEMA_NAME);
            self.set_state_entry(schema_address, schema_bytes).unwrap();
        }

        fn add_record(&self) {
            let record_list = RecordListBuilder::new()
                .with_records(vec![make_record()])
                .build()
                .unwrap();
            let record_bytes = record_list.into_bytes().unwrap();
            let record_address = make_record_address(RECORD_ID);
            self.set_state_entry(record_address, record_bytes).unwrap();
        }

        fn add_property(&self, property_name: &str, property_definition: PropertyDefinition) {
            let property_list = PropertyListBuilder::new()
                .with_properties(vec![make_property(property_name, property_definition)])
                .build()
                .unwrap();

            let property_list_bytes = property_list.into_bytes().unwrap();
            let property_list_address = make_property_address(RECORD_ID, property_name, 0);
            self.set_state_entry(property_list_address, property_list_bytes)
                .unwrap();
        }

        fn add_property_page(&self, property_name: &str, property_value: PropertyValue) {
            let property_page_list = PropertyPageListBuilder::new()
                .with_property_pages(vec![make_property_page(property_name, property_value)])
                .build()
                .expect("Failed to build property page list");

            let property_page_list_bytes = property_page_list
                .into_bytes()
                .expect("Failed to write page list to bytes");
            let address = make_property_address(RECORD_ID, property_name, 1);
            self.set_state_entry(address, property_page_list_bytes)
                .expect("Failed to set state");
        }

        fn add_finalized_record(&self) {
            let associated_agent = AssociatedAgentBuilder::new()
                .with_agent_id(PUBLIC_KEY.to_string())
                .with_timestamp(TIMESTAMP)
                .build()
                .expect("Failed to build associated agent");

            let record = RecordBuilder::new()
                .with_record_id(RECORD_ID.to_string())
                .with_schema(SCHEMA_NAME.to_string())
                .with_owners(vec![associated_agent.clone()])
                .with_custodians(vec![associated_agent.clone()])
                .with_field_final(true)
                .build()
                .expect("Failed to build new_record");

            let record_list = RecordListBuilder::new()
                .with_records(vec![record])
                .build()
                .unwrap();
            let record_bytes = record_list.into_bytes().unwrap();
            let record_address = make_record_address(RECORD_ID);
            self.set_state_entry(record_address, record_bytes).unwrap();
        }

        fn add_proposal(
            &self,
            issuing_agent: &str,
            receiving_agent_key: &str,
            role: Role,
            status: Status,
        ) {
            let proposal_list = ProposalListBuilder::new()
                .with_proposals(vec![make_proposal(
                    issuing_agent,
                    receiving_agent_key,
                    role,
                    status,
                )])
                .build()
                .unwrap();
            let proposal_list_bytes = proposal_list.into_bytes().unwrap();
            let proposal_list_address = make_proposal_address(RECORD_ID, receiving_agent_key);
            self.set_state_entry(proposal_list_address, proposal_list_bytes)
                .unwrap();
        }

        fn add_property_with_reporter(
            &self,
            property_name: &str,
            reporter_key: &str,
            authorized: bool,
            property_definition: PropertyDefinition,
        ) {
            let property = make_property_with_reporter(
                property_name,
                reporter_key,
                authorized,
                property_definition,
            );
            let property_list = PropertyListBuilder::new()
                .with_properties(vec![property])
                .build()
                .unwrap();

            let property_list_bytes = property_list.into_bytes().unwrap();
            let property_list_address = make_property_address(RECORD_ID, property_name, 0);
            self.set_state_entry(property_list_address, property_list_bytes)
                .unwrap();
        }
    }

    #[test]
    /// Test that if the CreateRecordAction is valid an OK is returned and that a new Record,
    /// Property and PropertyPage are added to state
    fn test_create_record_handler_valid() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        transaction_context.add_agent(PUBLIC_KEY);
        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();
        let create_record_action = create_record_action_with_properties(vec![
            optional_property_value(),
            required_property_value(),
        ]);

        assert!(transaction_handler
            ._create_record(&create_record_action, &mut state, PUBLIC_KEY, TIMESTAMP)
            .is_ok());

        let record = state
            .get_record(RECORD_ID)
            .expect("Failed to fetch record")
            .expect("No record found");

        assert_eq!(record, make_record());

        let optional_property = state
            .get_property(RECORD_ID, OPTIONAL_PROPERTY_NAME)
            .expect("Failed to fetch optional property")
            .expect("Optional property not found");

        assert_eq!(
            optional_property,
            make_property(OPTIONAL_PROPERTY_NAME, optional_property_definition())
        );

        let required_property = state
            .get_property(RECORD_ID, REQUIRED_PROPERTY_NAME)
            .expect("Failed to fetch required property")
            .expect("Required property not found");

        assert_eq!(
            required_property,
            make_property(REQUIRED_PROPERTY_NAME, required_property_definition())
        );

        let property_page_optional = state
            .get_property_page(RECORD_ID, OPTIONAL_PROPERTY_NAME, 1)
            .expect("Failed to fetch property page for optional property")
            .expect("Property page for optional property not found");

        assert_eq!(
            property_page_optional,
            make_property_page(OPTIONAL_PROPERTY_NAME, optional_property_value())
        );

        let property_page_required = state
            .get_property_page(RECORD_ID, REQUIRED_PROPERTY_NAME, 1)
            .expect("Failed to fetch property page for required property")
            .expect("Property page for required property not found");

        assert_eq!(
            property_page_required,
            make_property_page(REQUIRED_PROPERTY_NAME, required_property_value())
        );
    }

    #[test]
    /// Test that if the CreateRecordAction is invalid if the signer is not an Agent.
    fn test_create_record_agent_does_not_exist() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();
        let create_record_action = create_record_action_with_properties(vec![
            optional_property_value(),
            required_property_value(),
        ]);

        match transaction_handler._create_record(
            &create_record_action,
            &mut state,
            PUBLIC_KEY,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Agent does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("Agent is not registered: {}", PUBLIC_KEY)));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the CreateRecordAction is invalid if the schema does not exist.
    fn test_create_record_schema_does_not_exist() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let create_record_action = create_record_action_with_properties(vec![
            optional_property_value(),
            required_property_value(),
        ]);

        match transaction_handler._create_record(
            &create_record_action,
            &mut state,
            PUBLIC_KEY,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Schema does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("Schema does not exist {}", SCHEMA_NAME)));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the CreateRecordAction is invalid if the a record with the same id
    /// already exists.
    fn test_create_record_already_exist() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_schema();
        transaction_context.add_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let create_record_action = create_record_action_with_properties(vec![
            optional_property_value(),
            required_property_value(),
        ]);

        match transaction_handler._create_record(
            &create_record_action,
            &mut state,
            PUBLIC_KEY,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Record id is duplicated, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("Record already exists: {}", RECORD_ID)));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the CreateRecordAction is invalid if the the payload is missing a required
    /// property as defined in the schema.
    fn test_create_record_missing_required_property() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_schema();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let create_record_action =
            create_record_action_with_properties(vec![optional_property_value()]);

        match transaction_handler._create_record(
            &create_record_action,
            &mut state,
            PUBLIC_KEY,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Required property is missing, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!(
                    "Required property {} not provided",
                    REQUIRED_PROPERTY_NAME
                )));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the CreateRecordAction is invalid if the the payload has an extra property
    /// that is not defined in the schema.
    fn test_create_record_extra_property() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_schema();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let property_value_extra = PropertyValueBuilder::new()
            .with_name("invalid_property".to_string())
            .with_data_type(DataType::String)
            .with_string_value("Property that does not exist in schema".to_string())
            .build()
            .expect("Failed to build property value");

        let create_record_action = create_record_action_with_properties(vec![
            optional_property_value(),
            required_property_value(),
            property_value_extra.clone(),
        ]);

        match transaction_handler._create_record(
            &create_record_action,
            &mut state,
            PUBLIC_KEY,
            TIMESTAMP,
        ) {
            Ok(()) => panic!(
                "There is an invalid property in the payload,
                InvalidTransaction should be returned"
            ),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains("Provided property invalid_property is not in schema"));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the CreateRecordAction is invalid if a property value has a type that is not
    /// the same as the type in the property definition.
    fn test_create_record_invalid_property_value_type() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        transaction_context.add_agent(PUBLIC_KEY);
        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let property_value_wrong = PropertyValueBuilder::new()
            .with_name(REQUIRED_PROPERTY_NAME.to_string())
            .with_data_type(DataType::Number)
            .with_number_value(123)
            .build()
            .expect("Failed to build property value");

        let create_record_action = create_record_action_with_properties(vec![
            optional_property_value(),
            property_value_wrong.clone(),
        ]);

        match transaction_handler._create_record(
            &create_record_action,
            &mut state,
            PUBLIC_KEY,
            TIMESTAMP,
        ) {
            Ok(()) => panic!(
                "There is an invalid property value in the payload,
                InvalidTransaction should be returned"
            ),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!(
                    "Value provided for {} is the wrong type",
                    REQUIRED_PROPERTY_NAME
                )));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }


    fn optional_property_value() -> PropertyValue {
        PropertyValueBuilder::new()
            .with_name(OPTIONAL_PROPERTY_NAME.to_string())
            .with_data_type(DataType::Enum)
            .with_enum_value(1)
            .build()
            .expect("Failed to build property value")
    }

    fn required_property_value() -> PropertyValue {
        PropertyValueBuilder::new()
            .with_name(REQUIRED_PROPERTY_NAME.to_string())
            .with_data_type(DataType::String)
            .with_string_value("required_field".to_string())
            .build()
            .expect("Failed to build property value")
    }

    fn create_record_action_with_properties(properties: Vec<PropertyValue>) -> CreateRecordAction {
        CreateRecordActionBuilder::new()
            .with_record_id(RECORD_ID.to_string())
            .with_schema(SCHEMA_NAME.to_string())
            .with_properties(properties)
            .build()
            .expect("Failed to build CreateRecordAction")
    }

    fn create_finalize_record() -> FinalizeRecordAction {
        FinalizeRecordActionBuilder::new()
            .with_record_id(RECORD_ID.to_string())
            .build()
            .expect("Failed to build FinalizeRecordAction")
    }

    fn updated_property_value() -> PropertyValue {
        PropertyValueBuilder::new()
            .with_name(REQUIRED_PROPERTY_NAME.to_string())
            .with_data_type(DataType::String)
            .with_string_value("updated_required_field".to_string())
            .build()
            .expect("Failed to build property value")
    }

    fn update_property_action(properties: Vec<PropertyValue>) -> UpdatePropertiesAction {
        UpdatePropertiesActionBuilder::new()
            .with_record_id(RECORD_ID.to_string())
            .with_properties(properties)
            .build()
            .expect("Failed to build UpdatePropertiesAction")
    }

    fn create_proposal_action(role: Role, receiving_agent_key: &str) -> CreateProposalAction {
        CreateProposalActionBuilder::new()
            .with_record_id(RECORD_ID.to_string())
            .with_properties(vec![
                OPTIONAL_PROPERTY_NAME.to_string(),
                REQUIRED_PROPERTY_NAME.to_string(),
            ])
            .with_receiving_agent(receiving_agent_key.to_string())
            .with_role(role)
            .with_terms("".to_string())
            .build()
            .expect("Failed to build CreateProposalAction")
    }

    fn answer_proposal_action(
        role: Role,
        receiving_agent_key: &str,
        response: Response,
    ) -> AnswerProposalAction {
        AnswerProposalActionBuilder::new()
            .with_record_id(RECORD_ID.to_string())
            .with_receiving_agent(receiving_agent_key.to_string())
            .with_role(role)
            .with_response(response)
            .build()
            .expect("Failed to build AnswerProposalAction")
    }

    fn revoke_reporter_action(reporter_id: &str, properties: Vec<String>) -> RevokeReporterAction {
        RevokeReporterActionBuilder::new()
            .with_record_id(RECORD_ID.to_string())
            .with_reporter_id(reporter_id.to_string())
            .with_properties(properties)
            .build()
            .expect("Failed to build RevokeReporterAction")
    }

    fn optional_property_definition() -> PropertyDefinition {
        PropertyDefinitionBuilder::new()
            .with_name(OPTIONAL_PROPERTY_NAME.to_string())
            .with_data_type(DataType::Enum)
            .with_description("Optional".to_string())
            .with_enum_options(vec![
                "One".to_string(),
                "Two".to_string(),
                "Three".to_string(),
            ])
            .build()
            .expect("Failed to build property definition")
    }

    fn required_property_definition() -> PropertyDefinition {
        PropertyDefinitionBuilder::new()
            .with_name(REQUIRED_PROPERTY_NAME.to_string())
            .with_data_type(DataType::String)
            .with_description("Required".to_string())
            .with_required(true)
            .build()
            .expect("Failed to build property definition")
    }

    fn make_record() -> Record {
        let associated_agent = AssociatedAgentBuilder::new()
            .with_agent_id(PUBLIC_KEY.to_string())
            .with_timestamp(TIMESTAMP)
            .build()
            .expect("Failed to build AssociatedAgent");

        RecordBuilder::new()
            .with_record_id(RECORD_ID.to_string())
            .with_schema(SCHEMA_NAME.to_string())
            .with_owners(vec![associated_agent.clone()])
            .with_custodians(vec![associated_agent.clone()])
            .with_field_final(false)
            .build()
            .expect("Failed to build new_record")
    }

    fn make_property(property_name: &str, property_definition: PropertyDefinition) -> Property {
        let reporter = ReporterBuilder::new()
            .with_public_key(PUBLIC_KEY.to_string())
            .with_authorized(true)
            .with_index(0)
            .build()
            .expect("Failed to build Reporter");

        PropertyBuilder::new()
            .with_name(property_name.to_string())
            .with_record_id(RECORD_ID.to_string())
            .with_property_definition(property_definition)
            .with_reporters(vec![reporter.clone()])
            .with_current_page(1)
            .with_wrapped(false)
            .build()
            .expect("Failed to build property")
    }

    fn make_property_with_reporter(
        property_name: &str,
        reporter_key: &str,
        authorized: bool,
        property_definition: PropertyDefinition,
    ) -> Property {
        let reporter = ReporterBuilder::new()
            .with_public_key(reporter_key.to_string())
            .with_authorized(authorized)
            .with_index(0)
            .build()
            .expect("Failed to build Reporter");

        PropertyBuilder::new()
            .with_name(property_name.to_string())
            .with_record_id(RECORD_ID.to_string())
            .with_property_definition(property_definition)
            .with_reporters(vec![reporter.clone()])
            .with_current_page(1)
            .with_wrapped(false)
            .build()
            .expect("Failed to build property")
    }

    fn make_property_page(property_name: &str, property_value: PropertyValue) -> PropertyPage {
        let reported_value = ReportedValueBuilder::new()
            .with_reporter_index(0)
            .with_timestamp(TIMESTAMP)
            .with_value(property_value)
            .build()
            .expect("Failed to build ReportedValue");

        PropertyPageBuilder::new()
            .with_name(property_name.to_string())
            .with_record_id(RECORD_ID.to_string())
            .with_reported_values(vec![reported_value])
            .build()
            .expect("Failed to build PropertyPage")
    }

    fn make_proposal(
        issuing_agent: &str,
        receiving_agent_key: &str,
        role: Role,
        status: Status,
    ) -> Proposal {
        ProposalBuilder::new()
            .with_record_id(RECORD_ID.to_string())
            .with_timestamp(TIMESTAMP)
            .with_issuing_agent(issuing_agent.to_string())
            .with_receiving_agent(receiving_agent_key.to_string())
            .with_role(role)
            .with_properties(vec![
                OPTIONAL_PROPERTY_NAME.to_string(),
                REQUIRED_PROPERTY_NAME.to_string(),
            ])
            .with_status(status)
            .with_terms("".to_string())
            .build()
            .expect("Failed to build proposal")
    }
}
