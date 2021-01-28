// Copyright 2019 Cargill Incorporated
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

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::ApplyError;
        use sabre_sdk::TransactionContext;
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
        use sawtooth_sdk::processor::handler::TransactionContext;
    }
}

use grid_sdk::{
    agents::addressing::compute_agent_address,
    protocol::{
        pike::state::{Agent, AgentList},
        schema::state::{Schema, SchemaList},
        track_and_trace::state::{
            Property, PropertyList, PropertyListBuilder, PropertyPage, PropertyPageList,
            PropertyPageListBuilder, ProposalList, Record, RecordList, RecordListBuilder,
        },
    },
    protos::{FromBytes, IntoBytes},
    schemas::addressing::compute_schema_address,
    track_and_trace::addressing::*,
};

pub struct TrackAndTraceState<'a> {
    context: &'a mut dyn TransactionContext,
}

impl<'a> TrackAndTraceState<'a> {
    pub fn new(context: &'a mut dyn TransactionContext) -> TrackAndTraceState {
        TrackAndTraceState { context }
    }

    pub fn get_record(&self, record_id: &str) -> Result<Option<Record>, ApplyError> {
        let address = make_record_address(record_id);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let records = match RecordList::from_bytes(packed.as_slice()) {
                    Ok(records) => records,
                    Err(err) => {
                        return Err(ApplyError::InternalError(format!(
                            "Cannot deserialize record list: {:?}",
                            err,
                        )));
                    }
                };

                // find the record with the correct id
                for record in records.records() {
                    if record.record_id() == record_id {
                        return Ok(Some(record.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_record(&self, record_id: &str, record: Record) -> Result<(), ApplyError> {
        let address = make_record_address(record_id);
        let d = self.context.get_state_entry(&address)?;
        let mut records = match d {
            Some(packed) => match RecordList::from_bytes(packed.as_slice()) {
                Ok(record_list) => record_list.records().to_vec(),
                Err(err) => {
                    return Err(ApplyError::InternalError(format!(
                        "Cannot deserialize record list: {:?}",
                        err
                    )));
                }
            },
            None => vec![],
        };

        let mut index = None;
        for (i, record) in records.iter().enumerate() {
            if record.record_id() == record_id {
                index = Some(i);
                break;
            }
        }

        if let Some(i) = index {
            records.remove(i);
        }
        records.push(record);
        records.sort_by_key(|r| r.record_id().to_string());
        let record_list = RecordListBuilder::new()
            .with_records(records)
            .build()
            .map_err(|err| {
                ApplyError::InvalidTransaction(format!("Cannot build record list: {:?}", err))
            })?;

        let serialized = match record_list.into_bytes() {
            Ok(serialized) => serialized,
            Err(err) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Cannot serialize record list: {:?}",
                    err
                )));
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn get_schema(&self, schema_name: &str) -> Result<Option<Schema>, ApplyError> {
        let address = compute_schema_address(schema_name);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let schemas = match SchemaList::from_bytes(packed.as_slice()) {
                    Ok(schemas) => schemas,
                    Err(err) => {
                        return Err(ApplyError::InternalError(format!(
                            "Cannot deserialize schema list: {:?}",
                            err,
                        )));
                    }
                };

                // find the schema with the correct name
                for schema in schemas.schemas() {
                    if schema.name() == schema_name {
                        return Ok(Some(schema.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    /// Gets a Pike Agent. Handles retrieving the correct agent from an AgentList.
    pub fn get_agent(&self, public_key: &str) -> Result<Option<Agent>, ApplyError> {
        let address = compute_agent_address(public_key);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let agents = match AgentList::from_bytes(packed.as_slice()) {
                    Ok(agents) => agents,
                    Err(err) => {
                        return Err(ApplyError::InternalError(format!(
                            "Cannot deserialize agent list: {:?}",
                            err,
                        )));
                    }
                };

                // find the agent with the correct public_key
                for agent in agents.agents() {
                    if agent.public_key() == public_key {
                        return Ok(Some(agent.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn get_property(
        &self,
        record_id: &str,
        property_name: &str,
    ) -> Result<Option<Property>, ApplyError> {
        let address = make_property_address(record_id, property_name, 0);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let properties = match PropertyList::from_bytes(packed.as_slice()) {
                    Ok(properties) => properties,
                    Err(err) => {
                        return Err(ApplyError::InternalError(format!(
                            "Cannot deserialize property list: {:?}",
                            err,
                        )));
                    }
                };

                // find the property with the correct name
                for property in properties.properties() {
                    if property.name() == property_name {
                        return Ok(Some(property.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_property(
        &self,
        record_id: &str,
        property_name: &str,
        property: Property,
    ) -> Result<(), ApplyError> {
        let address = make_property_address(record_id, property_name, 0);
        let d = self.context.get_state_entry(&address)?;
        let mut properties = match d {
            Some(packed) => match PropertyList::from_bytes(packed.as_slice()) {
                Ok(property_list) => property_list.properties().to_vec(),
                Err(err) => {
                    return Err(ApplyError::InternalError(format!(
                        "Cannot deserialize property list: {:?}",
                        err
                    )));
                }
            },
            None => vec![],
        };

        // remove old property if it exists and sort the properties by name
        let mut index = None;
        for (i, prop) in properties.iter().enumerate() {
            if prop.name() == property_name {
                index = Some(i);
                break;
            }
        }

        if let Some(i) = index {
            properties.remove(i);
        }
        properties.push(property);
        properties.sort_by_key(|p| p.name().to_string());

        // build new PropertyList and set in state
        let property_list = PropertyListBuilder::new()
            .with_properties(properties)
            .build()
            .map_err(|err| {
                ApplyError::InvalidTransaction(format!("Cannot build property list: {:?}", err,))
            })?;

        let serialized = match property_list.into_bytes() {
            Ok(serialized) => serialized,
            Err(err) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Cannot serialize property list: {:?}",
                    err
                )));
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn get_property_page(
        &self,
        record_id: &str,
        property_name: &str,
        page: u32,
    ) -> Result<Option<PropertyPage>, ApplyError> {
        let address = make_property_address(record_id, property_name, page);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let property_pages = match PropertyPageList::from_bytes(packed.as_slice()) {
                    Ok(property_pages) => property_pages,
                    Err(err) => {
                        return Err(ApplyError::InternalError(format!(
                            "Cannot deserialize property page list: {:?}",
                            err,
                        )));
                    }
                };

                // find the property with the correct name
                for property_page in property_pages.property_pages() {
                    if property_page.name() == property_name {
                        return Ok(Some(property_page.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_property_page(
        &self,
        record_id: &str,
        property_name: &str,
        page_num: u32,
        property_page: PropertyPage,
    ) -> Result<(), ApplyError> {
        let address = make_property_address(record_id, property_name, page_num);
        let d = self.context.get_state_entry(&address)?;
        let mut pages = match d {
            Some(packed) => match PropertyPageList::from_bytes(packed.as_slice()) {
                Ok(property_page_list) => property_page_list.property_pages().to_vec(),
                Err(err) => {
                    return Err(ApplyError::InternalError(format!(
                        "Cannot deserialize property page list: {:?}",
                        err
                    )));
                }
            },
            None => vec![],
        };

        // remove old property page if it exists and sort the property pages by name
        let mut index = None;
        for (i, page) in pages.iter().enumerate() {
            if page.name() == property_name {
                index = Some(i);
                break;
            }
        }

        if let Some(i) = index {
            pages.remove(i);
        }
        pages.push(property_page);
        pages.sort_by_key(|pp| pp.name().to_string());

        // build new PropertyPageList and set in state
        let property_page_list = PropertyPageListBuilder::new()
            .with_property_pages(pages)
            .build()
            .map_err(|err| {
                ApplyError::InvalidTransaction(format!(
                    "Cannot build property page list: {:?}",
                    err
                ))
            })?;

        let serialized = match property_page_list.into_bytes() {
            Ok(serialized) => serialized,
            Err(err) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Cannot serialize property page list: {:?}",
                    err
                )));
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn get_proposal_list(
        &self,
        record_id: &str,
        agent_id: &str,
    ) -> Result<Option<ProposalList>, ApplyError> {
        let address = make_proposal_address(record_id, agent_id);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => match ProposalList::from_bytes(packed.as_slice()) {
                Ok(proposal_list) => Ok(Some(proposal_list)),
                Err(err) => Err(ApplyError::InternalError(format!(
                    "Cannot deserialize proposal list: {:?}",
                    err,
                ))),
            },
            None => Ok(None),
        }
    }

    pub fn set_proposal_list(
        &self,
        record_id: &str,
        agent_id: &str,
        proposals: ProposalList,
    ) -> Result<(), ApplyError> {
        let address = make_proposal_address(record_id, agent_id);
        let serialized = match proposals.into_bytes() {
            Ok(serialized) => serialized,
            Err(err) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Cannot serialize proposal list: {:?}",
                    err
                )));
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::collections::HashMap;

    use grid_sdk::protocol::pike::state::{AgentBuilder, AgentListBuilder};
    use grid_sdk::protocol::schema::state::{
        DataType, PropertyDefinition, PropertyDefinitionBuilder, PropertyValue,
        PropertyValueBuilder,
    };

    use grid_sdk::protocol::track_and_trace::state::{
        AssociatedAgentBuilder, PropertyBuilder, PropertyPageBuilder, ProposalBuilder,
        ProposalListBuilder, RecordBuilder, ReportedValueBuilder, ReporterBuilder, Role, Status,
    };

    use sawtooth_sdk::processor::handler::{ContextError, TransactionContext};

    const RECORD_ID: &str = "test_record";
    const PROPERTY_NAME: &str = "test_property_name";

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
            let agent_list = AgentListBuilder::new()
                .with_agents(vec![make_agent(public_key)])
                .build()
                .unwrap();
            let agent_bytes = agent_list.into_bytes().unwrap();
            let agent_address = compute_agent_address(public_key);
            self.set_state_entry(agent_address, agent_bytes).unwrap();
        }
    }

    #[test]
    // Test that if an agent does not exist in state, None is returned
    fn test_get_agent_none() {
        let mut transaction_context = MockTransactionContext::default();
        let state = TrackAndTraceState::new(&mut transaction_context);

        let result = state.get_agent("agent_public_key").unwrap();
        assert!(result.is_none())
    }

    #[test]
    // Test that if an agent exist in state, Some(agent) is returned
    fn test_get_agent_some() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_agent("agent_public_key");
        let state = TrackAndTraceState::new(&mut transaction_context);
        let result = state.get_agent("agent_public_key").unwrap();
        assert_eq!(result, Some(make_agent("agent_public_key")))
    }

    #[test]
    // Test that if a record does not exist in state, None is returned
    fn test_get_record_none() {
        let mut transaction_context = MockTransactionContext::default();
        let state = TrackAndTraceState::new(&mut transaction_context);

        let result = state.get_record("not_an_record").unwrap();
        assert!(result.is_none())
    }

    #[test]
    // Test that if an schema does not exist in state, None is returned
    fn test_get_schema_none() {
        let mut transaction_context = MockTransactionContext::default();
        let state = TrackAndTraceState::new(&mut transaction_context);

        let result = state.get_schema("not_an_schema").unwrap();
        assert!(result.is_none())
    }

    #[test]
    // Test that if an property does not exist in state, None is returned
    fn test_get_property_none() {
        let mut transaction_context = MockTransactionContext::default();
        let state = TrackAndTraceState::new(&mut transaction_context);

        let result = state.get_property("record_id", "not_a_property").unwrap();
        assert!(result.is_none())
    }

    #[test]
    // Test that if an property page does not exist in state, None is returned
    fn test_get_property_page_none() {
        let mut transaction_context = MockTransactionContext::default();
        let state = TrackAndTraceState::new(&mut transaction_context);

        let result = state
            .get_property_page("record_id", "property_name", 1)
            .unwrap();
        assert!(result.is_none())
    }

    #[test]
    // Test that a record can be added to state
    fn test_set_record() {
        let mut transaction_context = MockTransactionContext::default();
        let state = TrackAndTraceState::new(&mut transaction_context);

        assert!(state.set_record(RECORD_ID, make_record()).is_ok());
        let result = state.get_record(RECORD_ID).unwrap();
        assert_eq!(result, Some(make_record()));
    }

    #[test]
    // Test that an property can be added to state
    fn test_set_property() {
        let mut transaction_context = MockTransactionContext::default();
        let state = TrackAndTraceState::new(&mut transaction_context);

        assert!(state
            .set_property(RECORD_ID, PROPERTY_NAME, make_property())
            .is_ok());
        let result = state.get_property(RECORD_ID, PROPERTY_NAME).unwrap();
        assert_eq!(result, Some(make_property()));
    }

    #[test]
    // Test that an property page can be added to state
    fn test_set_property_page() {
        let mut transaction_context = MockTransactionContext::default();
        let state = TrackAndTraceState::new(&mut transaction_context);
        assert!(state
            .set_property_page(RECORD_ID, PROPERTY_NAME, 1, make_property_page())
            .is_ok());
        let result = state
            .get_property_page(RECORD_ID, PROPERTY_NAME, 1)
            .unwrap();
        assert_eq!(result, Some(make_property_page()));
    }

    #[test]
    // Test that an property page can be added to state
    fn test_set_proposal_list() {
        let mut transaction_context = MockTransactionContext::default();
        let state = TrackAndTraceState::new(&mut transaction_context);
        assert!(state
            .set_proposal_list(RECORD_ID, "agent_key", make_proposal_list())
            .is_ok());
        let result = state.get_proposal_list(RECORD_ID, "agent_key").unwrap();
        assert_eq!(result, Some(make_proposal_list()));
    }

    fn make_property_value() -> PropertyValue {
        PropertyValueBuilder::new()
            .with_name(PROPERTY_NAME.to_string())
            .with_data_type(DataType::String)
            .with_string_value("required_field".to_string())
            .build()
            .expect("Failed to build property value")
    }

    fn make_property_definition() -> PropertyDefinition {
        PropertyDefinitionBuilder::new()
            .with_name(PROPERTY_NAME.to_string())
            .with_data_type(DataType::String)
            .with_description("Required".to_string())
            .with_required(true)
            .build()
            .expect("Failed to build property definition")
    }

    fn make_record() -> Record {
        let associated_agent = AssociatedAgentBuilder::new()
            .with_agent_id("agent_key".to_string())
            .with_timestamp(1)
            .build()
            .expect("Failed to build AssociatedAgent");

        RecordBuilder::new()
            .with_record_id(RECORD_ID.to_string())
            .with_schema("schema_name".to_string())
            .with_owners(vec![associated_agent.clone()])
            .with_custodians(vec![associated_agent.clone()])
            .with_field_final(false)
            .build()
            .expect("Failed to build new_record")
    }

    fn make_property() -> Property {
        let reporter = ReporterBuilder::new()
            .with_public_key("agent_key".to_string())
            .with_authorized(true)
            .with_index(0)
            .build()
            .expect("Failed to build Reporter");

        PropertyBuilder::new()
            .with_name(PROPERTY_NAME.to_string())
            .with_record_id(RECORD_ID.to_string())
            .with_property_definition(make_property_definition())
            .with_reporters(vec![reporter.clone()])
            .with_current_page(1)
            .with_wrapped(false)
            .build()
            .expect("Failed to build property")
    }

    fn make_property_page() -> PropertyPage {
        let reported_value = ReportedValueBuilder::new()
            .with_reporter_index(0)
            .with_timestamp(1)
            .with_value(make_property_value())
            .build()
            .expect("Failed to build ReportedValue");

        PropertyPageBuilder::new()
            .with_name(PROPERTY_NAME.to_string())
            .with_record_id(RECORD_ID.to_string())
            .with_reported_values(vec![reported_value])
            .build()
            .expect("Failed to build PropertyPage")
    }

    fn make_proposal_list() -> ProposalList {
        let proposal = ProposalBuilder::new()
            .with_record_id(RECORD_ID.to_string())
            .with_timestamp(1)
            .with_issuing_agent("issuing_agent".to_string())
            .with_receiving_agent("receiving_agent_key".to_string())
            .with_role(Role::Owner)
            .with_properties(vec![PROPERTY_NAME.to_string()])
            .with_status(Status::Open)
            .with_terms("empty string NEED TO CHANGE".to_string())
            .build()
            .expect("Failed to build proposal");

        ProposalListBuilder::new()
            .with_proposals(vec![proposal])
            .build()
            .expect("Failed to build proposal list")
    }

    fn make_agent(public_key: &str) -> Agent {
        AgentBuilder::new()
            .with_org_id("test_org".to_string())
            .with_public_key(public_key.to_string())
            .with_active(true)
            .with_roles(vec![])
            .build()
            .expect("Failed to build agent")
    }
}
