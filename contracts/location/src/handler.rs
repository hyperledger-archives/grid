// Copyright 2020 Cargill Incorporated
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

use grid_sdk::{
    locations::addressing::GRID_NAMESPACE,
    permissions::PermissionChecker,
    protocol::location::{
        payload::{
            Action, LocationCreateAction, LocationDeleteAction, LocationNamespace, LocationPayload,
            LocationUpdateAction,
        },
        state::{LocationBuilder, LocationNamespace as StateNamespace},
    },
};

use grid_sdk::protos::FromBytes;

use crate::state::LocationState;

#[cfg(target_arch = "wasm32")]
fn apply(
    request: &TpProcessRequest,
    context: &mut dyn TransactionContext,
) -> Result<bool, ApplyError> {
    let handler = LocationTransactionHandler::new();
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

#[derive(Default)]
pub struct LocationTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

impl LocationTransactionHandler {
    pub fn new() -> Self {
        Self {
            family_name: "grid_location".to_string(),
            family_versions: vec!["1".to_string()],
            namespaces: vec![GRID_NAMESPACE.to_string()],
        }
    }
}

impl TransactionHandler for LocationTransactionHandler {
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
        let payload = LocationPayload::from_bytes(request.get_payload()).map_err(|err| {
            ApplyError::InvalidTransaction(format!("Cannot build location payload: {}", err))
        })?;

        info!(
            "Location Product Payload {:?} {}",
            payload.action(),
            payload.timestamp(),
        );

        let signer = request.get_header().get_signer_public_key();
        let mut state = LocationState::new(context);
        let perm_checker = PermissionChecker::new(context);

        match payload.action() {
            Action::LocationCreate(payload) => {
                create_location(&payload, &mut state, signer, &perm_checker)?
            }
            Action::LocationUpdate(payload) => {
                update_location(&payload, &mut state, signer, &perm_checker)?
            }
            Action::LocationDelete(payload) => {
                delete_location(&payload, &mut state, signer, &perm_checker)?
            }
        }
        Ok(())
    }
}

fn create_location(
    payload: &LocationCreateAction,
    state: &mut LocationState,
    signer: &str,
    perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    // 1) validate gln (gs1 only)
    if payload.namespace() == &LocationNamespace::GS1 && !is_gln_13_valid(&payload.location_id()) {
        return Err(ApplyError::InvalidTransaction(format!(
            "Invalid GLN: {}",
            payload.location_id()
        )));
    }

    // 2) check if location already exists
    if state.get_location(&payload.location_id())?.is_some() {
        return Err(ApplyError::InvalidTransaction(format!(
            "A location with GLN {} already exists",
            payload.location_id()
        )));
    }

    // 3) check if agent exists
    let agent = if let Some(agent) = state.get_agent(signer)? {
        agent
    } else {
        return Err(ApplyError::InvalidTransaction(format!(
            "Agent {} is not registered in Pike",
            signer
        )));
    };

    // 4) check if organization exists
    let organization = if let Some(org) = state.get_organization(payload.owner())? {
        org
    } else {
        return Err(ApplyError::InvalidTransaction(format!(
            "Organization {} is not registered with Pike",
            payload.owner()
        )));
    };

    // 5) check if agent belongs to organization
    if agent.org_id() != organization.org_id() {
        return Err(ApplyError::InvalidTransaction(format!(
            "Agent with public key {} is not registered to {}",
            agent.public_key(),
            organization.org_id()
        )));
    }

    // 6) check if agent has can_create_location permission
    if !perm_checker
        .has_permission(agent.public_key(), "can_create_location")
        .map_err(|err| ApplyError::InternalError(format!("Failed to check permissions: {}", err)))?
    {
        return Err(ApplyError::InvalidTransaction(format!(
            "Agent {} does not have permission to create locations",
            agent.public_key()
        )));
    }

    // 7) check if organization has gln in gs1_company_prefix metadata
    let mut has_gs1_prefix = false;
    for metadata in organization.metadata() {
        if metadata.key() == "gs1_company_prefixes"
            && payload.location_id().contains(metadata.value())
        {
            has_gs1_prefix = true;
        }
    }

    if !has_gs1_prefix {
        return Err(ApplyError::InvalidTransaction(format!(
            "Organization {} does not have the correct gs1 prefix",
            organization.org_id()
        )));
    }

    // 8) check if gs1 schema exists
    let schema = if let Some(schema) = state.get_schema("gs1_location")? {
        schema
    } else {
        return Err(ApplyError::InvalidTransaction(
            "gs1_location schema has not been defined".into(),
        ));
    };

    if payload.namespace() == &LocationNamespace::GS1 {
        // 9) Check if properties in location are all a part of the gs1 schema
        for property in payload.properties() {
            if schema
                .properties()
                .iter()
                .all(|p| p.name() != property.name())
            {
                return Err(ApplyError::InvalidTransaction(format!(
                    "{} is not a property that is defined by the gs1 schema",
                    property.name()
                )));
            }
        }

        // 10) check if location has all required fields
        for property in schema.properties().iter().filter(|p| *p.required()) {
            if !payload
                .properties()
                .iter()
                .any(|p| p.name() == property.name() && p.data_type() == property.data_type())
            {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Missing required field '{}' of type '{:?}'",
                    property.name(),
                    property.data_type()
                )));
            }
        }
    }

    let namespace = match payload.namespace() {
        LocationNamespace::GS1 => StateNamespace::GS1,
    };

    let location = LocationBuilder::new()
        .with_location_id(payload.location_id().to_string())
        .with_namespace(namespace)
        .with_owner(payload.owner().to_string())
        .with_properties(payload.properties().to_vec())
        .build()
        .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;

    state.set_location(location)
}

