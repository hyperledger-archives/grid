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
use grid_sdk::protocol::schema::state::{Schema, SchemaList};
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
        let address = compute_gs1_product_address(product_id); //product id = gtin
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
        let address = compute_gs1_product_address(product_id);
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
        let address = compute_gs1_product_address(product_id);
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

        // Filter out the product we are deleting
        let filtered_products = products
            .into_iter()
            .filter(|p| p.product_id() != product_id)
            .collect::<Vec<_>>();

        // If the only product at the address was the one we are removing, we can delete the entire state entry
        // Else, we can set the the filtered product list at the address
        if filtered_products.is_empty() {
            self.context
                .delete_state_entries(&[address])
                .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        } else {
            let product_list = ProductListBuilder::new()
                .with_products(filtered_products)
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
        }

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

    pub fn get_schema(&self, name: &str) -> Result<Option<Schema>, ApplyError> {
        let address = compute_schema_address(name);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let schemas = match SchemaList::from_bytes(packed.as_slice()) {
                    Ok(schemas) => schemas,
                    Err(err) => {
                        return Err(ApplyError::InvalidTransaction(format!(
                            "Cannot deserialize schema list: {:?}",
                            err,
                        )));
                    }
                };

                // find the schema with the correct name
                for schema in schemas.schemas() {
                    if schema.name() == name {
                        return Ok(Some(schema.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::collections::HashMap;

    use grid_sdk::protocol::pike::state::{AgentBuilder, AgentListBuilder};
    use grid_sdk::protocol::product::state::{ProductBuilder, ProductNamespace};
    use grid_sdk::protocol::schema::state::{DataType, PropertyValue, PropertyValueBuilder};

    use sawtooth_sdk::processor::handler::{ContextError, TransactionContext};

    const PRODUCT_ID: &str = "688955434684";

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
        let state = ProductState::new(&mut transaction_context);

        let result = state.get_agent("agent_public_key").unwrap();
        assert!(result.is_none())
    }

    #[test]
    // Test that if an agent exist in state, Some(agent) is returned
    fn test_get_agent_some() {
        let mut transaction_context = MockTransactionContext::default();
        transaction_context.add_agent("agent_public_key");
        let state = ProductState::new(&mut transaction_context);
        let result = state.get_agent("agent_public_key").unwrap();
        assert_eq!(result, Some(make_agent("agent_public_key")))
    }

    #[test]
    // Test that if a product does not exist in state, None is returned
    fn test_get_product_none() {
        let mut transaction_context = MockTransactionContext::default();
        let state = ProductState::new(&mut transaction_context);

        let result = state.get_product("not_a_product").unwrap();
        assert!(result.is_none())
    }

    #[test]
    // Test that a product can be added to state
    fn test_set_product() {
        let mut transaction_context = MockTransactionContext::default();
        let state = ProductState::new(&mut transaction_context);

        assert!(state.set_product(PRODUCT_ID, make_product()).is_ok());
        let result = state.get_product(PRODUCT_ID).unwrap();
        assert_eq!(result, Some(make_product()));
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

    fn make_product() -> Product {
        ProductBuilder::new()
            .with_product_id(PRODUCT_ID.to_string())
            .with_owner("some_owner".to_string())
            .with_product_namespace(ProductNamespace::GS1)
            .with_properties(make_properties())
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
}
