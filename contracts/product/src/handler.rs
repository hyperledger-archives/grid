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
use grid_sdk::protos::product_payload::*;
use grid_sdk::protos::product_state::Product;

use crate::addressing::*;
use crate::payload::{Action, ProductPayload};
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
        Err(err) => Err(err),
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
        payload: ProductCreateAction,
        mut state: ProductState,
        signer: &str,
        perm_checker: &PermissionChecker,
    ) -> Result<(), ApplyError> {
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

        // Check that the agent has an organization associated with it
        if agent.org_id().is_empty() {
            return Err(ApplyError::InvalidTransaction(format!(
                "The signing Agent does not have an associated organization: {}",
                signer
            )));
        }

        // Check that the agent has the pike permission "can_create_product" for the organization
        check_permission(perm_checker, signer, "can_create_product")?;

        // Check if the product type is a GS1 product
        if payload.get_product_type() != ProductCreateAction_ProductType::GS1 {
            return Err(ApplyError::InvalidTransaction(
                "Invalid product type enum for product".to_string(),
            ));
        }
        // Use this varible to pass in the type correct enum (product_state) on product create
        let product_type = grid_sdk::protos::product_state::Product_ProductType::GS1;

        // Check if product identifier is a valid gtin
        let product_id = payload.get_identifier();
        if let Err(e) = validate_gtin(product_id) {
            return Err(ApplyError::InvalidTransaction(e.to_string()));
        }

        // Check that the org owns the GS1 company prefix in the identifier
        let org = match state.get_organization(payload.get_owner())? {
            Some(org) => org,
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "The Agents organization does not exist: {}",
                    signer
                )));
            }
        };

        // Check if product exists in state
        match state.get_product(product_id) {
            Ok(Some(_)) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Product already exists: {}",
                    product_id
                )));
            }
            Ok(None) => (),
            Err(err) => return Err(err),
        }

        /* Check if the agents organization contain GS1 Company Prefix key in its metadata
        (gs1_company_prefixes), and the prefix must match the company prefix in the identifier */
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
                    "The agents organization does not own the GS1 company prefix in the GTIN identifier: {:?}",
                    org.metadata()
                )));
            }
        }

        let mut new_product = Product::new();
        new_product.set_identifier(product_id.to_string());
        new_product.set_owner(payload.get_owner().to_string());
        new_product.set_field_type(product_type);
        new_product.set_product_values(protobuf::RepeatedField::from_vec(
            payload.get_properties().to_vec(),
        ));

        state.set_product(signer, new_product)?;
        Ok(())
    }

    fn update_product(
        &self,
        payload: ProductUpdateAction,
        mut state: ProductState,
        signer: &str,
        perm_checker: &PermissionChecker,
    ) -> Result<(), ApplyError> {
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

        // Check if the product type is a GS1 product
        let product_type = payload.get_product_type();
        if product_type != ProductUpdateAction_ProductType::GS1 {
            return Err(ApplyError::InvalidTransaction(
                "Invalid product type enum for product".to_string(),
            ));
        }
        let product_id = payload.get_identifier();

        // Check if product identifier is a valid gtin
        if let Err(e) = validate_gtin(product_id) {
            return Err(ApplyError::InvalidTransaction(e.to_string()));
        }

        // Check if product exists
        let mut product = match state.get_product(product_id) {
            Ok(Some(product)) => Ok(product),
            Ok(None) => Err(ApplyError::InvalidTransaction(format!(
                "No product exists: {}",
                product_id
            ))),
            Err(err) => Err(err),
        }?;

        // Check if the agent updating the product is part of the organization associated with the product
        if product.get_owner() != agent.org_id() {
            return Err(ApplyError::InvalidTransaction(
                "Invalid organization for the agent submitting this transaction".to_string(),
            ));
        }

        // Check that the agent has the pike permission "can_update_product" for the organization
        check_permission(perm_checker, signer, "can_update_product")?;

        // Handle updating the product
        let updated_product_values = payload.properties.clone();
        product.set_product_values(updated_product_values);

        state.set_product(product_id, product)?;
        Ok(())
    }

    fn delete_product(
        &self,
        payload: ProductDeleteAction,
        mut state: ProductState,
        signer: &str,
        perm_checker: &PermissionChecker,
    ) -> Result<(), ApplyError> {
        // Check that the agent (signer) submitting the transactions exists in state
        let agent = match state.get_agent(signer)? {
            Some(agent) => agent,
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "The signing Agent does not exist: {}",
                    signer
                )));
            }
        };

        // Check if the product type is a GS1 product
        let product_type = payload.get_product_type();
        if product_type != ProductDeleteAction_ProductType::GS1 {
            return Err(ApplyError::InvalidTransaction(
                "Invalid product type enum for product".to_string(),
            ));
        }
        let product_id = payload.get_identifier();

        // Check if product identifier is a valid gtin
        if let Err(e) = validate_gtin(product_id) {
            return Err(ApplyError::InvalidTransaction(e.to_string()));
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

        // Check that the owner of the products organization is the same as the agent trying to delete the product
        if product.get_owner() != agent.org_id() {
            return Err(ApplyError::InvalidTransaction(
                "Invalid organization for the agent submitting this transaction".to_string(),
            ));
        }

        // Check that the agent deleting the product has the "can_delete_product" permission for the organization
        check_permission(perm_checker, signer, "can_delete_product")?;

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
        let payload = ProductPayload::new(request.get_payload());
        let payload = match payload {
            Err(e) => return Err(e),
            Ok(payload) => payload,
        };
        let payload = match payload {
            Some(x) => x,
            None => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Request must contain a payload",
                )));
            }
        };

        if payload.get_timestamp().to_string().is_empty() {
            return Err(ApplyError::InvalidTransaction(String::from(
                "Timestamp is not set",
            )));
        }

        let signer = request.get_header().get_signer_public_key();
        let state = ProductState::new(context);
        let perm_checker = PermissionChecker::new(context);

        match payload.get_action() {
            Action::CreateProduct(create_product_payload) => {
                self.create_product(create_product_payload, state, signer, &perm_checker)?
            }
            Action::UpdateProduct(update_product_payload) => {
                self.update_product(update_product_payload, state, signer, &perm_checker)?
            }
            Action::DeleteProduct(delete_product_payload) => {
                self.delete_product(delete_product_payload, state, signer, &perm_checker)?
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
