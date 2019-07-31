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
            namespaces: vec![get_product_prefix().to_string()],
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

        // Check that the agent has an organization associated with it
        if agent.org_id().is_empty() {
            return Err(ApplyError::InvalidTransaction(format!(
                "The signing Agent does not have an associated organization: {}",
                signer
            )));
        }

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

        // Check that the org owns the GS1 company prefix in the product_id
        let org = match state.get_organization(payload.owner())? {
            Some(org) => org,
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "The Agent's organization does not exist: {}",
                    signer,
                )));
            }
        };

        /* Check if the agents organization contain GS1 Company Prefix key in its metadata
        (gs1_company_prefixes), and the prefix must match the company prefix in the product_id */
        let gs1_company_prefix_vec = org.metadata().to_vec();
        let gs1_company_prefix_kv = match gs1_company_prefix_vec
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

        // Check that the agent has an organization associated with it
        if agent.org_id().is_empty() {
            return Err(ApplyError::InvalidTransaction(format!(
                "The signing Agent does not have an associated organization: {}",
                signer
            )));
        }

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
