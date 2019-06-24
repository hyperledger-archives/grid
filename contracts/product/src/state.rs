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
use grid_sdk::protos::product_state::Product;
use grid_sdk::protos::product_state::ProductList;
use grid_sdk::protos::FromBytes;

use protobuf::Message;

use crate::addressing::*;

pub struct ProductState<'a> {
    context: &'a dyn TransactionContext,
}

impl<'a> ProductState<'a> {
    pub fn new(context: &'a dyn TransactionContext) -> ProductState {
        ProductState { context }
    }

    pub fn get_product(&mut self, product_id: &str) -> Result<Option<Product>, ApplyError> {
        let address = make_product_address(product_id); //product id = gtin
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let products: ProductList = match protobuf::parse_from_bytes(packed.as_slice()) {
                    Ok(products) => products,
                    Err(_) => {
                        return Err(ApplyError::InternalError(String::from(
                            "Cannot deserialize product list",
                        )));
                    }
                };

                Ok(products
                    .get_entries()
                    .iter()
                    .find(|p| p.get_identifier() == product_id)
                    .cloned())
            }
            None => Ok(None),
        }
    }

    pub fn set_product(&mut self, product_id: &str, product: Product) -> Result<(), ApplyError> {
        let address = make_product_address(product_id);
        let d = self.context.get_state_entry(&address)?;
        let mut product_container = match d {
            Some(packed) => match protobuf::parse_from_bytes(packed.as_slice()) {
                Ok(products) => products,
                Err(_) => {
                    return Err(ApplyError::InternalError(String::from(
                        "Cannot deserialize product list",
                    )));
                }
            },
            None => ProductList::new(),
        };

        let mut products = product_container
            .take_entries()
            .into_iter()
            .filter(|p| p.get_identifier() != product_id)
            .collect::<Vec<_>>();
        products.push(product);
        products.sort_by(|p1, p2| p1.get_identifier().cmp(p2.get_identifier()));
        product_container.set_entries(protobuf::RepeatedField::from_vec(products));

        let serialized = match product_container.write_to_bytes() {
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

    // Currently product_id = gtin
    pub fn remove_product(&mut self, product_id: &str) -> Result<(), ApplyError> {
        let address = make_product_address(product_id);
        let d = self.context.get_state_entry(&address)?;
        let mut product_container = match d {
            Some(packed) => match protobuf::parse_from_bytes(packed.as_slice()) {
                Ok(products) => products,
                Err(_) => {
                    return Err(ApplyError::InternalError(String::from(
                        "Cannot deserialize product list",
                    )));
                }
            },
            None => ProductList::new(),
        };

        // Collect a new vector of products without the removed item
        let products = product_container
            .take_entries()
            .into_iter()
            .filter(|p| p.get_identifier() != product_id)
            .collect::<Vec<_>>();

        // Reset product list to the new vector
        product_container.set_entries(protobuf::RepeatedField::from_vec(products));

        let serialized = match product_container.write_to_bytes() {
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

    pub fn get_organization(&mut self, id: &str) -> Result<Option<Organization>, ApplyError> {
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
