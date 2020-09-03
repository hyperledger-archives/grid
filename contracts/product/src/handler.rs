// Copyright (c) 2019 Target Brands, Inc.
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
use grid_sdk::protocol::product::payload::{
    Action, ProductCreateAction, ProductDeleteAction, ProductPayload, ProductUpdateAction,
};
use grid_sdk::protocol::product::state::{ProductBuilder, ProductType};

use grid_sdk::protos::FromBytes;

use crate::addressing::*;
use crate::payload::validate_payload;
use crate::state::ProductState;
use crate::validation::validate_gtin;

#[cfg(target_arch = "wasm32")]
// Sabre apply must return a bool
fn apply(
    request: &TpProcessRequest,
    context: &mut dyn TransactionContext,
) -> Result<bool, ApplyError> {
    let handler = ProductTransactionHandler::new();
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
pub struct ProductTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

impl ProductTransactionHandler {
    pub fn new() -> ProductTransactionHandler {
        ProductTransactionHandler {
            family_name: "grid_product".to_string(),
            family_versions: vec!["1.0".to_string()],
            namespaces: vec![get_product_prefix()],
        }
    }

    fn create_product(
        &self,
        payload: &ProductCreateAction,
        state: &mut ProductState,
        signer: &str,
        perm_checker: &PermissionChecker,
    ) -> Result<(), ApplyError> {
        let product_id = payload.product_id();
        let owner = payload.owner();
        let product_type = payload.product_type();
        let properties = payload.properties();

        // Check that the agent submitting the transactions exists in state
        let agent = match state.get_agent(signer)? {
            Some(agent) => agent,
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "The signing Agent does not exist: {}",
                    signer
                )));
            }
        };

        // Check signing agent's permission
        check_permission(perm_checker, signer, "can_create_product")?;

        // Check if product exists in state
        if state.get_product(product_id)?.is_some() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Product already exists: {}",
                product_id,
            )));
        }

        // Check if the product type is a GS1 product
        if product_type != &ProductType::GS1 {
            return Err(ApplyError::InvalidTransaction(
                "Invalid product type enum for product".to_string(),
            ));
        }

        // Check if product product_id is a valid gtin
        if let Err(e) = validate_gtin(product_id) {
            return Err(ApplyError::InvalidTransaction(e.to_string()));
        }

        // Check that the organization ID exists in state
        let org = match state.get_organization(payload.owner())? {
            Some(org) => org,
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "The Agent's organization does not exist: {}",
                    signer,
                )));
            }
        };

        // Check that the agent belongs to organization
        if agent.org_id() != org.org_id() {
            return Err(ApplyError::InvalidTransaction(format!(
                "The signing Agent {} is not associated with organization {}",
                signer,
                org.org_id()
            )));
        }

        /* Check if the agents organization contain GS1 Company Prefix key in its metadata
        (gs1_company_prefixes), and the prefix must match the company prefix in the product_id */
        let metadata = org.metadata().to_vec();
        let gs1_company_prefix_kv = match metadata
            .iter()
            .find(|kv| kv.key() == "gs1_company_prefixes")
        {
            Some(gs1_company_prefix_kv) => gs1_company_prefix_kv,
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "The agents organization does not have the gs1_company_prefixes key in its metadata: {:?}",
                    org.metadata()
                )));
            }
        };
        // If the 'gs1_company_prefixes' key is found
        if gs1_company_prefix_kv.key().is_empty() {
            // If the gtin identifer does not contain the organizations gs1 prefix
            if !product_id.contains(gs1_company_prefix_kv.value()) {
                return Err(ApplyError::InvalidTransaction(format!(
                    "The agents organization does not own the GS1 company prefix in the GTIN product_id: {:?}",
                    org.metadata()
                )));
            }
        }

        let new_product = ProductBuilder::new()
            .with_product_id(product_id.to_string())
            .with_owner(owner.to_string())
            .with_product_type(product_type.clone())
            .with_properties(properties.to_vec())
            .build()
            .map_err(|err| {
                ApplyError::InvalidTransaction(format!("Cannot build product: {}", err))
            })?;

        state.set_product(product_id, new_product)?;

        Ok(())
    }

    fn update_product(
        &self,
        payload: &ProductUpdateAction,
        state: &mut ProductState,
        signer: &str,
        perm_checker: &PermissionChecker,
    ) -> Result<(), ApplyError> {
        let product_id = payload.product_id();
        let product_type = payload.product_type();
        let properties = payload.properties();

        // Check that the agent submitting the transactions exists in state
        let agent = match state.get_agent(signer)? {
            Some(agent) => agent,
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "The signing Agent does not exist: {}",
                    signer
                )));
            }
        };

        // Check signing agent's permission
        check_permission(perm_checker, signer, "can_update_product")?;

        // Check if the product type is a GS1 product
        if product_type != &ProductType::GS1 {
            return Err(ApplyError::InvalidTransaction(
                "Invalid product type enum for product".to_string(),
            ));
        }

        // Check if product exists
        let product = match state.get_product(product_id) {
            Ok(Some(product)) => Ok(product),
            Ok(None) => Err(ApplyError::InvalidTransaction(format!(
                "No product exists: {}",
                product_id
            ))),
            Err(err) => Err(err),
        }?;

        // Check if the agent updating the product is part of the organization associated with the product
        if product.owner() != agent.org_id() {
            return Err(ApplyError::InvalidTransaction(
                "Invalid organization for the agent submitting this transaction".to_string(),
            ));
        }

        // Check if product product_id is a valid gtin
        if let Err(e) = validate_gtin(product_id) {
            return Err(ApplyError::InvalidTransaction(e.to_string()));
        }

        // Handle updating the product
        let updated_product = ProductBuilder::new()
            .with_product_id(product_id.to_string())
            .with_owner(product.owner().to_string())
            .with_product_type(product_type.clone())
            .with_properties(properties.to_vec())
            .build()
            .map_err(|err| {
                ApplyError::InvalidTransaction(format!("Cannot build product: {}", err))
            })?;

        state.set_product(product_id, updated_product)?;

        Ok(())
    }

    fn delete_product(
        &self,
        payload: &ProductDeleteAction,
        state: &mut ProductState,
        signer: &str,
        perm_checker: &PermissionChecker,
    ) -> Result<(), ApplyError> {
        let product_id = payload.product_id();
        let product_type = payload.product_type();

        // Check that the agent submitting the transactions exists in state
        let agent = match state.get_agent(signer)? {
            Some(agent) => agent,
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "The signing Agent does not exist: {}",
                    signer
                )));
            }
        };

        // Check signing agent's permission
        check_permission(perm_checker, signer, "can_delete_product")?;

        // Check if the product type is a GS1 product
        if product_type != &ProductType::GS1 {
            return Err(ApplyError::InvalidTransaction(
                "Invalid product type enum for product".to_string(),
            ));
        }

        // Check if product exists in state
        let product = match state.get_product(product_id) {
            Ok(Some(product)) => Ok(product),
            Ok(None) => Err(ApplyError::InvalidTransaction(format!(
                "No product exists: {}",
                product_id
            ))),
            Err(err) => Err(err),
        }?;

        // Check if product product_id is a valid gtin
        if let Err(e) = validate_gtin(product_id) {
            return Err(ApplyError::InvalidTransaction(e.to_string()));
        }

        // Check that the owner of the products organization is the same as the agent trying to delete the product
        if product.owner() != agent.org_id() {
            return Err(ApplyError::InvalidTransaction(
                "Invalid organization for the agent submitting this transaction".to_string(),
            ));
        }

        // Delete the product
        state.remove_product(product_id)?;
        Ok(())
    }
}

