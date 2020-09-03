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

use protobuf::Message;
use protobuf::RepeatedField;

use std::error::Error as StdError;

use crate::protos;
use crate::protos::schema_state;
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};

use crate::protocol::schema::state::PropertyValue;

/// Native implementation of ProductNamespace enum
#[derive(Debug, Clone, PartialEq)]
pub enum ProductNamespace {
    GS1,
}

impl Default for ProductNamespace {
    fn default() -> Self {
        ProductNamespace::GS1
    }
}

impl FromProto<protos::product_state::Product_ProductNamespace> for ProductNamespace {
    fn from_proto(
        product_namespace: protos::product_state::Product_ProductNamespace,
    ) -> Result<Self, ProtoConversionError> {
        match product_namespace {
            protos::product_state::Product_ProductNamespace::GS1 => Ok(ProductNamespace::GS1),
            protos::product_state::Product_ProductNamespace::UNSET_TYPE => {
                Err(ProtoConversionError::InvalidTypeError(
                    "Cannot convert Product_ProductNamespace with type UNSET_TYPE".to_string(),
                ))
            }
        }
    }
}

impl FromNative<ProductNamespace> for protos::product_state::Product_ProductNamespace {
    fn from_native(product_namespace: ProductNamespace) -> Result<Self, ProtoConversionError> {
        match product_namespace {
            ProductNamespace::GS1 => Ok(protos::product_state::Product_ProductNamespace::GS1),
        }
    }
}

impl IntoProto<protos::product_state::Product_ProductNamespace> for ProductNamespace {}
impl IntoNative<ProductNamespace> for protos::product_state::Product_ProductNamespace {}

/// Native implementation of Product
#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    product_id: String,
    product_namespace: ProductNamespace,
    owner: String,
    properties: Vec<PropertyValue>,
}

impl Product {
    pub fn product_id(&self) -> &str {
        &self.product_id
    }

    pub fn product_namespace(&self) -> &ProductNamespace {
        &self.product_namespace
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn properties(&self) -> &[PropertyValue] {
        &self.properties
    }

    pub fn into_builder(self) -> ProductBuilder {
        ProductBuilder::new()
            .with_product_id(self.product_id)
            .with_product_namespace(self.product_namespace)
            .with_owner(self.owner)
            .with_properties(self.properties)
    }
}

impl FromProto<protos::product_state::Product> for Product {
    fn from_proto(product: protos::product_state::Product) -> Result<Self, ProtoConversionError> {
        Ok(Product {
            product_id: product.get_product_id().to_string(),
            product_namespace: ProductNamespace::from_proto(product.get_product_namespace())?,
            owner: product.get_owner().to_string(),
            properties: product
                .get_properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::from_proto)
                .collect::<Result<Vec<PropertyValue>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<Product> for protos::product_state::Product {
    fn from_native(product: Product) -> Result<Self, ProtoConversionError> {
        let mut proto = protos::product_state::Product::new();
        proto.set_product_id(product.product_id().to_string());
        proto.set_product_namespace(product.product_namespace().clone().into_proto()?);
        proto.set_owner(product.owner().to_string());
        proto.set_properties(RepeatedField::from_vec(
            product
                .properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::into_proto)
                .collect::<Result<Vec<schema_state::PropertyValue>, ProtoConversionError>>()?,
        ));
        Ok(proto)
    }
}

impl FromBytes<Product> for Product {
    fn from_bytes(bytes: &[u8]) -> Result<Product, ProtoConversionError> {
        let proto: protos::product_state::Product =
            protobuf::parse_from_bytes(bytes).map_err(|_| {
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
            ProtoConversionError::SerializationError("Unable to get bytes from Product".to_string())
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

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            ProductBuildError::MissingField(_) => None,
            ProductBuildError::EmptyVec(_) => None,
        }
    }
}

impl std::fmt::Display for ProductBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ProductBuildError::MissingField(ref s) => write!(f, "missing field \"{}\"", s),
            ProductBuildError::EmptyVec(ref s) => write!(f, "\"{}\" must not be empty", s),
        }
    }
}

/// Builder used to create a Product
#[derive(Default, Clone, PartialEq)]
pub struct ProductBuilder {
    pub product_id: Option<String>,
    pub product_namespace: Option<ProductNamespace>,
    pub owner: Option<String>,
    pub properties: Option<Vec<PropertyValue>>,
}

impl ProductBuilder {
    pub fn new() -> Self {
        ProductBuilder::default()
    }

    pub fn with_product_id(mut self, product_id: String) -> Self {
        self.product_id = Some(product_id);
        self
    }

    pub fn with_product_namespace(mut self, product_namespace: ProductNamespace) -> Self {
        self.product_namespace = Some(product_namespace);
        self
    }

