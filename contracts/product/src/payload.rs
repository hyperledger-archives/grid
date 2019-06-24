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
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
    }
}

use grid_sdk::protos::product_payload::{
    ProductCreateAction, ProductCreateAction_ProductType, ProductDeleteAction,
    ProductDeleteAction_ProductType, ProductPayload as Product_Payload_Proto,
    ProductPayload_Action, ProductUpdateAction, ProductUpdateAction_ProductType,
};

#[derive(Debug, Clone)]
pub enum Action {
    CreateProduct(ProductCreateAction),
    UpdateProduct(ProductUpdateAction),
    DeleteProduct(ProductDeleteAction),
}

pub struct ProductPayload {
    action: Action,
    timestamp: u64,
}

impl ProductPayload {
    pub fn new(payload: &[u8]) -> Result<Option<ProductPayload>, ApplyError> {
        let payload: Product_Payload_Proto = match protobuf::parse_from_bytes(payload) {
            Ok(payload) => payload,
            Err(_) => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Cannot deserialize payload",
                )));
            }
        };

        let product_action = payload.get_action();
        let action = match product_action {
            ProductPayload_Action::PRODUCT_CREATE => {
                let product_create = payload.get_product_create();
                if product_create.get_identifier().is_empty() {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Product id cannot be an empty string",
                    )));
                }

                if product_create.get_owner().is_empty() {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Product owner cannot be an empty string",
                    )));
                }

                if product_create.get_product_type() == ProductCreateAction_ProductType::UNSET_TYPE
                {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Product type cannot be: UNSET_TYPE",
                    )));
                }

                Action::CreateProduct(product_create.clone())
            }

            ProductPayload_Action::PRODUCT_UPDATE => {
                let product_update = payload.get_product_update();
                if product_update.get_identifier().is_empty() {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Product id cannot be an empty string",
                    )));
                }

                if product_update.get_product_type() == ProductUpdateAction_ProductType::UNSET_TYPE
                {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Product type cannot be: UNSET_TYPE",
                    )));
                }
                Action::UpdateProduct(product_update.clone())
            }

            ProductPayload_Action::PRODUCT_DELETE => {
                let product_delete = payload.get_product_delete();
                if product_delete.get_identifier().is_empty() {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Product id cannot be an empty string",
                    )));
                }

                if product_delete.get_product_type() == ProductDeleteAction_ProductType::UNSET_TYPE
                {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Product type cannot be: UNSET_TYPE",
                    )));
                }

                Action::DeleteProduct(product_delete.clone())
            }

            ProductPayload_Action::UNSET_ACTION => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "No action specified",
                )));
            }
        };

        let timestamp = payload.get_timestamp();
        Ok(Some(ProductPayload { action, timestamp }))
    }

    pub fn get_action(&self) -> Action {
        self.action.clone()
    }

    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }
}
