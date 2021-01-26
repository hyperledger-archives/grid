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

use grid_sdk::permissions::PermissionChecker;
use grid_sdk::protocol::schema::payload::{
    Action, SchemaCreateAction, SchemaPayload, SchemaUpdateAction,
};
use grid_sdk::protocol::schema::state::SchemaBuilder;
use grid_sdk::protos::FromBytes;

use crate::payload::validate_payload;
use crate::permissions::{permission_to_perm_string, Permission};
use crate::state::GridSchemaState;

pub const GRID_NAMESPACE: &str = "621dee";
pub const PIKE_NAMESPACE: &str = "621dee05";

#[cfg(target_arch = "wasm32")]
// Sabre apply must return a bool
fn apply(
    request: &TpProcessRequest,
    context: &mut dyn TransactionContext,
) -> Result<bool, ApplyError> {
    let handler = GridSchemaTransactionHandler::new();
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

pub struct GridSchemaTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

impl GridSchemaTransactionHandler {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        GridSchemaTransactionHandler {
            family_name: "grid_schema".to_string(),
            family_versions: vec!["1".to_string()],
            namespaces: vec![GRID_NAMESPACE.to_string()],
        }
    }
}

impl TransactionHandler for GridSchemaTransactionHandler {
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
        let payload = SchemaPayload::from_bytes(request.get_payload()).map_err(|err| {
            ApplyError::InvalidTransaction(format!("Cannot build schema payload: {}", err))
        })?;

        validate_payload(&payload)?;

        let signer = request.get_header().get_signer_public_key();
        let state = GridSchemaState::new(context);
        let perm_checker = PermissionChecker::new(context);

        info!("Grid Schema Payload {:?}", payload.action(),);

        match payload.action() {
            Action::SchemaCreate(schema_create_payload) => {
                schema_create(schema_create_payload, signer, &state, &perm_checker)
            }
            Action::SchemaUpdate(schema_update_payload) => {
                schema_update(schema_update_payload, signer, &state, &perm_checker)
            }
        }
    }
}

fn schema_create(
    payload: &SchemaCreateAction,
    signer: &str,
    state: &GridSchemaState,
    perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    let schema_name = payload.schema_name();
    let description = payload.description();
    let properties = payload.properties();

    if state.get_schema(schema_name)?.is_some() {
        return Err(ApplyError::InvalidTransaction(format!(
            "Schema with name {} already exists",
            schema_name
        )));
    }

    let permitted = perm_checker
        .has_permission(
            signer,
            &permission_to_perm_string(Permission::CanCreateSchema),
            payload.owner(),
        )
        .map_err(|err| {
            ApplyError::InternalError(format!("Failed to check permissions: {}", err))
        })?;

    if !permitted {
        return Err(ApplyError::InternalError(format!(
            "Agent {} does not have the correct permissions",
            &signer
        )));
    }

    let schema = SchemaBuilder::new()
        .with_name(schema_name.into())
        .with_description(description.into())
        .with_owner(payload.owner().into())
        .with_properties(properties.to_vec())
        .build()
        .map_err(|err| ApplyError::InvalidTransaction(format!("Cannot build schema: {}", err)))?;

    state.set_schema(schema_name, schema)
}