    pub fn with_owner(mut self, owner: String) -> Self {
        self.owner = Some(owner);
        self
    }

    pub fn with_properties(mut self, properties: Vec<PropertyValue>) -> Self {
        self.properties = Some(properties);
        self
    }

    pub fn build(self) -> Result<Product, ProductBuildError> {
        let product_id = self.product_id.ok_or_else(|| {
            ProductBuildError::MissingField("'product_id' field is required".to_string())
        })?;

        let product_namespace = self.product_namespace.ok_or_else(|| {
            ProductBuildError::MissingField("'product_namespace' field is required".to_string())
        })?;

        let owner = self.owner.ok_or_else(|| {
            ProductBuildError::MissingField("'owner' field is required".to_string())
        })?;

        // Product values are not required
        let properties = self.properties.ok_or_else(|| {
            ProductBuildError::MissingField("'properties' field is required".to_string())
        })?;

        Ok(Product {
            product_id,
            product_namespace,
            owner,
            properties,
        })
    }
}

/// Native implementation of ProductList
#[derive(Debug, Clone, PartialEq)]
pub struct ProductList {
    products: Vec<Product>,
}

impl ProductList {
    pub fn products(&self) -> &[Product] {
        &self.products
    }

    pub fn into_builder(self) -> ProductListBuilder {
        ProductListBuilder::new().with_products(self.products)
    }
}

impl FromProto<protos::product_state::ProductList> for ProductList {
    fn from_proto(
        product_list: protos::product_state::ProductList,
    ) -> Result<Self, ProtoConversionError> {
        Ok(ProductList {
            products: product_list
                .get_entries()
                .to_vec()
                .into_iter()
                .map(Product::from_proto)
                .collect::<Result<Vec<Product>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<ProductList> for protos::product_state::ProductList {
    fn from_native(product_list: ProductList) -> Result<Self, ProtoConversionError> {
        let mut product_list_proto = protos::product_state::ProductList::new();

        product_list_proto.set_entries(RepeatedField::from_vec(
            product_list
                .products()
                .to_vec()
                .into_iter()
                .map(Product::into_proto)
                .collect::<Result<Vec<protos::product_state::Product>, ProtoConversionError>>()?,
        ));

        Ok(product_list_proto)
    }
}

impl FromBytes<ProductList> for ProductList {
    fn from_bytes(bytes: &[u8]) -> Result<ProductList, ProtoConversionError> {
        let proto: protos::product_state::ProductList =
            protobuf::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get ProductList from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for ProductList {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from ProductList".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::product_state::ProductList> for ProductList {}
impl IntoNative<ProductList> for protos::product_state::ProductList {}

#[derive(Debug)]
pub enum ProductListBuildError {
    MissingField(String),
}

impl StdError for ProductListBuildError {
    fn description(&self) -> &str {
        match *self {
            ProductListBuildError::MissingField(ref msg) => msg,
        }
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            ProductListBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for ProductListBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ProductListBuildError::MissingField(ref s) => write!(f, "missing field \"{}\"", s),
        }
    }
}

/// Builder used to create a ProductList
#[derive(Default, Clone)]
pub struct ProductListBuilder {
    pub products: Option<Vec<Product>>,
}

impl ProductListBuilder {
    pub fn new() -> Self {
        ProductListBuilder::default()
    }

    pub fn with_products(mut self, products: Vec<Product>) -> ProductListBuilder {
        self.products = Some(products);
        self
    }

    pub fn build(self) -> Result<ProductList, ProductListBuildError> {
        // Product values are not required
        let products = self.products.ok_or_else(|| {
            ProductListBuildError::MissingField("'products' field is required".to_string())
        })?;

        let products = {
            if products.is_empty() {
                return Err(ProductListBuildError::MissingField(
                    "'products' cannot be empty".to_string(),
                ));
            } else {
                products
            }
        };

        Ok(ProductList { products })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::schema::state::{DataType, PropertyValueBuilder};
    use std::fmt::Debug;

    #[test]
    // Test that a product can be built correctly
    fn test_product_builder() {
        let product = build_product();

        assert_eq!(product.product_id(), "688955434684");
        assert_eq!(*product.product_namespace(), ProductNamespace::GS1);
        assert_eq!(product.owner(), "Target");
        assert_eq!(product.properties()[0].name(), "description");
        assert_eq!(*product.properties()[0].data_type(), DataType::String);
        assert_eq!(
            product.properties()[0].string_value(),
            "This is a product description"
        );
        assert_eq!(product.properties()[1].name(), "price");
        assert_eq!(*product.properties()[1].data_type(), DataType::Number);
        assert_eq!(*product.properties()[1].number_value(), 3);
    }

    #[test]
    // Test that a product can be converted to a product builder
    fn test_product_into_builder() {
        let product = build_product();

        let builder = product.into_builder();

        assert_eq!(builder.product_id, Some("688955434684".to_string()));
        assert_eq!(builder.product_namespace, Some(ProductNamespace::GS1));
        assert_eq!(builder.owner, Some("Target".to_string()));
        assert_eq!(builder.properties, Some(make_properties()));
    }

    #[test]
    // Test that a product can be converted to bytes and back
    fn test_product_into_bytes() {
        let builder = ProductBuilder::new();
        let original = builder
            .with_product_id("688955434684".into())
            .with_product_namespace(ProductNamespace::GS1)
            .with_owner("Target".into())
            .with_properties(make_properties())
            .build()
            .unwrap();

        test_from_bytes(original, Product::from_bytes);
    }

    #[test]
    // Test that a product list can be built correctly
    fn test_product_list_builder() {
        let product_list = build_product_list();

        assert_eq!(product_list.products.len(), 2);

        // Test product 1
        assert_eq!(product_list.products[0].product_id(), "688955434684");
        assert_eq!(
            *product_list.products[0].product_namespace(),
            ProductNamespace::GS1
        );
        assert_eq!(product_list.products[0].owner(), "Target");
        assert_eq!(
            product_list.products[0].properties()[0].name(),
            "description"
        );
        assert_eq!(
            *product_list.products[0].properties()[0].data_type(),
            DataType::String
        );
        assert_eq!(
            product_list.products[0].properties()[0].string_value(),
            "This is a product description"
        );
        assert_eq!(product_list.products[0].properties()[1].name(), "price");
        assert_eq!(
            *product_list.products[0].properties()[1].data_type(),
            DataType::Number
        );
        assert_eq!(*product_list.products[0].properties()[1].number_value(), 3);

        // Test product 2
        assert_eq!(product_list.products[1].product_id(), "688955434685");
        assert_eq!(
            *product_list.products[1].product_namespace(),
            ProductNamespace::GS1
        );
        assert_eq!(product_list.products[1].owner(), "Cargill");
        assert_eq!(
            product_list.products[1].properties()[0].name(),
            "description"
        );
        assert_eq!(
            *product_list.products[1].properties()[0].data_type(),
            DataType::String
        );
        assert_eq!(
            product_list.products[1].properties()[0].string_value(),
            "This is a product description"
        );
        assert_eq!(product_list.products[1].properties()[1].name(), "price");
        assert_eq!(
            *product_list.products[1].properties()[1].data_type(),
            DataType::Number
        );
        assert_eq!(*product_list.products[1].properties()[1].number_value(), 3);
    }

    #[test]
    // Test that a product list can be converted to a product list builder
    fn test_product_list_into_builder() {
        let product_list = build_product_list();

        let builder = product_list.into_builder();

        assert_eq!(builder.products, Some(make_products()));
    }

    #[test]
    // Test that a product list can be converted to bytes and back
    fn test_product_list_into_bytes() {
        let builder = ProductListBuilder::new();
        let original = builder.with_products(make_products()).build().unwrap();

        test_from_bytes(original, ProductList::from_bytes);
    }

    fn build_product() -> Product {
        ProductBuilder::new()
            .with_product_id("688955434684".into()) // GTIN-12
            .with_product_namespace(ProductNamespace::GS1)
            .with_owner("Target".into())
            .with_properties(make_properties())
            .build()
            .expect("Failed to build test product")
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

    fn build_product_list() -> ProductList {
        ProductListBuilder::new()
            .with_products(make_products())
            .build()
            .expect("Failed to build test product list")
    }

    fn make_products() -> Vec<Product> {
        vec![
            ProductBuilder::new()
                .with_product_id("688955434684".into()) // GTIN-12
                .with_product_namespace(ProductNamespace::GS1)
                .with_owner("Target".into())
                .with_properties(make_properties())
                .build()
                .expect("Failed to build test product"),
            ProductBuilder::new()
                .with_product_id("688955434685".into()) // GTIN-12
                .with_product_namespace(ProductNamespace::GS1)
                .with_owner("Cargill".into())
                .with_properties(make_properties())
                .build()
                .expect("Failed to build test product"),
        ]
    }

    fn test_from_bytes<T: FromBytes<T> + Clone + PartialEq + IntoBytes + Debug, F>(
        under_test: T,
        from_bytes: F,
    ) where
        F: Fn(&[u8]) -> Result<T, ProtoConversionError>,
    {
        let bytes = under_test.clone().into_bytes().unwrap();
        let created_from_bytes = from_bytes(&bytes).unwrap();
        assert_eq!(under_test, created_from_bytes);
    }
}