fn update_location(
    payload: &LocationUpdateAction,
    state: &mut LocationState,
    signer: &str,
    perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    // 1) check if location already exists
    let location = if let Some(location) = state.get_location(&payload.location_id())? {
        location
    } else {
        return Err(ApplyError::InvalidTransaction(format!(
            "A location with GLN {} does not exist",
            payload.location_id()
        )));
    };

    // 2) check if agent exists
    let agent = if let Some(agent) = state.get_agent(signer)? {
        agent
    } else {
        return Err(ApplyError::InvalidTransaction(format!(
            "Agent {} is not registered in Pike",
            signer
        )));
    };

    // 3) check if organization exists
    let organization = if let Some(org) = state.get_organization(location.owner())? {
        org
    } else {
        return Err(ApplyError::InvalidTransaction(format!(
            "Organization {} is not registered with Pike",
            location.owner()
        )));
    };

    // 4) check if agent belongs to organization
    if agent.org_id() != organization.org_id() {
        return Err(ApplyError::InvalidTransaction(format!(
            "Agent with public key {} is not registered to {}",
            agent.public_key(),
            organization.org_id()
        )));
    }

    // 5) check if agent has can_update_location permission
    if !perm_checker
        .has_permission(agent.public_key(), "can_update_location")
        .map_err(|err| ApplyError::InternalError(format!("Failed to check permissions: {}", err)))?
    {
        return Err(ApplyError::InvalidTransaction(format!(
            "Agent {} does not have permission to update locations",
            agent.public_key()
        )));
    }

    // 6) check if gs1 schema exists
    let schema = if let Some(schema) = state.get_schema("gs1_location")? {
        schema
    } else {
        return Err(ApplyError::InvalidTransaction(
            "gs1_location schema has not been defined".into(),
        ));
    };

    if payload.namespace() == &LocationNamespace::GS1 {
        // 7) Check if properties in location are all a part of the gs1 schema
        for property in payload.properties() {
            if schema
                .properties()
                .iter()
                .all(|p| p.name() != property.name())
            {
                return Err(ApplyError::InvalidTransaction(format!(
                    "{} is not a property that is defined by the gs1 schema",
                    property.name()
                )));
            }
        }

        // 8) check if location has all required fields
        for property in schema.properties().iter().filter(|p| *p.required()) {
            if !payload
                .properties()
                .iter()
                .any(|p| p.name() == property.name() && p.data_type() == property.data_type())
            {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Missing required field '{}' of type '{:?}'",
                    property.name(),
                    property.data_type()
                )));
            }
        }
    }

    let namespace = match payload.namespace() {
        LocationNamespace::GS1 => StateNamespace::GS1,
    };

    let location = LocationBuilder::new()
        .with_location_id(payload.location_id().to_string())
        .with_namespace(namespace)
        .with_owner(location.owner().to_string())
        .with_properties(payload.properties().to_vec())
        .build()
        .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;

    state.set_location(location)
}