fn schema_update(
    payload: &SchemaUpdateAction,
    signer: &str,
    state: &GridSchemaState,
    perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    let schema_name = payload.schema_name();
    let mut new_properties = payload.properties().to_vec();

    let schema = match state.get_schema(schema_name)? {
        Some(schema) => schema,
        None => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Schema with name {} does not exist",
                schema_name
            )));
        }
    };

    let permitted = perm_checker
        .has_permission(
            signer,
            &permission_to_perm_string(Permission::CanUpdateSchema),
            schema.owner(),
        )
        .map_err(|err| {
            ApplyError::InternalError(format!("Failed to check permissions: {}", err))
        })?;

    if !permitted {
        return Err(ApplyError::InternalError(format!(
            "Agent {} does not have the correct permissions",
            &signer
        )));
    }

    let mut properties = schema.properties().to_vec();
    properties.sort_by_key(|p| p.name().to_string());

    for property in new_properties.iter() {
        if properties
            .binary_search_by_key(&property.name().to_string(), |p| p.name().to_string())
            .is_ok()
        {
            return Err(ApplyError::InvalidTransaction(format!(
                "Schema already has PropertyDefination with name {}",
                property.name()
            )));
        }
    }
    properties.append(&mut new_properties);

    let schema = SchemaBuilder::new()
        .with_name(schema.name().into())
        .with_description(schema.description().into())
        .with_owner(schema.owner().into())
        .with_properties(properties)
        .build()
        .map_err(|err| ApplyError::InvalidTransaction(format!("Cannot build schema: {}", err)))?;

    state.set_schema(schema_name, schema)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::collections::HashMap;

    use crypto::digest::Digest;
    use crypto::sha2::Sha512;

    use grid_sdk::protocol::pike::state::{
        AgentBuilder, AgentListBuilder, RoleBuilder, RoleListBuilder,
    };
    use grid_sdk::protocol::schema::payload::{SchemaCreateBuilder, SchemaUpdateBuilder};
    use grid_sdk::protocol::schema::state::{
        DataType, PropertyDefinitionBuilder, SchemaBuilder, SchemaListBuilder,
    };
    use grid_sdk::protos::IntoBytes;
    use sawtooth_sdk::processor::handler::ApplyError;
    use sawtooth_sdk::processor::handler::{ContextError, TransactionContext};

    use crate::state::compute_schema_address;

    const PIKE_AGENT_NAMESPACE: &str = "00";
    const PIKE_ROLE_NAMESPACE: &str = "02";

    #[derive(Default)]
    /// A MockTransactionContext that can be used to test GridSchemaState
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
        fn add_role(&self) {
            let builder = RoleBuilder::new();
            let role = builder
                .with_org_id("test_org".to_string())
                .with_name("test_org.schema".to_string())
                .with_description("schema roles".to_string())
                .with_permissions(vec![
                    "schema::can-create-schema".to_string(),
                    "schema::can-update-schema".to_string(),
                ])
                .build()
                .unwrap();

            let builder = RoleListBuilder::new();
            let role_list = builder.with_roles(vec![role.clone()]).build().unwrap();
            let role_bytes = role_list.into_bytes().unwrap();
            let role_address = compute_role_address("test_org.schema");
            self.set_state_entry(role_address, role_bytes).unwrap();
        }

        fn add_agent(&self) {
            let builder = AgentBuilder::new();
            let agent = builder
                .with_org_id("test_org".to_string())
                .with_public_key("agent_public_key".to_string())
                .with_active(true)
                .with_roles(vec!["test_org.schema".to_string()])
                .build()
                .unwrap();

            let builder = AgentListBuilder::new();
            let agent_list = builder.with_agents(vec![agent.clone()]).build().unwrap();
            let agent_bytes = agent_list.into_bytes().unwrap();
            let agent_address = compute_agent_address("agent_public_key");
            self.set_state_entry(agent_address, agent_bytes).unwrap();
        }

        fn add_agent_inactive(&self) {
            let builder = AgentBuilder::new();
            let agent = builder
                .with_org_id("test_org".to_string())
                .with_public_key("agent_public_key".to_string())
                .with_active(false)
                .with_roles(vec!["test_org.schema".to_string()])
                .build()
                .unwrap();

            let builder = AgentListBuilder::new();
            let agent_list = builder.with_agents(vec![agent.clone()]).build().unwrap();
            let agent_bytes = agent_list.into_bytes().unwrap();
            let agent_address = compute_agent_address("agent_public_key");
            self.set_state_entry(agent_address, agent_bytes).unwrap();
        }

        fn add_agent_wrong_organization(&self) {
            let builder = AgentBuilder::new();
            let agent = builder
                .with_org_id("wrong_org".to_string())
                .with_public_key("agent_public_key".to_string())
                .with_active(true)
                .with_roles(vec!["test_org.schema".to_string()])
                .build()
                .unwrap();

            let builder = AgentListBuilder::new();
            let agent_list = builder.with_agents(vec![agent.clone()]).build().unwrap();
            let agent_bytes = agent_list.into_bytes().unwrap();
            let agent_address = compute_agent_address("agent_public_key");
            self.set_state_entry(agent_address, agent_bytes).unwrap();
        }

        fn add_agent_no_roles(&self) {
            let builder = AgentBuilder::new();
            let agent = builder
                .with_org_id("test_org".to_string())
                .with_public_key("agent_public_key".to_string())
                .with_active(true)
                .build()
                .unwrap();

            let builder = AgentListBuilder::new();
            let agent_list = builder.with_agents(vec![agent.clone()]).build().unwrap();
            let agent_bytes = agent_list.into_bytes().unwrap();
            let agent_address = compute_agent_address("agent_public_key");
            self.set_state_entry(agent_address, agent_bytes).unwrap();
        }

        fn add_schema(&self) {
            let builder = PropertyDefinitionBuilder::new();
            let property_definition = builder
                .with_name("TEST".to_string())
                .with_data_type(DataType::Enum)
                .with_description("Optional".to_string())
                .with_enum_options(vec![
                    "One".to_string(),
                    "Two".to_string(),
                    "Three".to_string(),
                ])
                .build()
                .unwrap();

            let builder = SchemaBuilder::new();
            let schema = builder
                .with_name("TestSchema".to_string())
                .with_description("Test Schema".to_string())
                .with_owner("test_org".to_string())
                .with_properties(vec![property_definition.clone()])
                .build()
                .unwrap();

            let builder = SchemaListBuilder::new();
            let schema_list = builder.with_schemas(vec![schema]).build().unwrap();
            let schema_bytes = schema_list.into_bytes().unwrap();
            let schema_address = compute_schema_address("TestSchema");
            self.set_state_entry(schema_address, schema_bytes).unwrap();
        }
    }

    #[test]
    // Test that if a schema with the same name already exists in state an InvalidTransaction
    // is returned
    fn test_create_schema_handler_schema_already_exists() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        let perm_checker = PermissionChecker::new(&transaction_context);
        let state = GridSchemaState::new(&transaction_context);
        let signer = "agent_public_key";

        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaCreateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_owner("test_org".to_string())
            .with_description("Test Schema".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        match schema_create(&action, signer, &state, &perm_checker) {
            Ok(()) => panic!("Schema already exists, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains("Schema with name TestSchema already exists"));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    // Test that if the transaction signer is not an agent an InvalidTransaction
    // is returned
    fn test_create_schema_handler_agent_does_not_exist() {
        let transaction_context = MockTransactionContext::default();
        let perm_checker = PermissionChecker::new(&transaction_context);
        let state = GridSchemaState::new(&transaction_context);
        let signer = "agent_public_key";

        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaCreateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_owner("test_org".to_string())
            .with_description("Test Schema".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        match schema_create(&action, signer, &state, &perm_checker) {
            Ok(()) => panic!("Agent does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InternalError(err)) => {
                assert!(err.contains("Failed to check permissions: InvalidPublicKey: The signer is not an Agent: agent_public_key"));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    // Test that if the agent is inactive an InvalidTransaction
    // is returned
    fn test_create_schema_handler_inactive_agent() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent_inactive();
        transaction_context.add_role();
        let perm_checker = PermissionChecker::new(&transaction_context);
        let state = GridSchemaState::new(&transaction_context);
        let signer = "agent_public_key";

        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaCreateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_owner("test_org".to_string())
            .with_description("Test Schema".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        match schema_create(&action, signer, &state, &perm_checker) {
            Ok(()) => panic!("Agent does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InternalError(err)) => {
                assert!(err.contains("Failed to check permissions: InvalidPublicKey: The signer is not an active agent: agent_public_key"));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    // Test that if the agent has the wrong roles an InvalidTransaction
    // is returned
    fn test_create_schema_handler_no_roles() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent_no_roles();
        let perm_checker = PermissionChecker::new(&transaction_context);
        let state = GridSchemaState::new(&transaction_context);
        let signer = "agent_public_key";

        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaCreateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_owner("test_org".to_string())
            .with_description("Test Schema".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        match schema_create(&action, signer, &state, &perm_checker) {
            Ok(()) => panic!("Agent does not have roles, InvalidTransaction should be returned"),
            Err(ApplyError::InternalError(err)) => {
                assert!(
                    err.contains("Agent agent_public_key does not have the correct permissions")
                );
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    // Test that if the SchemaCreateAction is valid OK is returned
    fn test_create_schema_handler_valid() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent();
        transaction_context.add_role();
        let perm_checker = PermissionChecker::new(&transaction_context);
        let state = GridSchemaState::new(&transaction_context);
        let signer = "agent_public_key";

        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaCreateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_owner("test_org".to_string())
            .with_description("Test Schema".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        assert!(schema_create(&action, signer, &state, &perm_checker).is_ok());
    }

    #[test]
    // Test that if the schema does not exist in state an InvalidTransaction is returned
    fn test_update_schema_handler_schema_does_not_exists() {
        let transaction_context = MockTransactionContext::default();
        let perm_checker = PermissionChecker::new(&transaction_context);
        let state = GridSchemaState::new(&transaction_context);
        let signer = "agent_public_key";

        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaUpdateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_owner("test_org".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        match schema_update(&action, signer, &state, &perm_checker) {
            Ok(()) => panic!("Schema already exists, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains("Schema with name TestSchema does not exist"));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    // Test that if the signer is not an agent an InvalidTransaction is returned
    fn test_update_schema_handler_agent_does_not_exist() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        let perm_checker = PermissionChecker::new(&transaction_context);
        let state = GridSchemaState::new(&transaction_context);
        let signer = "agent_public_key";

        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaUpdateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_owner("test_org".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        match schema_update(&action, signer, &state, &perm_checker) {
            Ok(()) => panic!("Agent does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InternalError(err)) => {
                assert!(err.contains("Failed to check permissions: InvalidPublicKey: The signer is not an Agent: agent_public_key"));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    // Test that if the agent is inactive an InvalidTransaction is returned
    fn test_update_schema_handler_inactive_agent() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        transaction_context.add_role();
        transaction_context.add_agent_inactive();
        let perm_checker = PermissionChecker::new(&transaction_context);
        let state = GridSchemaState::new(&transaction_context);
        let signer = "agent_public_key";

        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaUpdateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_owner("test_org".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        match schema_update(&action, signer, &state, &perm_checker) {
            Ok(()) => panic!("Agent does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InternalError(err)) => {
                assert!(err.contains("Failed to check permissions: InvalidPublicKey: The signer is not an active agent: agent_public_key"));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    // Test that if the agent belongs to the wrong organization an InvalidTransaction is returned
    fn test_update_schema_handler_agent_wrong_org() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        transaction_context.add_role();
        transaction_context.add_agent_wrong_organization();
        let perm_checker = PermissionChecker::new(&transaction_context);
        let state = GridSchemaState::new(&transaction_context);
        let signer = "agent_public_key";

        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaUpdateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_owner("test_org".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        match schema_update(&action, signer, &state, &perm_checker) {
            Ok(()) => panic!("Agent does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InternalError(err)) => {
                assert!(
                    err.contains("Agent agent_public_key does not have the correct permissions")
                );
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    // Test that if the agent has the wrong roles an InvalidTransaction
    // is returned
    fn test_update_schema_handler_no_roles() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent_no_roles();
        transaction_context.add_schema();
        let perm_checker = PermissionChecker::new(&transaction_context);
        let state = GridSchemaState::new(&transaction_context);
        let signer = "agent_public_key";

        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaUpdateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_owner("test_org".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        match schema_update(&action, signer, &state, &perm_checker) {
            Ok(()) => panic!("Agent does not have roles, InvalidTransaction should be returned"),
            Err(ApplyError::InternalError(err)) => {
                assert!(
                    err.contains("Agent agent_public_key does not have the correct permissions")
                );
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    // Test that if a property already exists in that schema an InvalidTransaction is returned
    fn test_update_schema_handler_duplicate_property() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        transaction_context.add_role();
        transaction_context.add_agent();
        let perm_checker = PermissionChecker::new(&transaction_context);
        let state = GridSchemaState::new(&transaction_context);
        let signer = "agent_public_key";

        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaUpdateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_owner("test_org".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        match schema_update(&action, signer, &state, &perm_checker) {
            Ok(()) => panic!("Agent does not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains("Schema already has PropertyDefination with name TEST"));
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    // Test that if the SchemaUpdateAction is valid an OK is returned
    fn test_update_schema_handler_valid() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_schema();
        transaction_context.add_agent();
        transaction_context.add_role();
        let perm_checker = PermissionChecker::new(&transaction_context);
        let state = GridSchemaState::new(&transaction_context);
        let signer = "agent_public_key";

        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("NEW".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaUpdateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_owner("test_org".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        assert!(schema_update(&action, signer, &state, &perm_checker).is_ok());
    }

    /// Computes the address a Pike Agent is stored at based on its public_key
    pub fn compute_agent_address(public_key: &str) -> String {
        let mut sha = Sha512::new();
        sha.input(public_key.as_bytes());

        String::from(PIKE_NAMESPACE) + PIKE_AGENT_NAMESPACE + &sha.result_str()[..60].to_string()
    }

    fn compute_role_address(name: &str) -> String {
        let mut sha = Sha512::new();
        sha.input(name.as_bytes());

        String::from(PIKE_NAMESPACE) + PIKE_ROLE_NAMESPACE + &sha.result_str()[..60]
    }
}