impl TransactionHandler for ProductTransactionHandler {
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
        let payload = ProductPayload::from_bytes(request.get_payload()).map_err(|err| {
            ApplyError::InvalidTransaction(format!("Cannot build product payload: {}", err))
        })?;

        validate_payload(&payload)?;

        info!(
            "Grid Product Payload {:?} {}",
            payload.action(),
            payload.timestamp(),
        );

        let signer = request.get_header().get_signer_public_key();
        let mut state = ProductState::new(context);
        let perm_checker = PermissionChecker::new(context);

        match payload.action() {
            Action::ProductCreate(create_product_payload) => {
                self.create_product(create_product_payload, &mut state, signer, &perm_checker)?
            }
            Action::ProductUpdate(update_product_payload) => {
                self.update_product(update_product_payload, &mut state, signer, &perm_checker)?
            }
            Action::ProductDelete(delete_product_payload) => {
                self.delete_product(delete_product_payload, &mut state, signer, &perm_checker)?
            }
        }
        Ok(())
    }
}

fn check_permission(
    perm_checker: &PermissionChecker,
    signer: &str,
    permission: &str,
) -> Result<(), ApplyError> {
    match perm_checker.has_permission(signer, permission) {
        Ok(true) => Ok(()),
        Ok(false) => Err(ApplyError::InvalidTransaction(format!(
            "The signer does not have the {} permission: {}.",
            permission, signer,
        ))),
        Err(e) => Err(ApplyError::InvalidTransaction(format!("{}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::collections::HashMap;

    use grid_sdk::protocol::pike::state::{
        AgentBuilder, AgentListBuilder, KeyValueEntryBuilder, OrganizationBuilder,
        OrganizationListBuilder,
    };
    use grid_sdk::protocol::product::payload::{
        ProductCreateAction, ProductCreateActionBuilder, ProductDeleteAction,
        ProductDeleteActionBuilder, ProductUpdateAction, ProductUpdateActionBuilder,
    };
    use grid_sdk::protocol::product::state::{
        Product, ProductBuilder, ProductListBuilder, ProductType,
    };
    use grid_sdk::protocol::schema::state::{DataType, PropertyValue, PropertyValueBuilder};
    use grid_sdk::protos::IntoBytes;

    use sawtooth_sdk::processor::handler::{ContextError, TransactionContext};

    const AGENT_ORG_ID: &str = "test_org";
    const PUBLIC_KEY: &str = "test_public_key";
    const PRODUCT_ID: &str = "688955434684";
    const PRODUCT_2_ID: &str = "9781981855728";

    #[derive(Default, Debug)]
    /// A MockTransactionContext that can be used to test ProductState
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
                .with_org_id(AGENT_ORG_ID.to_string())
                .with_public_key(public_key.to_string())
                .with_active(true)
                .with_roles(vec![
                    "can_create_product".to_string(),
                    "can_update_product".to_string(),
                    "can_delete_product".to_string(),
                ])
                .build()
                .unwrap();

            let builder = AgentListBuilder::new();
            let agent_list = builder.with_agents(vec![agent.clone()]).build().unwrap();
            let agent_bytes = agent_list.into_bytes().unwrap();
            let agent_address = compute_agent_address(public_key);
            self.set_state_entry(agent_address, agent_bytes).unwrap();
        }

        fn add_agent_without_roles(&self, public_key: &str) {
            let builder = AgentBuilder::new();
            let agent = builder
                .with_org_id(AGENT_ORG_ID.to_string())
                .with_public_key(public_key.to_string())
                .with_active(true)
                .with_roles(vec![])
                .build()
                .unwrap();

            let builder = AgentListBuilder::new();
            let agent_list = builder.with_agents(vec![agent.clone()]).build().unwrap();
            let agent_bytes = agent_list.into_bytes().unwrap();
            let agent_address = compute_agent_address(public_key);
            self.set_state_entry(agent_address, agent_bytes).unwrap();
        }

        fn add_org(&self, org_id: &str) {
            // Products can only be created when there is a gs1 prefix
            // within the product organization's metadata
            let builder = KeyValueEntryBuilder::new();
            let key_value = builder
                .with_key("gs1_company_prefixes".to_string())
                .with_value("test_value".to_string())
                .build()
                .unwrap();

            let builder = OrganizationBuilder::new();
            let org = builder
                .with_org_id(org_id.to_string())
                .with_name("test_org_name".to_string())
                .with_address("test_org_address".to_string())
                .with_metadata(vec![key_value.clone()])
                .build()
                .unwrap();

            let builder = OrganizationListBuilder::new();
            let org_list = builder
                .with_organizations(vec![org.clone()])
                .build()
                .unwrap();
            let org_bytes = org_list.into_bytes().unwrap();
            let org_address = compute_org_address(org_id);
            self.set_state_entry(org_address, org_bytes).unwrap();
        }

        fn add_org_without_gs1_prefix(&self, org_id: &str) {
            let builder = OrganizationBuilder::new();
            let org = builder
                .with_org_id(org_id.to_string())
                .with_name("test_org_name".to_string())
                .with_address("test_org_address".to_string())
                .build()
                .unwrap();

            let builder = OrganizationListBuilder::new();
            let org_list = builder
                .with_organizations(vec![org.clone()])
                .build()
                .unwrap();
            let org_bytes = org_list.into_bytes().unwrap();
            let org_address = compute_org_address(org_id);
            self.set_state_entry(org_address, org_bytes).unwrap();
        }

        fn add_product(&self, prod_id: &str) {
            let product_list = ProductListBuilder::new()
                .with_products(vec![make_product()])
                .build()
                .unwrap();
            let product_bytes = product_list.into_bytes().unwrap();
            let product_address = make_product_address(prod_id);
            self.set_state_entry(product_address, product_bytes)
                .unwrap();
        }

        fn add_products(&self, product_ids: &[&str]) {
            let product_list = ProductListBuilder::new()
                .with_products(make_products(product_ids))
                .build()
                .unwrap();
            let product_list_bytes = product_list.into_bytes().unwrap();
            let product_list_bytes_copy = product_list_bytes.clone();
            let product_1_address = make_product_address(PRODUCT_ID);
            let product_2_address = make_product_address(PRODUCT_2_ID);
            self.set_state_entries(vec![
                (product_1_address, product_list_bytes),
                (product_2_address, product_list_bytes_copy),
            ])
            .unwrap();
        }
    }

    #[test]
    /// Test that if ProductCreationAction is valid an OK is returned and a new Product is added to state
    fn test_create_product_handler_valid() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_org(AGENT_ORG_ID);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = ProductState::new(&transaction_context);

        let transaction_handler = ProductTransactionHandler::new();
        let product_create_action = make_product_create_action();

        assert!(transaction_handler
            .create_product(
                &product_create_action,
                &mut state,
                PUBLIC_KEY,
                &perm_checker
            )
            .is_ok());

        let product = state
            .get_product(PRODUCT_ID)
            .expect("Failed to fetch product")
            .expect("No product found");

        assert_eq!(product, make_product());
    }

    #[test]
    /// Test that ProductCreationAction is invalid if the signer is not an Agent.
    fn test_create_product_agent_does_not_exist() {
        let transaction_context = MockTransactionContext::default();
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = ProductState::new(&transaction_context);

        let transaction_handler = ProductTransactionHandler::new();
        let product_create_action = make_product_create_action();

        match transaction_handler.create_product(
            &product_create_action,
            &mut state,
            PUBLIC_KEY,
            &perm_checker,
        ) {
            Ok(()) => panic!("Agent should not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("The signing Agent does not exist: {}", PUBLIC_KEY)));
            }
            Err(err) => panic!("Should have gotten invalid error but go {}", err),
        }
    }

    #[test]
    /// Test that ProductCreationAction is invalid if the agent does not have can_create_product role
    fn test_create_product_agent_without_roles() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent_without_roles(PUBLIC_KEY);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = ProductState::new(&transaction_context);

        let transaction_handler = ProductTransactionHandler::new();
        let product_create_action = make_product_create_action();

        match transaction_handler.create_product(
            &product_create_action,
            &mut state,
            PUBLIC_KEY,
            &perm_checker,
        ) {
            Ok(()) => panic!(
                "Agent should not have can_create_product role, InvalidTransaction should be returned"
            ),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!(
                    "The signer does not have the can_create_product permission: {}",
                    PUBLIC_KEY
                )));
            }
            Err(err) => panic!("Should have gotten invalid error but go {}", err),
        }
    }

    #[test]
    /// Test that ProductCreationAction is invalid if the agent's org does not exist.
    fn test_create_product_org_does_not_exist() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = ProductState::new(&transaction_context);

        let transaction_handler = ProductTransactionHandler::new();
        let product_create_action = make_product_create_action();

        match transaction_handler.create_product(
            &product_create_action,
            &mut state,
            PUBLIC_KEY,
            &perm_checker,
        ) {
            Ok(()) => panic!(
                "Agent's organization should not exist, InvalidTransaction should be returned"
            ),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!(
                    "The Agent's organization does not exist: {}",
                    PUBLIC_KEY
                )));
            }
            Err(err) => panic!("Should have gotten invalid error but go {}", err),
        }
    }

    #[test]
    /// Test that ProductCreationAction is invalid if the agent's org does not contain the gs1 prefix.
    fn test_create_product_org_without_gs1_prefix() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_org_without_gs1_prefix(AGENT_ORG_ID);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = ProductState::new(&transaction_context);

        let transaction_handler = ProductTransactionHandler::new();
        let product_create_action = make_product_create_action();

        match transaction_handler.create_product(
            &product_create_action,
            &mut state,
            PUBLIC_KEY,
            &perm_checker
        ) {
            Ok(()) => panic!("Agent's organization should not have a gs1 prefix key, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains("The agents organization does not have the gs1_company_prefixes key in its metadata: []"));
            }
            Err(err) => panic!("Should have gotten invalid error but go {}", err),
        }
    }

    #[test]
    /// Test that ProductCreationAction is invalid if the a product with the same id
    /// already exists.
    fn test_create_product_already_exist() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_org(AGENT_ORG_ID);
        transaction_context.add_product(PRODUCT_ID);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = ProductState::new(&transaction_context);

        let transaction_handler = ProductTransactionHandler::new();
        let product_create_action = make_product_create_action();

        match transaction_handler.create_product(
            &product_create_action,
            &mut state,
            PUBLIC_KEY,
            &perm_checker,
        ) {
            Ok(()) => panic!("Product should not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("Product already exists: {}", PRODUCT_ID)));
            }
            Err(err) => panic!("Should have gotten invalid error but go {}", err),
        }
    }

    #[test]
    /// Test that if ProductUpdateAction is valid an OK is returned and a Product is updated in state
    fn test_update_product_handler_valid() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_org(AGENT_ORG_ID);
        transaction_context.add_product(PRODUCT_ID);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = ProductState::new(&transaction_context);

        let transaction_handler = ProductTransactionHandler::new();
        let product_update_action = make_product_update_action();

        assert!(transaction_handler
            .update_product(
                &product_update_action,
                &mut state,
                PUBLIC_KEY,
                &perm_checker,
            )
            .is_ok());

        let product = state
            .get_product(PRODUCT_ID)
            .expect("Failed to fetch product")
            .expect("No product found");

        assert_eq!(product, make_updated_product());
    }

    #[test]
    /// Test that ProductUpdateAction is invalid if the signer is not an Agent.
    fn test_update_product_agent_does_not_exist() {
        let transaction_context = MockTransactionContext::default();
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = ProductState::new(&transaction_context);

        let transaction_handler = ProductTransactionHandler::new();
        let product_update_action = make_product_update_action();

        match transaction_handler.update_product(
            &product_update_action,
            &mut state,
            PUBLIC_KEY,
            &perm_checker,
        ) {
            Ok(()) => panic!("Agent should not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("The signing Agent does not exist: {}", PUBLIC_KEY)));
            }
            Err(err) => panic!("Should have gotten invalid error but go {}", err),
        }
    }

    #[test]
    /// Test that ProductUpdateAction is invalid if the agent does not have can_update_product role
    fn test_update_product_agent_without_roles() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent_without_roles(PUBLIC_KEY);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = ProductState::new(&transaction_context);

        let transaction_handler = ProductTransactionHandler::new();
        let product_update_action = make_product_update_action();

        match transaction_handler.update_product(
            &product_update_action,
            &mut state,
            PUBLIC_KEY,
            &perm_checker,
        ) {
            Ok(()) => panic!(
                "Agent should not have can_update_product role, InvalidTransaction should be returned"
            ),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!(
                    "The signer does not have the can_update_product permission: {}",
                    PUBLIC_KEY
                )));
            }
            Err(err) => panic!("Should have gotten invalid error but go {}", err),
        }
    }

    #[test]
    /// Test that ProductUpdateAction is invalid if there is no product to update
    fn test_update_product_that_does_not_exist() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = ProductState::new(&transaction_context);

        let transaction_handler = ProductTransactionHandler::new();
        let product_update_action = make_product_update_action();

        match transaction_handler.update_product(
            &product_update_action,
            &mut state,
            PUBLIC_KEY,
            &perm_checker,
        ) {
            Ok(()) => panic!("Product should not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("No product exists: {}", PRODUCT_ID)));
            }
            Err(err) => panic!("Should have gotten invalid error but go {}", err),
        }
    }

    #[test]
    /// Test that if ProductDeleteAction is valid an OK is returned and a Product is deleted from state
    fn test_delete_product_handler_valid() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_org(AGENT_ORG_ID);
        transaction_context.add_products(&vec![PRODUCT_ID, PRODUCT_2_ID]);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = ProductState::new(&transaction_context);

        let transaction_handler = ProductTransactionHandler::new();
        let product_delete_action = make_product_delete_action(PRODUCT_ID);

        assert!(transaction_handler
            .delete_product(
                &product_delete_action,
                &mut state,
                PUBLIC_KEY,
                &perm_checker
            )
            .is_ok());

        let product = state.get_product(PRODUCT_ID).expect("No product found");

        assert_eq!(product, None);
    }

    #[test]
    /// Test that if ProductDeleteAction is valid an OK is returned and a
    /// second product is deleted from state
    fn test_delete_second_product_handler_valid() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_org(AGENT_ORG_ID);
        transaction_context.add_products(&vec![PRODUCT_ID, PRODUCT_2_ID]);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = ProductState::new(&transaction_context);

        let transaction_handler = ProductTransactionHandler::new();
        let product_delete_action = make_product_delete_action(PRODUCT_2_ID);

        assert!(transaction_handler
            .delete_product(
                &product_delete_action,
                &mut state,
                PUBLIC_KEY,
                &perm_checker,
            )
            .is_ok());

        let product = state.get_product(PRODUCT_2_ID).expect("No product found");

        assert_eq!(product, None);
    }

    #[test]
    /// Test that ProductDeleteAction is invalid if the agent does not have can_delete_product role
    fn test_delete_product_agent_without_roles() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent_without_roles(PUBLIC_KEY);
        transaction_context.add_org(AGENT_ORG_ID);
        transaction_context.add_products(&vec![PRODUCT_ID, PRODUCT_2_ID]);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = ProductState::new(&transaction_context);

        let transaction_handler = ProductTransactionHandler::new();
        let product_delete_action = make_product_delete_action(PRODUCT_ID);

        match transaction_handler.delete_product(
            &product_delete_action,
            &mut state,
            PUBLIC_KEY,
            &perm_checker,
        ) {
            Ok(()) => panic!(
                "Agent should not have can_delete_product role, InvalidTransaction should be returned"
            ),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!(
                    "The signer does not have the can_delete_product permission: {}",
                    PUBLIC_KEY
                )));
            }
            Err(err) => panic!("Should have gotten invalid error but go {}", err),
        }
    }

    #[test]
    /// Test that ProductDeleteAction is invalid when deleting a non existant product
    fn test_delete_product_not_exists() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_org(AGENT_ORG_ID);
        transaction_context.add_products(&vec![PRODUCT_ID, PRODUCT_2_ID]);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = ProductState::new(&transaction_context);

        let transaction_handler = ProductTransactionHandler::new();
        let product_delete_action = make_product_delete_action("13491387613");

        match transaction_handler.delete_product(
            &product_delete_action,
            &mut state,
            PUBLIC_KEY,
            &perm_checker,
        ) {
            Ok(()) => panic!("Product should not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains("No product exists: 13491387613"));
            }
            Err(err) => panic!("Should have gotten invalid error but go {}", err),
        }
    }

    fn make_product() -> Product {
        ProductBuilder::new()
            .with_product_id(PRODUCT_ID.to_string())
            .with_owner(AGENT_ORG_ID.to_string())
            .with_product_type(ProductType::GS1)
            .with_properties(make_properties())
            .build()
            .expect("Failed to build new_product")
    }

    fn make_products(product_ids: &[&str]) -> Vec<Product> {
        vec![
            ProductBuilder::new()
                .with_product_id(product_ids[0].to_string())
                .with_owner(AGENT_ORG_ID.to_string())
                .with_product_type(ProductType::GS1)
                .with_properties(make_properties())
                .build()
                .expect("Failed to build new_product"),
            ProductBuilder::new()
                .with_product_id(product_ids[1].to_string())
                .with_owner(AGENT_ORG_ID.to_string())
                .with_product_type(ProductType::GS1)
                .with_properties(make_properties())
                .build()
                .expect("Failed to build new_product"),
        ]
    }

    fn make_updated_product() -> Product {
        ProductBuilder::new()
            .with_product_id(PRODUCT_ID.to_string())
            .with_owner(AGENT_ORG_ID.to_string())
            .with_product_type(ProductType::GS1)
            .with_properties(make_updated_properties())
            .build()
            .expect("Failed to build new_product")
    }

    fn make_properties() -> Vec<PropertyValue> {
        let property_value_description = PropertyValueBuilder::new()
            .with_name("description".into())
            .with_data_type(DataType::String)
            .with_string_value("This is a product description".into())
            .build()
            .unwrap();
        let property_value_price = PropertyValueBuilder::new()
            .with_name("price".into())
            .with_data_type(DataType::Number)
            .with_number_value(3)
            .build()
            .unwrap();

        vec![
            property_value_description.clone(),
            property_value_price.clone(),
        ]
    }

    fn make_updated_properties() -> Vec<PropertyValue> {
        let property_value_description = PropertyValueBuilder::new()
            .with_name("description".into())
            .with_data_type(DataType::String)
            .with_string_value("This is a new product description".into())
            .build()
            .unwrap();
        let property_value_price = PropertyValueBuilder::new()
            .with_name("price".into())
            .with_data_type(DataType::Number)
            .with_number_value(4)
            .build()
            .unwrap();

        vec![
            property_value_description.clone(),
            property_value_price.clone(),
        ]
    }

    fn make_product_create_action() -> ProductCreateAction {
        ProductCreateActionBuilder::new()
            .with_product_id(PRODUCT_ID.to_string())
            .with_owner(AGENT_ORG_ID.to_string())
            .with_product_type(ProductType::GS1)
            .with_properties(make_properties())
            .build()
            .expect("Failed to build ProductCreateAction")
    }

    fn make_product_update_action() -> ProductUpdateAction {
        ProductUpdateActionBuilder::new()
            .with_product_id(PRODUCT_ID.to_string())
            .with_product_type(ProductType::GS1)
            .with_properties(make_updated_properties())
            .build()
            .expect("Failed to build ProductUpdateAction")
    }

    fn make_product_delete_action(product_id: &str) -> ProductDeleteAction {
        ProductDeleteActionBuilder::new()
            .with_product_id(product_id.to_string())
            .with_product_type(ProductType::GS1)
            .build()
            .expect("Failed to build ProductDeleteAction")
    }
}
