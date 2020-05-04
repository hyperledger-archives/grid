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
            family_name: "grid-track-and-trace".to_string(),
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
            .with_custodians(vec![owner])
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

            if provided_properties.contains_key(property_name) {
                let mut new_property_page = PropertyPageBuilder::new()
                    .with_name(property_name.to_string())
                    .with_record_id(record_id.to_string());

                let provided_property = provided_properties[property_name].clone();
                let reported_value = ReportedValueBuilder::new()
                    .with_reporter_index(0)
                    .with_timestamp(timestamp)
                    .with_value(provided_property)
                    .build()
                    .map_err(|err| map_builder_error_to_apply_error(err, "ReportedValue"))?;

                new_property_page = new_property_page.with_reported_values(vec![reported_value]);
                state.set_property_page(
                    record_id,
                    property_name,
                    1,
                    new_property_page
                        .build()
                        .map_err(|err| map_builder_error_to_apply_error(err, "PropertyPage"))?,
                )?;
            }
        }

        Ok(())
    }

    fn _finalize_record(
        &self,
        payload: &FinalizeRecordAction,
        state: &mut TrackAndTraceState,
        signer: &str,
    ) -> Result<(), ApplyError> {
        let record_id = payload.record_id();
        let final_record = match state.get_record(record_id)? {
            Some(final_record) => final_record,
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Record does not exist: {}",
                    record_id
                )));
            }
        };
        let owner = match final_record.owners().last() {
            Some(x) => x,
            None => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Owner was not found",
                )));
            }
        };
        let custodian = match final_record.custodians().last() {
            Some(x) => x,
            None => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Custodian was not found",
                )));
            }
        };

        if owner.agent_id() != signer || custodian.agent_id() != signer {
            return Err(ApplyError::InvalidTransaction(
                "Must be owner and custodian to finalize record".to_string(),
            ));
        }
        if *final_record.field_final() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Record is already final: {}",
                record_id
            )));
        }

        let updated_record = final_record
            .clone()
            .into_builder()
            .with_field_final(true)
            .build()
            .map_err(|err| map_builder_error_to_apply_error(err, "Record"))?;

        state.set_record(record_id, updated_record)?;

        Ok(())
    }

    fn _update_properties(
        &self,
        payload: &UpdatePropertiesAction,
        state: &mut TrackAndTraceState,
        signer: &str,
        timestamp: u64,
    ) -> Result<(), ApplyError> {
        let record_id = payload.record_id();
        let update_record = match state.get_record(record_id)? {
            Some(update_record) => update_record,
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Record does not exist: {}",
                    record_id
                )));
            }
        };

        if *update_record.field_final() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Record is final: {}",
                record_id
            )));
        }

        let updates = payload.properties();

        for update in updates {
            let name = update.name();
            let data_type = update.data_type();

            let prop = match state.get_property(record_id, name)? {
                Some(prop) => prop,
                None => {
                    return Err(ApplyError::InvalidTransaction(format!(
                        "Record does not have provided property: {}",
                        name
                    )));
                }
            };

            let mut allowed = false;
            let mut reporter_index = 0;
            for reporter in prop.reporters() {
                if reporter.public_key() == signer && *reporter.authorized() {
                    allowed = true;
                    reporter_index = *reporter.index();
                    break;
                }
            }
            if !allowed {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Reporter is not authorized: {}",
                    signer
                )));
            }

            if data_type != prop.property_definition().data_type() {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Update has wrong type: {:?} != {:?}",
                    data_type,
                    prop.property_definition().data_type()
                )));
            }

            let page_number = prop.current_page();
            let page = match state.get_property_page(record_id, name, *page_number)? {
                Some(page) => page,
                None => PropertyPageBuilder::new()
                    .with_name(prop.name().to_string())
                    .with_record_id(record_id.to_string())
                    .with_reported_values(vec![])
                    .build()
                    .map_err(|err| map_builder_error_to_apply_error(err, "PropertyPage"))?,
            };

            let reported_value = ReportedValueBuilder::new()
                .with_reporter_index(reporter_index)
                .with_timestamp(timestamp)
                .with_value(update.clone())
                .build()
                .map_err(|err| map_builder_error_to_apply_error(err, "ReportedValue"))?;

            let mut updated_reported_values = page.reported_values().to_vec();
            updated_reported_values.push(reported_value);
            updated_reported_values.sort_by_key(|rv| (*rv.timestamp(), *rv.reporter_index()));

            let updated_property_page = page
                .clone()
                .into_builder()
                .with_reported_values(updated_reported_values)
                .build()
                .map_err(|err| map_builder_error_to_apply_error(err, "PropertyPage"))?;

            state.set_property_page(
                record_id,
                name,
                *page_number,
                updated_property_page.clone(),
            )?;

            if updated_property_page.reported_values().len() >= PROPERTY_PAGE_MAX_LENGTH {
                let new_page_number = if *page_number > PROPERTY_PAGE_MAX_LENGTH as u32 {
                    1
                } else {
                    page_number + 1
                };

                let new_page = match state.get_property_page(record_id, name, new_page_number)? {
                    Some(new_page) => new_page
                        .into_builder()
                        .with_reported_values(vec![])
                        .build()
                        .map_err(|err| map_builder_error_to_apply_error(err, "PropertyPage"))?,
                    None => PropertyPageBuilder::new()
                        .with_name(name.to_string())
                        .with_record_id(record_id.to_string())
                        .with_reported_values(vec![])
                        .build()
                        .map_err(|err| map_builder_error_to_apply_error(err, "PropertyPage"))?,
                };

                state.set_property_page(record_id, name, new_page_number, new_page)?;

                let wrapped = new_page_number == 1 && !prop.wrapped();
                let new_property = prop
                    .clone()
                    .into_builder()
                    .with_current_page(new_page_number)
                    .with_wrapped(wrapped)
                    .build()
                    .map_err(|err| map_builder_error_to_apply_error(err, "Property"))?;

                state.set_property(record_id, name, new_property)?;
            }
        }

        Ok(())
    }

    fn _create_proposal(
        &self,
        payload: &CreateProposalAction,
        state: &mut TrackAndTraceState,
        signer: &str,
        timestamp: u64,
    ) -> Result<(), ApplyError> {
        let record_id = payload.record_id();
        let receiving_agent = payload.receiving_agent();
        let role = payload.role();
        let properties = payload.properties();
        let terms = payload.terms();

        match state.get_agent(signer)? {
            Some(agent) => agent,
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Issuing agent does not exist: {}",
                    signer
                )));
            }
        };

        match state.get_agent(&receiving_agent)? {
            Some(agent) => agent,
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Receiving agent does not exist: {}",
                    receiving_agent
                )));
            }
        };

        let mut proposals = match state.get_proposal_list(&record_id, &receiving_agent)? {
            Some(proposals) => proposals.proposals().to_vec(),
            None => vec![],
        };

        let open_proposals = proposals
            .iter()
            .filter(|proposal| proposal.status() == &Status::Open)
            .collect::<Vec<_>>();

        for prop in open_proposals {
            if prop.receiving_agent() == receiving_agent
                && prop.role() == role
                && prop.record_id() == record_id
            {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Proposal already exists",
                )));
            }
        }

        let proposal_record = match state.get_record(&record_id)? {
            Some(record) => record,
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Record does not exist: {}",
                    record_id
                )));
            }
        };

        if *proposal_record.field_final() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Record is final: {}",
                record_id
            )));
        }

        if role == &Role::Owner {
            let owner = match proposal_record.owners().last() {
                Some(owner) => owner,
                None => {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Owner not found",
                    )));
                }
            };
            if owner.agent_id() != signer {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Only the owner can create a proposal to change ownership",
                )));
            }
        }

        if role == &Role::Custodian {
            let custodian = match proposal_record.custodians().last() {
                Some(custodian) => custodian,
                None => {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Custodian not found",
                    )));
                }
            };

            if custodian.agent_id() != signer {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Only the custodian can create a proposal to change custodianship",
                )));
            }
        }

        if role == &Role::Reporter {
            let owner = match proposal_record.owners().last() {
                Some(owner) => owner,
                None => {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Owner not found",
                    )));
                }
            };
            if owner.agent_id() != signer {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Only the owner can create a proposal to authorize a reporter",
                )));
            }
            if properties.is_empty() {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "No properties were specified for authorization",
                )));
            }
        }

        let new_proposal = ProposalBuilder::new()
            .with_record_id(record_id.to_string())
            .with_timestamp(timestamp)
            .with_issuing_agent(signer.to_string())
            .with_receiving_agent(receiving_agent.to_string())
            .with_role(role.clone())
            .with_properties(properties.to_vec())
            .with_status(Status::Open)
            .with_terms(terms.to_string())
            .build()
            .map_err(|err| map_builder_error_to_apply_error(err, "Proposal"))?;

        proposals.push(new_proposal);
        proposals.sort_by_key(|p| {
            (
                p.record_id().to_string(),
                p.receiving_agent().to_string(),
                *p.timestamp(),
            )
        });
        let proposal_list = ProposalListBuilder::new()
            .with_proposals(proposals)
            .build()
            .map_err(|err| map_builder_error_to_apply_error(err, "ProposalList"))?;

        state.set_proposal_list(&record_id, &receiving_agent, proposal_list)?;

        Ok(())
    }

    fn _answer_proposal(
        &self,
        payload: &AnswerProposalAction,
        state: &mut TrackAndTraceState,
        signer: &str,
        timestamp: u64,
    ) -> Result<(), ApplyError> {
        let record_id = payload.record_id();
        let receiving_agent = payload.receiving_agent();
        let role = payload.role();
        let response = payload.response();

        let mut proposals = match state.get_proposal_list(record_id, receiving_agent)? {
            Some(proposal_list) => proposal_list.proposals().to_vec(),
            None => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Proposal does not exist",
                )));
            }
        };

        // find proposal to answer or return error.
        let (proposal_index, current_proposal) = proposals
            .iter()
            .enumerate()
            .find_map(|(i, prop)| {
                if prop.receiving_agent() == receiving_agent
                    && prop.role() == role
                    && prop.record_id() == record_id
                    && prop.status() == &Status::Open
                {
                    Some((i, prop.clone()))
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                ApplyError::InvalidTransaction(format!(
                    "No open proposals found for record {} for {}",
                    record_id, receiving_agent
                ))
            })?;

        let mut updated_proposal_builder = current_proposal.clone().into_builder();

        match response {
            Response::Cancel => {
                if current_proposal.issuing_agent() != signer {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Only the issuing agent can cancel a proposal",
                    )));
                }
                updated_proposal_builder = updated_proposal_builder.with_status(Status::Canceled);
            }

            Response::Reject => {
                if current_proposal.receiving_agent() != signer {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Only the receiving agent can reject a proposal",
                    )));
                }
                updated_proposal_builder = updated_proposal_builder.with_status(Status::Rejected);
            }

            Response::Accept => {
                if current_proposal.receiving_agent() != signer {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Only the receiving agent can accept a proposal",
                    )));
                };

                let proposal_record = match state.get_record(record_id)? {
                    Some(record) => record,
                    None => {
                        return Err(ApplyError::InvalidTransaction(format!(
                            "Record in proposal does not exist: {}",
                            record_id
                        )));
                    }
                };

                let owner = match proposal_record.owners().last() {
                    Some(owner) => owner.clone(),
                    None => {
                        return Err(ApplyError::InvalidTransaction(String::from(
                            "Owner not found",
                        )));
                    }
                };

                let custodian = match proposal_record.custodians().last() {
                    Some(custodian) => custodian.clone(),
                    None => {
                        return Err(ApplyError::InvalidTransaction(String::from(
                            "Custodian not found",
                        )));
                    }
                };

                match role {
                    Role::Owner => {
                        if owner.agent_id() != current_proposal.issuing_agent() {
                            info!("Record owner does not match the issuing agent of the proposal");

                            updated_proposal_builder =
                                updated_proposal_builder.with_status(Status::Canceled);
                        } else {
                            let new_owner = AssociatedAgentBuilder::new()
                                .with_agent_id(receiving_agent.to_string())
                                .with_timestamp(timestamp)
                                .build()
                                .map_err(|err| {
                                    map_builder_error_to_apply_error(err, "AssociatedAgent")
                                })?;

                            let mut record_owners = proposal_record.owners().to_vec();
                            record_owners.push(new_owner);
                            let updated_record = proposal_record
                                .clone()
                                .into_builder()
                                .with_owners(record_owners)
                                .build()
                                .map_err(|err| map_builder_error_to_apply_error(err, "Record"))?;

                            state.set_record(record_id, updated_record)?;

                            let schema = match state.get_schema(proposal_record.schema())? {
                                Some(record_type) => record_type,
                                None => {
                                    return Err(ApplyError::InvalidTransaction(format!(
                                        "Schema does not exist: {}",
                                        proposal_record.schema()
                                    )));
                                }
                            };

                            for prop_schema in schema.properties() {
                                let prop =
                                    match state.get_property(record_id, prop_schema.name())? {
                                        Some(prop) => prop,
                                        None => {
                                            return Err(ApplyError::InvalidTransaction(format!(
                                                "Property does not exist: {}",
                                                prop_schema.name()
                                            )));
                                        }
                                    };

                                let mut authorized = false;
                                let mut new_reporters = prop
                                    .reporters()
                                    .to_vec()
                                    .iter()
                                    .map(|reporter| {
                                        if reporter.public_key() == owner.agent_id() {
                                            reporter
                                                .clone()
                                                .into_builder()
                                                .with_authorized(false)
                                                .build()
                                                .map_err(|err| {
                                                    map_builder_error_to_apply_error(
                                                        err, "Reporter",
                                                    )
                                                })
                                        } else if reporter.public_key() == receiving_agent {
                                            authorized = true;
                                            reporter
                                                .clone()
                                                .into_builder()
                                                .with_authorized(true)
                                                .build()
                                                .map_err(|err| {
                                                    map_builder_error_to_apply_error(
                                                        err, "Reporter",
                                                    )
                                                })
                                        } else {
                                            Ok(reporter.clone())
                                        }
                                    })
                                    .collect::<Result<Vec<_>, ApplyError>>()?;

                                if !authorized {
                                    let reporter = ReporterBuilder::new()
                                        .with_public_key(receiving_agent.to_string())
                                        .with_authorized(true)
                                        .with_index(prop.reporters().len() as u32)
                                        .build()
                                        .map_err(|err| {
                                            map_builder_error_to_apply_error(err, "Reporter")
                                        })?;
                                    new_reporters.push(reporter);
                                }

                                let updated_property = prop
                                    .clone()
                                    .into_builder()
                                    .with_reporters(new_reporters.clone())
                                    .build()
                                    .map_err(|err| {
                                        map_builder_error_to_apply_error(err, "Property")
                                    })?;

                                state.set_property(record_id, prop.name(), updated_property)?;
                            }
                            updated_proposal_builder =
                                updated_proposal_builder.with_status(Status::Accepted);
                        }
                    }
                    Role::Custodian => {
                        if custodian.agent_id() != current_proposal.issuing_agent() {
                            info!(
                                "Record custodian does not match the issuing agent of the proposal"
                            );
                            updated_proposal_builder =
                                updated_proposal_builder.with_status(Status::Canceled);
                        } else {
                            let new_custodian = AssociatedAgentBuilder::new()
                                .with_agent_id(receiving_agent.to_string())
                                .with_timestamp(timestamp)
                                .build()
                                .map_err(|err| {
                                    map_builder_error_to_apply_error(err, "AssociatedAgent")
                                })?;

                            let mut record_custodians = proposal_record.custodians().to_vec();
                            record_custodians.push(new_custodian);
                            let updated_record = proposal_record
                                .into_builder()
                                .with_custodians(record_custodians)
                                .build()
                                .map_err(|err| map_builder_error_to_apply_error(err, "Record"))?;

                            state.set_record(record_id, updated_record)?;

                            updated_proposal_builder =
                                updated_proposal_builder.with_status(Status::Accepted);
                        }
                    }
                    Role::Reporter => {
                        if owner.agent_id() != current_proposal.issuing_agent() {
                            info!("Record owner does not match the issuing agent of the proposal");

                            updated_proposal_builder =
                                updated_proposal_builder.with_status(Status::Canceled);
                        } else {
                            let reporter_builder = ReporterBuilder::new()
                                .with_public_key(receiving_agent.to_string())
                                .with_authorized(true);

                            for prop_name in current_proposal.properties() {
                                let prop = match state.get_property(record_id, prop_name)? {
                                    Some(prop) => prop,
                                    None => {
                                        return Err(ApplyError::InvalidTransaction(format!(
                                            "Property does not exist: {}",
                                            prop_name
                                        )));
                                    }
                                };
                                let reporter = reporter_builder
                                    .clone()
                                    .with_index(prop.reporters().len() as u32)
                                    .build()
                                    .map_err(|err| {
                                        map_builder_error_to_apply_error(err, "Reporter")
                                    })?;

                                let mut updated_reporter_list = prop.reporters().to_vec();
                                updated_reporter_list.push(reporter);
                                let updated_property = prop
                                    .clone()
                                    .into_builder()
                                    .with_reporters(updated_reporter_list)
                                    .build()
                                    .map_err(|err| {
                                        map_builder_error_to_apply_error(err, "Property")
                                    })?;
                                state.set_property(record_id, prop_name, updated_property)?;
                            }
                            updated_proposal_builder =
                                updated_proposal_builder.with_status(Status::Accepted);
                        }
                    }
                }
            }
        }
        let updated_proposal = updated_proposal_builder
            .build()
            .map_err(|err| map_builder_error_to_apply_error(err, "Proposal"))?;

        // remove outdated proposal
        proposals.remove(proposal_index);
        proposals.push(updated_proposal);
        proposals.sort_by_key(|p| {
            (
                p.record_id().to_string(),
                p.receiving_agent().to_string(),
                *p.timestamp(),
            )
        });

        let proposal_list = ProposalListBuilder::new()
            .with_proposals(proposals)
            .build()
            .map_err(|err| map_builder_error_to_apply_error(err, "ProposalList"))?;

        state.set_proposal_list(&record_id, &receiving_agent, proposal_list)?;

        Ok(())
    }

    fn _revoke_reporter(
        &self,
        payload: &RevokeReporterAction,
        state: &mut TrackAndTraceState,
        signer: &str,
    ) -> Result<(), ApplyError> {
        let record_id = payload.record_id();
        let reporter_id = payload.reporter_id();
        let properties = payload.properties();

        let revoke_record = match state.get_record(record_id)? {
            Some(record) => record,
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Record does not exists: {}",
                    record_id
                )));
            }
        };

        let owner = match revoke_record.owners().last() {
            Some(x) => x,
            None => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Owner was not found",
                )));
            }
        };

        if owner.agent_id() != signer {
            return Err(ApplyError::InvalidTransaction(
                "Must be owner to revoke reporters".to_string(),
            ));
        }

        if *revoke_record.field_final() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Record is final: {}",
                record_id
            )));
        }

        for prop_name in properties {
            let prop = match state.get_property(record_id, prop_name)? {
                Some(prop) => prop,
                None => {
                    return Err(ApplyError::InvalidTransaction(format!(
                        "Property does not exist: {}",
                        prop_name
                    )));
                }
            };

            let mut revoked = false;
            let new_reporters = prop
                .reporters()
                .to_vec()
                .iter()
                .map(|reporter| {
                    if reporter.public_key() == reporter_id {
                        if !*reporter.authorized() {
                            return Err(ApplyError::InvalidTransaction(
                                "Reporter is already unauthorized.".to_string(),
                            ));
                        }
                        revoked = true;
                        reporter
                            .clone()
                            .into_builder()
                            .with_authorized(false)
                            .build()
                            .map_err(|err| map_builder_error_to_apply_error(err, "Reporter"))
                    } else {
                        Ok(reporter.clone())
                    }
                })
                .collect::<Result<Vec<_>, ApplyError>>()?;

            if !revoked {
                return Err(ApplyError::InvalidTransaction(format!(
                    "{} not a reporter for property {}",
                    reporter_id, prop_name
                )));
            }
            let updated_property = prop
                .clone()
                .into_builder()
                .with_reporters(new_reporters)
                .build()
                .map_err(|err| map_builder_error_to_apply_error(err, "Property"))?;

            state.set_property(record_id, prop_name, updated_property)?;
        }

        Ok(())
    }
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

        info!(
            "Track and Trace payload: {:?} {}",
            payload.action(),
            payload.timestamp(),
        );

        match payload.action() {
            Action::CreateRecord(action_payload) => {
                self._create_record(action_payload, &mut state, signer, *payload.timestamp())?
            }
            Action::FinalizeRecord(action_payload) => {
                self._finalize_record(action_payload, &mut state, signer)?
            }
            Action::UpdateProperties(action_payload) => {
                self._update_properties(action_payload, &mut state, signer, *payload.timestamp())?
            }
            Action::CreateProposal(action_payload) => {
                self._create_proposal(action_payload, &mut state, signer, *payload.timestamp())?
            }
            Action::AnswerProposal(action_payload) => {
                self._answer_proposal(action_payload, &mut state, signer, *payload.timestamp())?
            }
            Action::RevokeReporter(action_payload) => {
                self._revoke_reporter(action_payload, &mut state, signer)?
            }
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
// Sabre apply must return a bool
fn apply(
    request: &TpProcessRequest,
    context: &mut dyn TransactionContext,
) -> Result<bool, ApplyError> {
    let handler = TrackAndTraceTransactionHandler::new();
    match handler.apply(request, context) {
        Ok(_) => Ok(true),
        Err(err) => {
            info!("{} received {}", handler.family_name(), err);
            Err(err)
        }
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

    #[test]
    /// Test that if the FinalizeRecordAction is valid an OK is returned and that the record is
    /// marked as finalized
    fn test_finalize_record_handler_valid() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        assert!(transaction_handler
            ._finalize_record(&create_finalize_record(), &mut state, PUBLIC_KEY)
            .is_ok());

        let finalized_record = state
            .get_record(RECORD_ID)
            .expect("Failed to fetch record")
            .expect("Record not found");

        assert!(finalized_record.field_final());
    }

    #[test]
    /// Test that if the FinalizeRecordAction fails if a record with the provided id does
    /// not exist.
    fn test_finalize_record_handler_record_does_not_exist() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        transaction_context.add_agent(PUBLIC_KEY);

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        match transaction_handler._finalize_record(
            &create_finalize_record(),
            &mut state,
            PUBLIC_KEY,
        ) {
            Ok(()) => panic!("Record does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("Record does not exist: {}", RECORD_ID)));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the FinalizeRecordAction fails if the signer is not record owner nor
    // custodian
    fn test_finalize_record_handler_signer_not_owner_nor_custodian() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);
        let signer = "agent_public_key_not_owner";

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        match transaction_handler._finalize_record(&create_finalize_record(), &mut state, signer) {
            Ok(()) => panic!(
                "Signer is not record owner nor custodian, InvalidTransaction should be returned"
            ),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains("Must be owner and custodian to finalize record"));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the FinalizeRecordAction fails if the record is alreadt final.
    fn test_finalize_record_handler_record_already_final() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_finalized_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        match transaction_handler._finalize_record(
            &create_finalize_record(),
            &mut state,
            PUBLIC_KEY,
        ) {
            Ok(()) => panic!("Record is already final, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("Record is already final: {}", RECORD_ID)));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the UpdatedPropertiesAction is valid an OK is returned and new value is added
    /// to the record's PropertyPage
    fn test_update_properties_valid() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_record();
        transaction_context.add_property(REQUIRED_PROPERTY_NAME, required_property_definition());
        transaction_context.add_property_page(REQUIRED_PROPERTY_NAME, required_property_value());

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();
        let update_property_action = update_property_action(vec![updated_property_value()]);

        assert!(transaction_handler
            ._update_properties(&update_property_action, &mut state, PUBLIC_KEY, TIMESTAMP,)
            .is_ok());

        let page = state
            .get_property_page(RECORD_ID, REQUIRED_PROPERTY_NAME, 1)
            .expect("Failed to get property page from state")
            .expect("Property page is none, it should be some");

        assert_eq!(page.reported_values().len(), 2);
        assert_eq!(page.reported_values()[1].value(), &updated_property_value());
    }

    #[test]
    /// Test that if the UpdatedPropertiesAction fails if the record does not exist
    fn test_update_properties_record_does_not_exist() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_finalized_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();
        let update_property_action = update_property_action(vec![updated_property_value()]);

        match transaction_handler._update_properties(
            &update_property_action,
            &mut state,
            PUBLIC_KEY,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Record is does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("Record is final: {}", RECORD_ID)));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the UpdatedPropertiesAction fails if the record is set to final
    fn test_update_properties_record_is_final() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_finalized_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();
        let update_property_action = update_property_action(vec![updated_property_value()]);

        match transaction_handler._update_properties(
            &update_property_action,
            &mut state,
            PUBLIC_KEY,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Record is does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("Record is final: {}", RECORD_ID)));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the UpdatedPropertiesAction fails if the property to be updated is not part of
    /// the record
    fn test_update_properties_record_missing_property() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();
        let update_property_action = update_property_action(vec![updated_property_value()]);

        match transaction_handler._update_properties(
            &update_property_action,
            &mut state,
            PUBLIC_KEY,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Record is does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!(
                    "Record does not have provided property: {}",
                    REQUIRED_PROPERTY_NAME
                )));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the UpdatedPropertiesAction fails if the signer is not an authorized reporter
    fn test_update_properties_signer_not_authorized() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_record();
        transaction_context.add_property(REQUIRED_PROPERTY_NAME, required_property_definition());
        transaction_context.add_property_page(REQUIRED_PROPERTY_NAME, required_property_value());

        let signer = "not_authorized";
        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();
        let update_property_action = update_property_action(vec![updated_property_value()]);

        match transaction_handler._update_properties(
            &update_property_action,
            &mut state,
            signer.clone(),
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Record is does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("Reporter is not authorized: {}", signer)));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the UpdatedPropertiesAction fails if the property to be updated is set to
    /// a data type that does not match the property definition.
    fn test_update_properties_property_value_wrong_type() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_record();
        transaction_context.add_property(REQUIRED_PROPERTY_NAME, required_property_definition());
        transaction_context.add_property_page(REQUIRED_PROPERTY_NAME, required_property_value());

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let property_value_wrong = PropertyValueBuilder::new()
            .with_name(REQUIRED_PROPERTY_NAME.to_string())
            .with_data_type(DataType::Number)
            .with_number_value(123)
            .build()
            .expect("Failed to build property value");

        let transaction_handler = TrackAndTraceTransactionHandler::new();
        let update_property_action = update_property_action(vec![property_value_wrong]);

        match transaction_handler._update_properties(
            &update_property_action,
            &mut state,
            PUBLIC_KEY,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Record is does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!(
                    "Update has wrong type: {:?} != {:?}",
                    DataType::Number,
                    DataType::String
                )));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the UpdatedPropertiesAction start new PropertyPage when needed.
    fn test_update_properties_new_page() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_record();
        transaction_context.add_property(REQUIRED_PROPERTY_NAME, required_property_definition());
        transaction_context.add_property_page(REQUIRED_PROPERTY_NAME, required_property_value());

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();
        let property_value = updated_property_value();

        // Generate enough value updates that would required the start of a new PropertyPage.
        let updates = std::iter::repeat(property_value)
            .take(PROPERTY_PAGE_MAX_LENGTH)
            .collect::<Vec<_>>();

        let update_property_action = update_property_action(updates);

        assert!(transaction_handler
            ._update_properties(&update_property_action, &mut state, PUBLIC_KEY, TIMESTAMP,)
            .is_ok());

        let new_page = state
            .get_property_page(RECORD_ID, REQUIRED_PROPERTY_NAME, 2)
            .expect("Failed to get property page from state");

        assert!(new_page.is_some());
    }

    #[test]
    /// Test that if the CreateProposalAction, with role set to Owner, is valid an OK is returned
    /// and new proposal is added state
    fn test_create_proposal_valid_owner() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_agent(receiving_agent_key);
        transaction_context.add_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        assert!(transaction_handler
            ._create_proposal(
                &create_proposal_action(Role::Owner, receiving_agent_key),
                &mut state,
                PUBLIC_KEY,
                TIMESTAMP,
            )
            .is_ok());

        let proposal_list = state
            .get_proposal_list(RECORD_ID, receiving_agent_key)
            .expect("Failed to get ProposalList from state")
            .expect("ProposalList not found");

        assert_eq!(proposal_list.proposals().len(), 1);
        assert_eq!(
            proposal_list.proposals()[0],
            make_proposal(PUBLIC_KEY, receiving_agent_key, Role::Owner, Status::Open)
        );
    }

    #[test]
    /// Test that if the CreateProposalAction, with role set to Custodian, is valid an OK is returned
    /// and new proposal is added state
    fn test_create_proposal_valid_custodian() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_agent(receiving_agent_key);
        transaction_context.add_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        assert!(transaction_handler
            ._create_proposal(
                &create_proposal_action(Role::Custodian, receiving_agent_key),
                &mut state,
                PUBLIC_KEY,
                TIMESTAMP,
            )
            .is_ok());

        let proposal_list = state
            .get_proposal_list(RECORD_ID, receiving_agent_key)
            .expect("Failed to get ProposalList from state")
            .expect("ProposalList not found");

        assert_eq!(proposal_list.proposals().len(), 1);
        assert_eq!(
            proposal_list.proposals()[0],
            make_proposal(
                PUBLIC_KEY,
                receiving_agent_key,
                Role::Custodian,
                Status::Open
            )
        );
    }

    #[test]
    /// Test that if the CreateProposalAction fails when the signer is not an agent
    fn test_create_proposal_agent_does_not_exist() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        transaction_context.add_agent(receiving_agent_key);

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        match transaction_handler._create_proposal(
            &create_proposal_action(Role::Owner, receiving_agent_key),
            &mut state,
            PUBLIC_KEY,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Signer is not an Agent, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("Issuing agent does not exist: {}", PUBLIC_KEY,)));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the CreateProposalAction fails when the receiving agent key is not a valid agent
    fn test_create_proposal_receiving_agent_does_not_exist() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        transaction_context.add_agent(PUBLIC_KEY);

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        match transaction_handler._create_proposal(
            &create_proposal_action(Role::Owner, receiving_agent_key),
            &mut state,
            PUBLIC_KEY,
            TIMESTAMP,
        ) {
            Ok(()) => panic!(
                "Receiving agent key is not a valid Agent, InvalidTransaction should be returned"
            ),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!(
                    "Receiving agent does not exist: {}",
                    receiving_agent_key,
                )));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the CreateProposalAction fails when the a similar proposal already exists
    fn test_create_proposal_already_exists() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_agent(receiving_agent_key);
        transaction_context.add_proposal(
            PUBLIC_KEY,
            receiving_agent_key,
            Role::Owner,
            Status::Open,
        );

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        match transaction_handler._create_proposal(
            &create_proposal_action(Role::Owner, receiving_agent_key),
            &mut state,
            PUBLIC_KEY,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Proposal already exists, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains("Proposal already exists"));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the CreateProposalAction fails when the record does not exist
    fn test_create_proposal_record_does_not_exist() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_agent(receiving_agent_key);

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        match transaction_handler._create_proposal(
            &create_proposal_action(Role::Owner, receiving_agent_key),
            &mut state,
            PUBLIC_KEY,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Record does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("Record does not exist: {}", RECORD_ID)));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the CreateProposalAction fails when the record is already final
    fn test_create_proposal_record_is_final() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_agent(receiving_agent_key);
        transaction_context.add_finalized_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        match transaction_handler._create_proposal(
            &create_proposal_action(Role::Owner, receiving_agent_key),
            &mut state,
            PUBLIC_KEY,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Record is final, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("Record is final: {}", RECORD_ID)));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the CreateProposalAction fails when the signer is not owner and tries to
    /// transfer owenership
    fn test_create_proposal_signer_not_owner() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        let signer_public_key = "not_owner_agent_key";
        transaction_context.add_agent(signer_public_key);
        transaction_context.add_agent(receiving_agent_key);
        transaction_context.add_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        match transaction_handler._create_proposal(
            &create_proposal_action(Role::Owner, receiving_agent_key),
            &mut state,
            signer_public_key,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Signer is not owner, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains("Only the owner can create a proposal to change ownership"));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the CreateProposalAction fails when the signer is not custodian and tries to
    /// transfer custodianship
    fn test_create_proposal_signer_not_custodian() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        let signer_public_key = "not_custodian_agent_key";
        transaction_context.add_agent(signer_public_key);
        transaction_context.add_agent(receiving_agent_key);
        transaction_context.add_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        match transaction_handler._create_proposal(
            &create_proposal_action(Role::Custodian, receiving_agent_key),
            &mut state,
            signer_public_key,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Signer is not custodian, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err
                    .contains("Only the custodian can create a proposal to change custodianship"));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that the CreateProposalAction fails when a reporter authorization is proposed but no
    /// properties are specified
    fn test_create_proposal_reporter_with_no_properties() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_agent(receiving_agent_key);
        transaction_context.add_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        match transaction_handler._create_proposal(
            &create_proposal_no_props_action(Role::Reporter, receiving_agent_key),
            &mut state,
            PUBLIC_KEY,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("No properties specified, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains("No properties were specified for authorization"));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that if the AnswerProposalAction, with Reponse set to Cancel, is valid an OK is returned
    /// and the proposal is updated to have status Canceled
    fn test_answer_proposal_cancel_ok() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_agent(receiving_agent_key);
        transaction_context.add_record();
        transaction_context.add_proposal(
            PUBLIC_KEY,
            receiving_agent_key,
            Role::Owner,
            Status::Open,
        );

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload = answer_proposal_action(Role::Owner, receiving_agent_key, Response::Cancel);
        assert!(transaction_handler
            ._answer_proposal(&payload, &mut state, PUBLIC_KEY, TIMESTAMP,)
            .is_ok());

        let proposal_list = state
            .get_proposal_list(RECORD_ID, receiving_agent_key)
            .expect("Failed to get ProposalList from state")
            .expect("ProposalList not found");

        assert_eq!(proposal_list.proposals().len(), 1);
        assert_eq!(
            proposal_list.proposals()[0],
            make_proposal(
                PUBLIC_KEY,
                receiving_agent_key,
                Role::Owner,
                Status::Canceled
            )
        );
    }

    #[test]
    /// Test that if the AnswerProposalAction, with Reponse set to Reject, is valid an OK is returned
    /// and the proposal is updated to have status Rejected
    fn test_answer_proposal_reject_ok() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_agent(receiving_agent_key);
        transaction_context.add_record();
        transaction_context.add_proposal(
            PUBLIC_KEY,
            receiving_agent_key,
            Role::Owner,
            Status::Open,
        );

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload = answer_proposal_action(Role::Owner, receiving_agent_key, Response::Reject);

        assert!(transaction_handler
            ._answer_proposal(&payload, &mut state, receiving_agent_key, TIMESTAMP,)
            .is_ok());

        let proposal_list = state
            .get_proposal_list(RECORD_ID, receiving_agent_key)
            .expect("Failed to get ProposalList from state")
            .expect("ProposalList not found");

        assert_eq!(proposal_list.proposals().len(), 1);
        assert_eq!(
            proposal_list.proposals()[0],
            make_proposal(
                PUBLIC_KEY,
                receiving_agent_key,
                Role::Owner,
                Status::Rejected
            )
        );
    }

    #[test]
    /// Test that when the AnswerProposalAction, with Reponse set to Accept and Role set to Owner,
    /// is valid an OK is returned and the proposal is updated to have status Accepted, the owner
    /// in the record has been updated and the reporters for the properties have been updated.
    fn test_answer_proposal_accept_owner_ok() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_agent(receiving_agent_key);
        transaction_context.add_record();
        transaction_context.add_schema();
        transaction_context.add_property(OPTIONAL_PROPERTY_NAME, optional_property_definition());
        transaction_context.add_property(REQUIRED_PROPERTY_NAME, required_property_definition());

        transaction_context.add_proposal(
            PUBLIC_KEY,
            receiving_agent_key,
            Role::Owner,
            Status::Open,
        );

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload = answer_proposal_action(Role::Owner, receiving_agent_key, Response::Accept);

        assert!(transaction_handler
            ._answer_proposal(&payload, &mut state, receiving_agent_key, TIMESTAMP,)
            .is_ok());

        let proposal_list = state
            .get_proposal_list(RECORD_ID, receiving_agent_key)
            .expect("Failed to get ProposalList from state")
            .expect("ProposalList not found");

        assert_eq!(proposal_list.proposals().len(), 1);
        assert_eq!(
            proposal_list.proposals()[0],
            make_proposal(
                PUBLIC_KEY,
                receiving_agent_key,
                Role::Owner,
                Status::Accepted
            )
        );

        let record = state
            .get_record(RECORD_ID)
            .expect("Failed to fetch record")
            .expect("No record found");

        let old_owner = AssociatedAgentBuilder::new()
            .with_agent_id(PUBLIC_KEY.to_string())
            .with_timestamp(TIMESTAMP)
            .build()
            .expect("Failed to build AssociatedAgent");

        let new_owner = AssociatedAgentBuilder::new()
            .with_agent_id(receiving_agent_key.to_string())
            .with_timestamp(TIMESTAMP)
            .build()
            .expect("Failed to build AssociatedAgent");

        // Create record that has the receiving agent as a owner.
        let expected_record = make_record()
            .into_builder()
            .with_owners(vec![old_owner, new_owner])
            .build()
            .expect("Failed to build record");

        // Compare record found in state, whith what would be expected if the transaction was
        // executed correctly
        assert_eq!(record, expected_record);

        let required_property = state
            .get_property(RECORD_ID, REQUIRED_PROPERTY_NAME)
            .expect("Failed to fetch required property")
            .expect("Required property not found");

        let old_reporter = ReporterBuilder::new()
            .with_public_key(PUBLIC_KEY.to_string())
            .with_authorized(false)
            .with_index(0)
            .build()
            .expect("Failed to build Reporter");

        let new_reporter = ReporterBuilder::new()
            .with_public_key(receiving_agent_key.to_string())
            .with_authorized(true)
            .with_index(1)
            .build()
            .expect("Failed to build Reporter");

        // Create record that has the receiving agent as a reporter.
        let expected_property =
            make_property(REQUIRED_PROPERTY_NAME, required_property_definition())
                .into_builder()
                .with_reporters(vec![old_reporter, new_reporter])
                .build()
                .expect("Failed to build property");

        // Compare property found in state, whith what would be expected if the transaction was
        // executed correctly
        assert_eq!(required_property, expected_property);
    }

    #[test]
    /// Test that when the AnswerProposalAction, with Reponse set to Accept and Role set to Custodian,
    /// is valid an OK is returned and the proposal is updated to have status Accepted, the custodians
    /// in the record have been updated.
    fn test_answer_proposal_accept_custodian_ok() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_agent(receiving_agent_key);
        transaction_context.add_record();
        transaction_context.add_schema();
        transaction_context.add_property(OPTIONAL_PROPERTY_NAME, optional_property_definition());
        transaction_context.add_property(REQUIRED_PROPERTY_NAME, required_property_definition());

        transaction_context.add_proposal(
            PUBLIC_KEY,
            receiving_agent_key,
            Role::Custodian,
            Status::Open,
        );

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload =
            answer_proposal_action(Role::Custodian, receiving_agent_key, Response::Accept);

        assert!(transaction_handler
            ._answer_proposal(&payload, &mut state, receiving_agent_key, TIMESTAMP,)
            .is_ok());

        let proposal_list = state
            .get_proposal_list(RECORD_ID, receiving_agent_key)
            .expect("Failed to get ProposalList from state")
            .expect("ProposalList not found");

        assert_eq!(proposal_list.proposals().len(), 1);
        assert_eq!(
            proposal_list.proposals()[0],
            make_proposal(
                PUBLIC_KEY,
                receiving_agent_key,
                Role::Custodian,
                Status::Accepted
            )
        );

        let record = state
            .get_record(RECORD_ID)
            .expect("Failed to fetch record")
            .expect("No record found");

        let old_custodian = AssociatedAgentBuilder::new()
            .with_agent_id(PUBLIC_KEY.to_string())
            .with_timestamp(TIMESTAMP)
            .build()
            .expect("Failed to build AssociatedAgent");

        let new_custodian = AssociatedAgentBuilder::new()
            .with_agent_id(receiving_agent_key.to_string())
            .with_timestamp(TIMESTAMP)
            .build()
            .expect("Failed to build AssociatedAgent");

        // Create record that has receiving agent as a custodian
        let expected_record = make_record()
            .into_builder()
            .with_custodians(vec![old_custodian, new_custodian])
            .build()
            .expect("Failed to build record");

        // Compare record found in state, whith what would be expected if the transaction was
        // executed correctly
        assert_eq!(record, expected_record);
    }

    #[test]
    /// Test that when the AnswerProposalAction, with Reponse set to Accept and Role set to Reporter,
    /// is valid an OK is returned and the proposal is updated to have status Accepted,
    /// and the reporters for the properties have been updated.
    fn test_answer_proposal_accept_reporter_ok() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_agent(receiving_agent_key);
        transaction_context.add_record();
        transaction_context.add_schema();
        transaction_context.add_property(OPTIONAL_PROPERTY_NAME, optional_property_definition());
        transaction_context.add_property(REQUIRED_PROPERTY_NAME, required_property_definition());
        transaction_context.add_proposal(
            PUBLIC_KEY,
            receiving_agent_key,
            Role::Reporter,
            Status::Open,
        );

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload = answer_proposal_action(Role::Reporter, receiving_agent_key, Response::Accept);

        assert!(transaction_handler
            ._answer_proposal(&payload, &mut state, receiving_agent_key, TIMESTAMP,)
            .is_ok());

        let proposal_list = state
            .get_proposal_list(RECORD_ID, receiving_agent_key)
            .expect("Failed to get ProposalList from state")
            .expect("ProposalList not found");

        assert_eq!(proposal_list.proposals().len(), 1);
        assert_eq!(
            proposal_list.proposals()[0],
            make_proposal(
                PUBLIC_KEY,
                receiving_agent_key,
                Role::Reporter,
                Status::Accepted
            )
        );

        let required_property = state
            .get_property(RECORD_ID, REQUIRED_PROPERTY_NAME)
            .expect("Failed to fetch required property")
            .expect("Required property not found");

        let optional_property = state
            .get_property(RECORD_ID, OPTIONAL_PROPERTY_NAME)
            .expect("Failed to fetch optional property")
            .expect("Optional property not found");

        let old_reporter = ReporterBuilder::new()
            .with_public_key(PUBLIC_KEY.to_string())
            .with_authorized(true)
            .with_index(0)
            .build()
            .expect("Failed to build Reporter");

        let new_reporter = ReporterBuilder::new()
            .with_public_key(receiving_agent_key.to_string())
            .with_authorized(true)
            .with_index(1)
            .build()
            .expect("Failed to build Reporter");

        // Create required property that has receiving agent as an reporter
        let expected_required_property =
            make_property(REQUIRED_PROPERTY_NAME, required_property_definition())
                .into_builder()
                .with_reporters(vec![old_reporter.clone(), new_reporter.clone()])
                .build()
                .expect("Failed to build property");

        // Create optional property that has receiving agent as an reporter
        let expected_optional_property =
            make_property(OPTIONAL_PROPERTY_NAME, optional_property_definition())
                .into_builder()
                .with_reporters(vec![old_reporter.clone(), new_reporter.clone()])
                .build()
                .expect("Failed to build property");

        // Compare properties found in state, whith what would be expected if the transaction was
        // executed correctly
        assert_eq!(required_property, expected_required_property);
        assert_eq!(optional_property, expected_optional_property);
    }

    #[test]
    /// Test that when the AnswerProposalAction fails if the proposal does not exists.
    fn test_answer_proposal_does_not_exist() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload = answer_proposal_action(Role::Reporter, receiving_agent_key, Response::Accept);

        match transaction_handler._answer_proposal(
            &payload,
            &mut state,
            receiving_agent_key,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Proposal does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!(
                    "No open proposals found for record {} for {}",
                    RECORD_ID, receiving_agent_key
                )));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that when the AnswerProposalAction fails if the response is set to cancel and signer
    /// is not the proposal's issuing agent
    fn test_answer_proposal_signer_not_issuing_agent() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";

        transaction_context.add_proposal(
            PUBLIC_KEY,
            receiving_agent_key,
            Role::Reporter,
            Status::Open,
        );

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload = answer_proposal_action(Role::Reporter, receiving_agent_key, Response::Cancel);

        match transaction_handler._answer_proposal(
            &payload,
            &mut state,
            receiving_agent_key,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Signer not issuing agent, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains("Only the issuing agent can cancel a proposal"));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that when the AnswerProposalAction fails if the response is set to reject and signer
    /// is not the proposal's receiving agent
    fn test_answer_proposal_signer_not_receiving_agent_reject() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";

        transaction_context.add_proposal(
            PUBLIC_KEY,
            receiving_agent_key,
            Role::Reporter,
            Status::Open,
        );

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload = answer_proposal_action(Role::Reporter, receiving_agent_key, Response::Reject);

        match transaction_handler._answer_proposal(&payload, &mut state, PUBLIC_KEY, TIMESTAMP) {
            Ok(()) => panic!("Signer not issuing agent, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains("Only the receiving agent can reject a proposal"));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that when the AnswerProposalAction fails if the response is set to accept and signer
    /// is not the proposal's receiving agent
    fn test_answer_proposal_signer_not_receiving_agent_accept() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";

        transaction_context.add_proposal(
            PUBLIC_KEY,
            receiving_agent_key,
            Role::Reporter,
            Status::Open,
        );

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload = answer_proposal_action(Role::Reporter, receiving_agent_key, Response::Accept);

        match transaction_handler._answer_proposal(&payload, &mut state, PUBLIC_KEY, TIMESTAMP) {
            Ok(()) => panic!("Signer not issuing agent, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains("Only the receiving agent can accept a proposal"));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that when the AnswerProposalAction fails if the response is set to accept and the
    /// record does not exist
    fn test_answer_proposal_record_does_not_exist() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";

        transaction_context.add_proposal(
            PUBLIC_KEY,
            receiving_agent_key,
            Role::Reporter,
            Status::Open,
        );

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload = answer_proposal_action(Role::Reporter, receiving_agent_key, Response::Accept);

        match transaction_handler._answer_proposal(
            &payload,
            &mut state,
            receiving_agent_key,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Signer not issuing agent, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("Record in proposal does not exist: {}", RECORD_ID)));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that when the AnswerProposalAction is Ok if the response is set to accept and the
    /// role is set to owner, but the proposal's issuing agent is not the owner of the record.
    /// The status of the proposal should be set to Canceled.
    fn test_answer_proposal_issuing_agent_not_owner() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        let issuing_agent_key = "issuing_agent_key";

        transaction_context.add_proposal(
            issuing_agent_key,
            receiving_agent_key,
            Role::Owner,
            Status::Open,
        );
        transaction_context.add_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload = answer_proposal_action(Role::Owner, receiving_agent_key, Response::Accept);

        assert!(transaction_handler
            ._answer_proposal(&payload, &mut state, receiving_agent_key, TIMESTAMP,)
            .is_ok());

        let proposal_list = state
            .get_proposal_list(RECORD_ID, receiving_agent_key)
            .expect("Failed to get ProposalList from state")
            .expect("ProposalList not found");

        assert_eq!(proposal_list.proposals().len(), 1);
        assert_eq!(
            proposal_list.proposals()[0],
            make_proposal(
                issuing_agent_key,
                receiving_agent_key,
                Role::Owner,
                Status::Canceled
            )
        );
    }

    #[test]
    /// Test that when the AnswerProposalAction fails if the response is set to accept and the
    /// role is set to owner, but the record's schema does not exist.
    fn test_answer_proposal_schema_does_not_exist() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";

        transaction_context.add_proposal(
            PUBLIC_KEY,
            receiving_agent_key,
            Role::Owner,
            Status::Open,
        );
        transaction_context.add_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload = answer_proposal_action(Role::Owner, receiving_agent_key, Response::Accept);

        match transaction_handler._answer_proposal(
            &payload,
            &mut state,
            receiving_agent_key,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Schema does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("Schema does not exist: {}", SCHEMA_NAME)));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that when the AnswerProposalAction fails if the response is set to accept and the
    /// role is set to owner, schema's properties does not exist.
    fn test_answer_proposal_schema_property_does_not_exist() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";

        transaction_context.add_proposal(
            PUBLIC_KEY,
            receiving_agent_key,
            Role::Owner,
            Status::Open,
        );
        transaction_context.add_record();
        transaction_context.add_schema();
        transaction_context.add_property(REQUIRED_PROPERTY_NAME, required_property_definition());

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload = answer_proposal_action(Role::Owner, receiving_agent_key, Response::Accept);

        match transaction_handler._answer_proposal(
            &payload,
            &mut state,
            receiving_agent_key,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Property does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!(
                    "Property does not exist: {}",
                    OPTIONAL_PROPERTY_NAME
                )));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that when the AnswerProposalAction is Ok if the response is set to accept and the
    /// role is set to custodian, but the proposal's issuing agent is not the custodian of the
    /// record. The status of the proposal should be set to Canceled.
    fn test_answer_proposal_issuing_agent_not_custodian() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        let issuing_agent_key = "issuing_agent_key";

        transaction_context.add_proposal(
            issuing_agent_key,
            receiving_agent_key,
            Role::Custodian,
            Status::Open,
        );
        transaction_context.add_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload =
            answer_proposal_action(Role::Custodian, receiving_agent_key, Response::Accept);

        assert!(transaction_handler
            ._answer_proposal(&payload, &mut state, receiving_agent_key, TIMESTAMP,)
            .is_ok());

        let proposal_list = state
            .get_proposal_list(RECORD_ID, receiving_agent_key)
            .expect("Failed to get ProposalList from state")
            .expect("ProposalList not found");

        assert_eq!(proposal_list.proposals().len(), 1);
        assert_eq!(
            proposal_list.proposals()[0],
            make_proposal(
                issuing_agent_key,
                receiving_agent_key,
                Role::Custodian,
                Status::Canceled
            )
        );
    }

    #[test]
    /// Test that when the AnswerProposalAction is Ok if the response is set to accept and the
    /// role is set to reporter, but the proposal's issuing agent is not the owner of the record.
    /// The status of the proposal should be set to Canceled.
    fn test_answer_proposal_issuing_agent_not_owner_reporter() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";
        let issuing_agent_key = "issuing_agent_key";

        transaction_context.add_proposal(
            issuing_agent_key,
            receiving_agent_key,
            Role::Reporter,
            Status::Open,
        );
        transaction_context.add_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload = answer_proposal_action(Role::Reporter, receiving_agent_key, Response::Accept);

        assert!(transaction_handler
            ._answer_proposal(&payload, &mut state, receiving_agent_key, TIMESTAMP,)
            .is_ok());

        let proposal_list = state
            .get_proposal_list(RECORD_ID, receiving_agent_key)
            .expect("Failed to get ProposalList from state")
            .expect("ProposalList not found");

        assert_eq!(proposal_list.proposals().len(), 1);
        assert_eq!(
            proposal_list.proposals()[0],
            make_proposal(
                issuing_agent_key,
                receiving_agent_key,
                Role::Reporter,
                Status::Canceled
            )
        );
    }

    #[test]
    /// Test that when the AnswerProposalAction fails if the response is set to accept and the
    /// role is set to reporter, but one of the proposal's properties does not exist.
    fn test_answer_proposal_property_does_not_exist() {
        let mut transaction_context = MockTransactionContext::default();
        let receiving_agent_key = "receiving_agent_key";

        transaction_context.add_proposal(
            PUBLIC_KEY,
            receiving_agent_key,
            Role::Reporter,
            Status::Open,
        );
        transaction_context.add_record();
        transaction_context.add_schema();
        transaction_context.add_property(REQUIRED_PROPERTY_NAME, required_property_definition());

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload = answer_proposal_action(Role::Reporter, receiving_agent_key, Response::Accept);

        match transaction_handler._answer_proposal(
            &payload,
            &mut state,
            receiving_agent_key,
            TIMESTAMP,
        ) {
            Ok(()) => panic!("Schema does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!(
                    "Property does not exist: {}",
                    OPTIONAL_PROPERTY_NAME
                )));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that when the RevokeReporterAction is valid an Ok is returned and the properties
    /// reporters are updated.
    fn test_revoke_reporter_valid() {
        let mut transaction_context = MockTransactionContext::default();
        let reporter_key = "reporter_key";

        transaction_context.add_record();

        transaction_context.add_property_with_reporter(
            REQUIRED_PROPERTY_NAME,
            reporter_key,
            true,
            required_property_definition(),
        );

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload =
            revoke_reporter_action(reporter_key, vec![REQUIRED_PROPERTY_NAME.to_string()]);

        assert!(transaction_handler
            ._revoke_reporter(&payload, &mut state, PUBLIC_KEY,)
            .is_ok());

        let required_property = state
            .get_property(RECORD_ID, REQUIRED_PROPERTY_NAME)
            .expect("Failed to fetch required property")
            .expect("Required property not found");

        assert_eq!(
            required_property,
            make_property_with_reporter(
                REQUIRED_PROPERTY_NAME,
                reporter_key,
                false,
                required_property_definition()
            )
        );
    }

    #[test]
    /// Test that when the RevokeReporterAction fails if the record does not exist
    fn test_revoke_reporter_record_does_not_exist() {
        let mut transaction_context = MockTransactionContext::default();
        let reporter_key = "reporter_key";

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload =
            revoke_reporter_action(reporter_key, vec![REQUIRED_PROPERTY_NAME.to_string()]);

        match transaction_handler._revoke_reporter(&payload, &mut state, PUBLIC_KEY) {
            Ok(()) => panic!("Record does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("Record does not exists: {}", RECORD_ID)));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that when the RevokeReporterAction fails if the signer is not the record's owner
    fn test_revoke_reporter_signer_not_owner() {
        let mut transaction_context = MockTransactionContext::default();
        let reporter_key = "reporter_key";
        transaction_context.add_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload =
            revoke_reporter_action(reporter_key, vec![REQUIRED_PROPERTY_NAME.to_string()]);

        match transaction_handler._revoke_reporter(&payload, &mut state, "not_owner_key") {
            Ok(()) => panic!("Signer not owner, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains("Must be owner to revoke reporters"));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that when the RevokeReporterAction fails if the record is final
    fn test_revoke_reporter_record_is_final() {
        let mut transaction_context = MockTransactionContext::default();
        let reporter_key = "reporter_key";
        transaction_context.add_finalized_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload =
            revoke_reporter_action(reporter_key, vec![REQUIRED_PROPERTY_NAME.to_string()]);

        match transaction_handler._revoke_reporter(&payload, &mut state, PUBLIC_KEY) {
            Ok(()) => panic!("Record is final, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("Record is final: {}", RECORD_ID)));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that when the RevokeReporterAction fails if one of the payload's properties do not
    /// exist
    fn test_revoke_reporter_property_does_not_exist() {
        let mut transaction_context = MockTransactionContext::default();
        let reporter_key = "reporter_key";
        transaction_context.add_record();

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload =
            revoke_reporter_action(reporter_key, vec![REQUIRED_PROPERTY_NAME.to_string()]);

        match transaction_handler._revoke_reporter(&payload, &mut state, PUBLIC_KEY) {
            Ok(()) => panic!("Property does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!(
                    "Property does not exist: {}",
                    REQUIRED_PROPERTY_NAME
                )));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that when the RevokeReporterAction fails if the reporter is already
    /// unauthorized
    fn test_revoke_reporter_already_unauthorized() {
        let mut transaction_context = MockTransactionContext::default();
        let reporter_key = "reporter_key";
        transaction_context.add_record();
        transaction_context.add_property_with_reporter(
            REQUIRED_PROPERTY_NAME,
            reporter_key,
            false,
            required_property_definition(),
        );

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload =
            revoke_reporter_action(reporter_key, vec![REQUIRED_PROPERTY_NAME.to_string()]);

        match transaction_handler._revoke_reporter(&payload, &mut state, PUBLIC_KEY) {
            Ok(()) => {
                panic!("Reporter already unauthorized, InvalidTransaction should be returned")
            }
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains("Reporter is already unauthorized."));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that when the RevokeReporterAction fails if the key is not a valid reporter for the
    /// property
    fn test_revoke_reporter_not_valid() {
        let mut transaction_context = MockTransactionContext::default();
        let reporter_key = "reporter_key";
        transaction_context.add_record();
        transaction_context.add_property(REQUIRED_PROPERTY_NAME, required_property_definition());

        let mut state = TrackAndTraceState::new(&mut transaction_context);

        let transaction_handler = TrackAndTraceTransactionHandler::new();

        let payload =
            revoke_reporter_action(reporter_key, vec![REQUIRED_PROPERTY_NAME.to_string()]);

        match transaction_handler._revoke_reporter(&payload, &mut state, PUBLIC_KEY) {
            Ok(()) => {
                panic!("Reporter already unauthorized, InvalidTransaction should be returned")
            }
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!(
                    "{} not a reporter for property {}",
                    reporter_key, REQUIRED_PROPERTY_NAME
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

    fn create_proposal_no_props_action(
        role: Role,
        receiving_agent_key: &str,
    ) -> CreateProposalAction {
        CreateProposalActionBuilder::new()
            .with_record_id(RECORD_ID.to_string())
            .with_properties(vec![])
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
