/*
 * Copyright (c) 2019 Target Brands, Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use protobuf::Message;
use protobuf::RepeatedField;

use std::error::Error as StdError;

use crate::protos;
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};
use crate::protocol::schema{
    PropertyValue,
}

/// Native implementation of ProductType enum
#[derive(Debug, Clone, PartialEq)]
pub enum ProductType {
    GS1,
}

/// Native implementation of Product
#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    identifier: String,
    product_type: ProductType,
    owner: String,
    product_values: Vec<PropertyValue>
}

impl Product {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

     pub fn product_type(&self) -> &ProductType {
        &self.product_type
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn product_values(&self) -> &[PropertyValue] {
        &self.product_values
    }
}

impl FromProto<protos::product_state::Product> for Product {
    fn from_proto(
        product: protos::product_state::Product,
    ) -> Result<Self, ProtoConversionError> {
        Ok(Product {
            identifier: product.get_identifier().to_string(),
            product_type: ProductType::from_proto(product.get_product_type())?,
            owner: product.get_owner().to_string(),
            product_values: product
                .get_product_values()
                .to_vec()
                .into_iter()
                .map(PropertyValue::from_proto)
                .collect::<Result<Vec<PropertyValue>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<Product> for protos::product_state::Product {
    fn from_native(product: Product) -> Result<Self, ProtoConversionError> {
        let mut proto_product = protos::product_state::Product::new();
        proto_product.set_identifier(product.identifier().to_string());
        proto_product
            .set_product_type(product.product_type().clone().into_proto()?);
        proto_product.set_owner(product.owner().to_string());
        proto_product.set_product_values(
            RepeatedField::from_vec(
                product.product_values().to_vec().into_iter()
                .map(PropertyValue::into_proto)
                .collect::<Result<Vec<PropertyValue>, ProtoConversionError>>()?,));
        Ok(proto_product)
    }
}

impl FromBytes<Product> for Product {
    fn from_bytes(bytes: &[u8]) -> Result<Product, ProtoConversionError> {
        let proto: protos::product_state::Product = protobuf::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get Product from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for Product {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from Product".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::product_state::Product> for Product {}
impl IntoNative<Product> for protos::product_state::Product {}

#[derive(Debug)]
pub enum ProductBuildError {
    MissingField(String),
    EmptyVec(String),
}

impl StdError for ProductBuildError {
    fn description(&self) -> &str {
        match *self {
            ProductBuildError::MissingField(ref msg) => msg,
            ProductBuildError::EmptyVec(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            ProductBuildError::MissingField(_) => None,
            ProductBuildError::EmptyVec(_) => None,
        }
    }
}

impl std::fmt::Display for ProductBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ProductBuildError::MissingField(ref s) => write!(f, "MissingField: {}", s),
            ProductBuildError::EmptyVec(ref s) => write!(f, "EmptyVec: {}", s),
        }
    }
}

/// Builder used to create a Product
#[derive(Default, Clone, PartialEq)]
pub struct ProductBuilder {
    pub identifier: Option<String>,
    pub product_type: Option<ProductType>,
    pub owner: Option<String>,
    pub product_values: Vec<PropertyValue>
}

impl ProductBuilder {
    pub fn new() -> Self {
        ProductBuilder::default()
    }

    pub fn with_identifier(mut self, identifier: String) -> ProductBuilder {
        self.identifier = Some(identifier);
        self
    }

    pub fn with_product_type(mut self, product_type: ProductType) -> ProductBuilder {
        self.product_type = Some(product_type);
        self
    }

    pub fn with_owner(mut self, owner: String) -> ProductBuilder {
        self.owner = Some(owner);
        self
    }

    pub fn with_product_values(
        mut self,
        product_values: Vec<PropertyValue>,
    ) -> ProductBuilder {
        self.product_values = product_values;
        self
    }

    pub fn build(self) -> Result<Product, ProductBuildError> {
        let identifier = self.identifier.ok_or_else(|| {
            ProductBuildError::MissingField("'identifier' field is required".to_string())
        })?;

        let product_type = self.product_type.ok_or_else(|| {
            ProductBuildError::MissingField("'product_type' field is required".to_string())
        })?;

        let owner = self.owner.ok_or_else(|| {
            ProductBuildError::MissingField("'owner' field is required".to_string())
        })?;

        let product_values = {
            if !self.product_values.is_empty() {
                self.product_values
            } else {
                return Err(ProductBuildError::EmptyVec(
                    "'product_values' cannot be empty".to_string(),
                ));
            }
        };

        Ok(Product {
            identifier,
            product_type,
            owner,
            product_values,
        })
    }
}