fn delete_location(
    payload: &LocationDeleteAction,
    state: &mut LocationState,
    signer: &str,
    perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    // 1) check if location already exists
    let location = if let Some(location) = state.get_location(&payload.location_id())? {
        location
    } else {
        return Err(ApplyError::InvalidTransaction(format!(
            "A location with GLN {} does not exist",
            payload.location_id()
        )));
    };

    // 2) check if agent exists
    let agent = if let Some(agent) = state.get_agent(signer)? {
        agent
    } else {
        return Err(ApplyError::InvalidTransaction(format!(
            "Agent {} is not registered in Pike",
            signer
        )));
    };

    // 3) check if organization exists
    let organization = if let Some(org) = state.get_organization(location.owner())? {
        org
    } else {
        return Err(ApplyError::InvalidTransaction(format!(
            "Organization {} is not registered with Pike",
            location.owner()
        )));
    };

    // 4) check if agent belongs to organization
    if agent.org_id() != organization.org_id() {
        return Err(ApplyError::InvalidTransaction(format!(
            "Agent with public key {} is not registered to {}",
            agent.public_key(),
            organization.org_id()
        )));
    }

    // 5) check if agent has can_delete_location permission
    if !perm_checker
        .has_permission(agent.public_key(), "can_delete_location")
        .map_err(|err| ApplyError::InternalError(format!("Failed to check permissions: {}", err)))?
    {
        return Err(ApplyError::InvalidTransaction(format!(
            "Agent {} does not have permission to delete locations",
            agent.public_key()
        )));
    }

    state.remove_location(payload.location_id())
}

