// Copyright 2018-2022 Cargill Incorporated
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

use crate::rest_api::resources::submit::v2::error::BuilderError;

use super::{PropertyValue, TransactionPayload};

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum ProductNamespace {
    #[serde(rename = "GS1")]
    Gs1,
}

impl Default for ProductNamespace {
    fn default() -> Self {
        ProductNamespace::Gs1
    }
}

// Allow the enum variants to end in the same `Product` postfix
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ProductAction {
    CreateProduct(CreateProductAction),
    UpdateProduct(UpdateProductAction),
    DeleteProduct(DeleteProductAction),
}

impl ProductAction {
    pub fn into_inner(self) -> Box<dyn TransactionPayload> {
        unimplemented!();
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ProductPayload {
    action: ProductAction,
    timestamp: u64,
}

impl ProductPayload {
    pub fn new(timestamp: u64, action: ProductAction) -> Self {
        Self { action, timestamp }
    }

    pub fn action(&self) -> &ProductAction {
        &self.action
    }
    pub fn timestamp(&self) -> &u64 {
        &self.timestamp
    }

    pub fn into_transaction_payload(self) -> Box<dyn TransactionPayload> {
        self.action.into_inner()
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct CreateProductAction {
    product_namespace: ProductNamespace,
    product_id: String,
    owner: String,
    properties: Vec<PropertyValue>,
}

impl CreateProductAction {
    pub fn product_namespace(&self) -> &ProductNamespace {
        &self.product_namespace
    }

    pub fn product_id(&self) -> &str {
        &self.product_id
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn properties(&self) -> &[PropertyValue] {
        &self.properties
    }
}

#[derive(Default, Debug)]
pub struct CreateProductActionBuilder {
    product_namespace: Option<ProductNamespace>,
    product_id: Option<String>,
    owner: Option<String>,
    properties: Option<Vec<PropertyValue>>,
}

impl CreateProductActionBuilder {
    pub fn new() -> Self {
        CreateProductActionBuilder::default()
    }
    pub fn with_product_namespace(mut self, value: ProductNamespace) -> Self {
        self.product_namespace = Some(value);
        self
    }
    pub fn with_product_id(mut self, value: String) -> Self {
        self.product_id = Some(value);
        self
    }
    pub fn with_owner(mut self, value: String) -> Self {
        self.owner = Some(value);
        self
    }
    pub fn with_properties(mut self, value: Vec<PropertyValue>) -> Self {
        self.properties = Some(value);
        self
    }
    pub fn build(self) -> Result<CreateProductAction, BuilderError> {
        let product_namespace = self.product_namespace.ok_or_else(|| {
            BuilderError::MissingField("'product_namespace' field is required".to_string())
        })?;
        let product_id = self
            .product_id
            .ok_or_else(|| BuilderError::MissingField("'product_id' field is required".into()))?;
        let owner = self
            .owner
            .ok_or_else(|| BuilderError::MissingField("'owner' field is required".into()))?;
        let properties = self
            .properties
            .ok_or_else(|| BuilderError::MissingField("'properties' field is required".into()))?;
        Ok(CreateProductAction {
            product_namespace,
            product_id,
            owner,
            properties,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct UpdateProductAction {
    product_namespace: ProductNamespace,
    product_id: String,
    properties: Vec<PropertyValue>,
}

impl UpdateProductAction {
    pub fn product_namespace(&self) -> &ProductNamespace {
        &self.product_namespace
    }

    pub fn product_id(&self) -> &str {
        &self.product_id
    }

    pub fn properties(&self) -> &[PropertyValue] {
        &self.properties
    }
}

#[derive(Default, Clone)]
pub struct UpdateProductActionBuilder {
    product_namespace: Option<ProductNamespace>,
    product_id: Option<String>,
    properties: Vec<PropertyValue>,
}

impl UpdateProductActionBuilder {
    pub fn new() -> Self {
        UpdateProductActionBuilder::default()
    }

    pub fn with_product_namespace(mut self, product_namespace: ProductNamespace) -> Self {
        self.product_namespace = Some(product_namespace);
        self
    }

    pub fn with_product_id(mut self, product_id: String) -> Self {
        self.product_id = Some(product_id);
        self
    }

    pub fn with_properties(mut self, properties: Vec<PropertyValue>) -> Self {
        self.properties = properties;
        self
    }

    pub fn build(self) -> Result<UpdateProductAction, BuilderError> {
        let product_namespace = self.product_namespace.ok_or_else(|| {
            BuilderError::MissingField("'product_namespace' field is required".to_string())
        })?;

        let product_id = self.product_id.ok_or_else(|| {
            BuilderError::MissingField("'product_id' field is required".to_string())
        })?;

        let properties = {
            if !self.properties.is_empty() {
                self.properties
            } else {
                return Err(BuilderError::MissingField(
                    "'properties' field is required".to_string(),
                ));
            }
        };

        Ok(UpdateProductAction {
            product_namespace,
            product_id,
            properties,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct DeleteProductAction {
    product_namespace: ProductNamespace,
    product_id: String,
}

impl DeleteProductAction {
    pub fn product_namespace(&self) -> &ProductNamespace {
        &self.product_namespace
    }

    pub fn product_id(&self) -> &str {
        &self.product_id
    }
}

#[derive(Default, Clone)]
pub struct DeleteProductActionBuilder {
    product_namespace: Option<ProductNamespace>,
    product_id: Option<String>,
}

impl DeleteProductActionBuilder {
    pub fn new() -> Self {
        DeleteProductActionBuilder::default()
    }

    pub fn with_product_namespace(mut self, product_namespace: ProductNamespace) -> Self {
        self.product_namespace = Some(product_namespace);
        self
    }

    pub fn with_product_id(mut self, product_id: String) -> Self {
        self.product_id = Some(product_id);
        self
    }

    pub fn build(self) -> Result<DeleteProductAction, BuilderError> {
        let product_namespace = self.product_namespace.ok_or_else(|| {
            BuilderError::MissingField("'product_namespace' field is required".to_string())
        })?;

        let product_id = self.product_id.ok_or_else(|| {
            BuilderError::MissingField("'product_id' field is required".to_string())
        })?;

        Ok(DeleteProductAction {
            product_namespace,
            product_id,
        })
    }
}
