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
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
        use sawtooth_sdk::processor::handler::TransactionContext;
    }
}

use grid_sdk::protocol::pike::state::{Agent, AgentList};
use grid_sdk::protocol::pike::state::{Organization, OrganizationList};
use grid_sdk::protocol::product::state::{Product, ProductList, ProductListBuilder};
use grid_sdk::protos::{FromBytes, IntoBytes};

use crate::addressing::*;

pub struct ProductState<'a> {
    context: &'a dyn TransactionContext,
}

impl<'a> ProductState<'a> {
    pub fn new(context: &'a dyn TransactionContext) -> ProductState {
        ProductState { context }
    }

    pub fn get_product(&self, product_id: &str) -> Result<Option<Product>, ApplyError> {
        let address = make_product_address(product_id); //product id = gtin
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let products = match ProductList::from_bytes(packed.as_slice()) {
                    Ok(products) => products,
                    Err(_) => {
                        return Err(ApplyError::InternalError(String::from(
                            "Cannot deserialize product list",
                        )));
                    }
                };

                // find the product with the correct id
                Ok(products
                    .products()
                    .iter()
                    .find(|p| p.product_id() == product_id)
                    .cloned())
            }
            None => Ok(None),
        }
    }

    pub fn set_product(&self, product_id: &str, product: Product) -> Result<(), ApplyError> {
        let address = make_product_address(product_id);
        let d = self.context.get_state_entry(&address)?;
        let mut products = match d {
            Some(packed) => match ProductList::from_bytes(packed.as_slice()) {
                Ok(product_list) => product_list.products().to_vec(),
                Err(err) => {
                    return Err(ApplyError::InternalError(format!(
                        "Cannot deserialize product list: {:?}",
                        err
                    )));
                }
            },
            None => vec![],
        };

        let mut index = None;
        for (i, product) in products.iter().enumerate() {
            if product.product_id() == product_id {
                index = Some(i);
                break;
            }
        }

        if let Some(i) = index {
            products.remove(i);
        }
        products.push(product);
        products.sort_by_key(|r| r.product_id().to_string());
        let product_list = ProductListBuilder::new()
            .with_products(products)
            .build()
            .map_err(|err| {
                ApplyError::InvalidTransaction(format!("Cannot build product list: {:?}", err))
            })?;

        let serialized = match product_list.into_bytes() {
            Ok(serialized) => serialized,
            Err(err) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Cannot serialize product list: {:?}",
                    err
                )));
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    // Currently product_id = gtin
    pub fn remove_product(&self, product_id: &str) -> Result<(), ApplyError> {
        let address = make_product_address(product_id);
        let d = self.context.get_state_entry(&address)?;
        let products = match d {
            Some(packed) => match ProductList::from_bytes(packed.as_slice()) {
                Ok(product_list) => product_list.products().to_vec(),
                Err(err) => {
                    return Err(ApplyError::InternalError(format!(
                        "Cannot deserialize product list: {:?}",
                        err
                    )));
                }
            },
            None => vec![],
        };

        // Collect a new vector of products without the removed item
        let product_list = ProductListBuilder::new()
            .with_products(
                products
                    .into_iter()
                    .filter(|p| p.product_id() != product_id)
                    .collect::<Vec<_>>(),
            )
            .build()
            .map_err(|err| {
                ApplyError::InvalidTransaction(format!("Cannot build product list: {:?}", err))
            })?;

        let serialized = match product_list.into_bytes() {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize product list",
                )));
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    /// Gets a Pike Agent. Handles retrieving the correct agent from an AgentList.
    pub fn get_agent(&self, public_key: &str) -> Result<Option<Agent>, ApplyError> {
        let address = compute_agent_address(public_key);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let agents: AgentList = match AgentList::from_bytes(packed.as_slice()) {
                    Ok(agents) => agents,
                    Err(err) => {
                        return Err(ApplyError::InvalidTransaction(format!(
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

    pub fn get_organization(&self, id: &str) -> Result<Option<Organization>, ApplyError> {
        let address = compute_org_address(id);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let orgs: OrganizationList = match OrganizationList::from_bytes(packed.as_slice()) {
                    Ok(orgs) => orgs,
                    Err(err) => {
                        return Err(ApplyError::InternalError(format!(
                            "Cannot deserialize organization list: {:?}",
                            err,
                        )))
                    }
                };

                for org in orgs.organizations() {
                    if org.org_id() == id {
                        return Ok(Some(org.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }
}