fn is_gln_13_valid(gln: &str) -> bool {
    if gln.len() != 13 {
        return false;
    }
    let mut acc = 0;
    let (start, check_digit_str) = gln.split_at(gln.len() - 1);
    for (i, c) in start.chars().enumerate() {
        if let Some(v) = c.to_digit(10) {
            if i % 2 == 0 {
                acc += v;
            } else {
                acc += v * 3;
            }
        } else {
            return false;
        }
    }

    let check_digit = if let Ok(c) = check_digit_str.parse::<u32>() {
        c
    } else {
        return false;
    };

    (check_digit + acc) % 10 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::collections::HashMap;

    use grid_sdk::{
        agents::addressing::compute_agent_address,
        organizations::addressing::compute_organization_address,
        protocol::{
            location::payload::{
                LocationCreateActionBuilder, LocationDeleteActionBuilder,
                LocationUpdateActionBuilder,
            },
            pike::state::{
                AgentBuilder, AgentListBuilder, KeyValueEntryBuilder, OrganizationBuilder,
                OrganizationListBuilder,
            },
            schema::state::{
                DataType, PropertyDefinitionBuilder, PropertyValueBuilder, SchemaBuilder,
                SchemaListBuilder,
            },
        },
        protos::IntoBytes,
        schemas::addressing::compute_schema_address,
    };

    use sawtooth_sdk::processor::handler::{ContextError, TransactionContext};

    #[derive(Default, Debug)]
    struct MockTransactionContext {
        state: RefCell<HashMap<String, Vec<u8>>>,
    }

    impl MockTransactionContext {
        fn new() -> Self {
            let mut entries = Vec::new();

            // create organization with prefix
            let key_value = KeyValueEntryBuilder::new()
                .with_key("gs1_company_prefixes".to_string())
                .with_value("9012".to_string())
                .build()
                .unwrap();
            let prefix_org = OrganizationBuilder::new()
                .with_org_id("prefix_org".to_string())
                .with_name("test_org_name".to_string())
                .with_address("test_org_address".to_string())
                .with_metadata(vec![key_value.clone()])
                .build()
                .unwrap();
            let prefix_org_list = OrganizationListBuilder::new()
                .with_organizations(vec![prefix_org.clone()])
                .build()
                .unwrap();
            let prefix_org_bytes = prefix_org_list.into_bytes().unwrap();
            let prefix_org_address = compute_organization_address("prefix_org");

            entries.push((prefix_org_address, prefix_org_bytes));

            // create organization without a prefix
            let no_prefix_org = OrganizationBuilder::new()
                .with_org_id("no_prefix_org".to_string())
                .with_name("test_org_name".to_string())
                .with_address("test_org_address".to_string())
                .build()
                .unwrap();
            let no_prefix_org_list = OrganizationListBuilder::new()
                .with_organizations(vec![no_prefix_org.clone()])
                .build()
                .unwrap();
            let no_prefix_org_bytes = no_prefix_org_list.into_bytes().unwrap();
            let no_prefix_org_address = compute_organization_address("no_prefix_org");

            entries.push((no_prefix_org_address, no_prefix_org_bytes));

            // create agent with correct permissions
            let agent_with_perms = AgentBuilder::new()
                .with_org_id("prefix_org".to_string())
                .with_public_key("agent_with_perms".to_string())
                .with_active(true)
                .with_roles(vec![
                    "can_delete_location".to_string(),
                    "can_create_location".to_string(),
                    "can_update_location".to_string(),
                ])
                .build()
                .unwrap();
            let agent_list_with_perms = AgentListBuilder::new()
                .with_agents(vec![agent_with_perms.clone()])
                .build()
                .unwrap();
            let agent_with_perms_bytes = agent_list_with_perms.into_bytes().unwrap();
            let agent_with_perms_address = compute_agent_address("agent_with_perms");

            entries.push((agent_with_perms_address, agent_with_perms_bytes));

            // create agent with no permissions for organization with prefix
            let agent_no_perms = AgentBuilder::new()
                .with_org_id("prefix_org".to_string())
                .with_public_key("agent_no_perms".to_string())
                .with_active(true)
                .build()
                .unwrap();
            let agent_list_with_perms = AgentListBuilder::new()
                .with_agents(vec![agent_no_perms.clone()])
                .build()
                .unwrap();
            let agent_no_perms_bytes = agent_list_with_perms.into_bytes().unwrap();
            let agent_no_perms_address = compute_agent_address("agent_no_perms");

            entries.push((agent_no_perms_address, agent_no_perms_bytes));

            // create agent with correct permissions for org with no prefix
            let agent_with_perms_no_prefix = AgentBuilder::new()
                .with_org_id("no_prefix_org".to_string())
                .with_public_key("agent_with_perms_no_prefix".to_string())
                .with_active(true)
                .with_roles(vec![
                    "can_delete_location".to_string(),
                    "can_create_location".to_string(),
                    "can_update_location".to_string(),
                ])
                .build()
                .unwrap();
            let agent_list_with_perms = AgentListBuilder::new()
                .with_agents(vec![agent_with_perms_no_prefix.clone()])
                .build()
                .unwrap();
            let agent_with_perms_no_prefix_bytes = agent_list_with_perms.into_bytes().unwrap();
            let agent_with_perms_no_prefix_address =
                compute_agent_address("agent_with_perms_no_prefix");

            entries.push((
                agent_with_perms_no_prefix_address,
                agent_with_perms_no_prefix_bytes,
            ));

            let mock = MockTransactionContext::default();
            mock.set_state_entries(entries).unwrap();

            mock
        }

        fn create_gs1_schema(&self) {
            let properties = vec![
                PropertyDefinitionBuilder::new()
                    .with_name("locationName".into())
                    .with_data_type(DataType::String)
                    .with_required(true)
                    .build()
                    .unwrap(),
                PropertyDefinitionBuilder::new()
                    .with_name("description".into())
                    .with_data_type(DataType::String)
                    .with_required(true)
                    .build()
                    .unwrap(),
            ];

            let schema = SchemaBuilder::new()
                .with_name("gs1_location".into())
                .with_description("GS1 Location".into())
                .with_owner("prefix_org".into())
                .with_properties(properties)
                .build()
                .unwrap();

            let schema_list = SchemaListBuilder::new()
                .with_schemas(vec![schema])
                .build()
                .unwrap();

            self.set_state_entries(vec![(
                compute_schema_address("gs1_location"),
                schema_list.into_bytes().unwrap(),
            )])
            .unwrap();
        }
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
        fn delete_state_entries(&self, addresses: &[String]) -> Result<Vec<String>, ContextError> {
            for addr in addresses {
                self.state.borrow_mut().remove(addr);
            }

            Ok(addresses.to_vec())
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

    fn create_default_location(state: &mut LocationState, perm_checker: &PermissionChecker) {
        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::String)
                .with_string_value("An alley filled with tacos".into())
                .build()
                .unwrap(),
        ];

        let location = LocationCreateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_owner("prefix_org".into())
            .with_properties(properties)
            .build()
            .unwrap();

        let result = create_location(&location, state, "agent_with_perms", &perm_checker);

        assert!(result.is_ok());
    }

    #[test]
    fn test_add_location_valid() {
        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::String)
                .with_string_value("An alley filled with tacos".into())
                .build()
                .unwrap(),
        ];

        let payload = LocationCreateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_owner("prefix_org".into())
            .with_properties(properties)
            .build()
            .unwrap();

        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        let result = create_location(&payload, &mut state, "agent_with_perms", &perm_checker);

        assert!(result.is_ok());
    }

    #[test]
    fn test_update_location_valid() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        create_default_location(&mut state, &perm_checker);

        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::String)
                .with_string_value("An alley filled with tacos".into())
                .build()
                .unwrap(),
        ];

        let payload = LocationUpdateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_properties(properties)
            .build()
            .unwrap();

        let result = update_location(&payload, &mut state, "agent_with_perms", &perm_checker);

        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_location_valid() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        create_default_location(&mut state, &perm_checker);

        let payload = LocationDeleteActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .build()
            .unwrap();

        let result = delete_location(&payload, &mut state, "agent_with_perms", &perm_checker);

        assert!(result.is_ok());
    }

    #[test]
    fn test_create_location_invalid_gln() {
        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::String)
                .with_string_value("An alley filled with tacos".into())
                .build()
                .unwrap(),
        ];

        let payload = LocationCreateActionBuilder::new()
            .with_location_id("12345".into())
            .with_namespace(LocationNamespace::GS1)
            .with_owner("prefix_org".into())
            .with_properties(properties)
            .build()
            .unwrap();

        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        match create_location(&payload, &mut state, "agent_with_perms", &perm_checker) {
            Ok(()) => panic!("Failed to find invalid GLN"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!("Invalid GLN: 12345", msg);
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_create_location_already_exists() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        create_default_location(&mut state, &perm_checker);

        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::String)
                .with_string_value("An alley filled with tacos".into())
                .build()
                .unwrap(),
        ];

        let payload = LocationCreateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_owner("prefix_org".into())
            .with_properties(properties)
            .build()
            .unwrap();

        match create_location(&payload, &mut state, "agent_with_perms", &perm_checker) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!("A location with GLN 9012345000004 already exists", msg);
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_create_location_agent_does_not_exist() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::String)
                .with_string_value("An alley filled with tacos".into())
                .build()
                .unwrap(),
        ];

        let payload = LocationCreateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_owner("prefix_org".into())
            .with_properties(properties)
            .build()
            .unwrap();

        match create_location(&payload, &mut state, "no_agent", &perm_checker) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!("Agent no_agent is not registered in Pike", msg);
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_create_location_org_does_not_exist() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::String)
                .with_string_value("An alley filled with tacos".into())
                .build()
                .unwrap(),
        ];

        let payload = LocationCreateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_owner("fake_org".into())
            .with_properties(properties)
            .build()
            .unwrap();

        match create_location(&payload, &mut state, "agent_with_perms", &perm_checker) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!("Organization fake_org is not registered with Pike", msg);
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_create_location_wrong_agent_for_org() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::String)
                .with_string_value("An alley filled with tacos".into())
                .build()
                .unwrap(),
        ];

        let payload = LocationCreateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_owner("prefix_org".into())
            .with_properties(properties)
            .build()
            .unwrap();

        match create_location(
            &payload,
            &mut state,
            "agent_with_perms_no_prefix",
            &perm_checker,
        ) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!(
                    "Agent with public key agent_with_perms_no_prefix is not registered to prefix_org",
                    msg);
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_create_location_agent_does_not_have_create_perms() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::String)
                .with_string_value("An alley filled with tacos".into())
                .build()
                .unwrap(),
        ];

        let payload = LocationCreateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_owner("prefix_org".into())
            .with_properties(properties)
            .build()
            .unwrap();

        match create_location(&payload, &mut state, "agent_no_perms", &perm_checker) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!(
                    "Agent agent_no_perms does not have permission to create locations",
                    msg
                );
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_create_location_org_does_not_have_gs1_prefix() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::String)
                .with_string_value("An alley filled with tacos".into())
                .build()
                .unwrap(),
        ];

        let payload = LocationCreateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_owner("no_prefix_org".into())
            .with_properties(properties)
            .build()
            .unwrap();

        match create_location(
            &payload,
            &mut state,
            "agent_with_perms_no_prefix",
            &perm_checker,
        ) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!(
                    "Organization no_prefix_org does not have the correct gs1 prefix",
                    msg
                );
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_create_location_gs1_schema_does_not_exist() {
        let mock_context = MockTransactionContext::new();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::String)
                .with_string_value("An alley filled with tacos".into())
                .build()
                .unwrap(),
        ];

        let payload = LocationCreateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_owner("prefix_org".into())
            .with_properties(properties)
            .build()
            .unwrap();

        match create_location(&payload, &mut state, "agent_with_perms", &perm_checker) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!("gs1_location schema has not been defined", msg);
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_create_location_undefined_property() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::String)
                .with_string_value("An alley filled with tacos".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("mvp".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Cat".into())
                .build()
                .unwrap(),
        ];

        let payload = LocationCreateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_owner("prefix_org".into())
            .with_properties(properties)
            .build()
            .unwrap();

        match create_location(&payload, &mut state, "agent_with_perms", &perm_checker) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!(
                    "mvp is not a property that is defined by the gs1 schema",
                    msg
                );
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_create_location_missing_required_property() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        let properties = vec![PropertyValueBuilder::new()
            .with_name("locationName".into())
            .with_data_type(DataType::String)
            .with_string_value("Taco Alley".into())
            .build()
            .unwrap()];

        let payload = LocationCreateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_owner("prefix_org".into())
            .with_properties(properties)
            .build()
            .unwrap();

        match create_location(&payload, &mut state, "agent_with_perms", &perm_checker) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!("Missing required field 'description' of type 'String'", msg);
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_create_location_required_property_has_wrong_type() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::Number)
                .with_number_value(745)
                .build()
                .unwrap(),
        ];

        let payload = LocationCreateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_owner("prefix_org".into())
            .with_properties(properties)
            .build()
            .unwrap();

        match create_location(&payload, &mut state, "agent_with_perms", &perm_checker) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!("Missing required field 'description' of type 'String'", msg);
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_update_location_does_not_exist() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::Number)
                .with_number_value(745)
                .build()
                .unwrap(),
        ];

        let payload = LocationUpdateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_properties(properties)
            .build()
            .unwrap();

        match update_location(&payload, &mut state, "agent_with_perms", &perm_checker) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!("A location with GLN 9012345000004 does not exist", msg);
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_update_location_agent_does_not_have_update_perms() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        create_default_location(&mut state, &perm_checker);

        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::Number)
                .with_number_value(745)
                .build()
                .unwrap(),
        ];

        let payload = LocationUpdateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_properties(properties)
            .build()
            .unwrap();

        match update_location(&payload, &mut state, "agent_no_perms", &perm_checker) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!(
                    "Agent agent_no_perms does not have permission to update locations",
                    msg
                );
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_update_location_agent_does_not_exist() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        create_default_location(&mut state, &perm_checker);

        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::String)
                .with_string_value("An alley filled with tacos".into())
                .build()
                .unwrap(),
        ];

        let payload = LocationUpdateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_properties(properties)
            .build()
            .unwrap();

        match update_location(&payload, &mut state, "no_agent", &perm_checker) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!("Agent no_agent is not registered in Pike", msg);
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_update_location_wrong_agent_for_org() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        create_default_location(&mut state, &perm_checker);

        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::String)
                .with_string_value("An alley filled with tacos".into())
                .build()
                .unwrap(),
        ];

        let payload = LocationUpdateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_properties(properties)
            .build()
            .unwrap();

        match update_location(
            &payload,
            &mut state,
            "agent_with_perms_no_prefix",
            &perm_checker,
        ) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!(
                    "Agent with public key agent_with_perms_no_prefix is not registered to prefix_org",
                    msg);
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_update_location_undefined_property() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        create_default_location(&mut state, &perm_checker);

        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::String)
                .with_string_value("An alley filled with tacos".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("mvp".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Cat".into())
                .build()
                .unwrap(),
        ];

        let payload = LocationUpdateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_properties(properties)
            .build()
            .unwrap();

        match update_location(&payload, &mut state, "agent_with_perms", &perm_checker) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!(
                    "mvp is not a property that is defined by the gs1 schema",
                    msg
                );
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_update_location_missing_required_property() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        create_default_location(&mut state, &perm_checker);

        let properties = vec![PropertyValueBuilder::new()
            .with_name("locationName".into())
            .with_data_type(DataType::String)
            .with_string_value("Taco Alley".into())
            .build()
            .unwrap()];

        let payload = LocationUpdateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_properties(properties)
            .build()
            .unwrap();

        match update_location(&payload, &mut state, "agent_with_perms", &perm_checker) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!("Missing required field 'description' of type 'String'", msg);
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_update_location_required_property_has_wrong_type() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        create_default_location(&mut state, &perm_checker);

        let properties = vec![
            PropertyValueBuilder::new()
                .with_name("locationName".into())
                .with_data_type(DataType::String)
                .with_string_value("Taco Alley".into())
                .build()
                .unwrap(),
            PropertyValueBuilder::new()
                .with_name("description".into())
                .with_data_type(DataType::Number)
                .with_number_value(745)
                .build()
                .unwrap(),
        ];

        let payload = LocationUpdateActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .with_properties(properties)
            .build()
            .unwrap();

        match update_location(&payload, &mut state, "agent_with_perms", &perm_checker) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!("Missing required field 'description' of type 'String'", msg);
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_delete_location_does_not_exist() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        let payload = LocationDeleteActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .build()
            .unwrap();

        match delete_location(&payload, &mut state, "agent_with_perms", &perm_checker) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!("A location with GLN 9012345000004 does not exist", msg);
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_delete_location_agent_does_not_have_delete_perms() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        create_default_location(&mut state, &perm_checker);

        let payload = LocationDeleteActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .build()
            .unwrap();

        match delete_location(&payload, &mut state, "agent_no_perms", &perm_checker) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!(
                    "Agent agent_no_perms does not have permission to delete locations",
                    msg
                );
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_delete_location_agent_does_not_exist() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        create_default_location(&mut state, &perm_checker);

        let payload = LocationDeleteActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .build()
            .unwrap();

        match delete_location(&payload, &mut state, "no_agent", &perm_checker) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!("Agent no_agent is not registered in Pike", msg);
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }

    #[test]
    fn test_delete_location_wrong_agent_for_org() {
        let mock_context = MockTransactionContext::new();
        mock_context.create_gs1_schema();

        let perm_checker = PermissionChecker::new(&mock_context);
        let mut state = LocationState::new(&mock_context);

        create_default_location(&mut state, &perm_checker);

        let payload = LocationDeleteActionBuilder::new()
            .with_location_id("9012345000004".into())
            .with_namespace(LocationNamespace::GS1)
            .build()
            .unwrap();

        match delete_location(
            &payload,
            &mut state,
            "agent_with_perms_no_prefix",
            &perm_checker,
        ) {
            Ok(()) => panic!("Unexpected positive result"),
            Err(ApplyError::InvalidTransaction(ref msg)) => {
                assert_eq!(
                    "Agent with public key agent_with_perms_no_prefix is not registered to prefix_org",
                    msg);
            }
            Err(err) => panic!("Wrong error: {}", err),
        }
    }
}
