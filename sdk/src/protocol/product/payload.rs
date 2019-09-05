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

use super::errors::BuilderError;

use crate::protocol::{product::state::ProductType, schema::state::PropertyValue};
use crate::protos;
use crate::protos::{product_payload, product_payload::ProductPayload_Action};
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};

/// Native implementation for ProductPayload_Action
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    ProductCreate(ProductCreateAction),
    ProductUpdate(ProductUpdateAction),
    ProductDelete(ProductDeleteAction),
}

// Rust native implementation for ProductPayload
#[derive(Debug, Clone, PartialEq)]
pub struct ProductPayload {
    action: Action,
    timestamp: u64,
}

impl ProductPayload {
    pub fn action(&self) -> &Action {
        &self.action
    }
    pub fn timestamp(&self) -> &u64 {
        &self.timestamp
    }
}

impl FromProto<protos::product_payload::ProductPayload> for ProductPayload {
    fn from_proto(
        payload: protos::product_payload::ProductPayload,
    ) -> Result<Self, ProtoConversionError> {
        let action = match payload.get_action() {
            ProductPayload_Action::PRODUCT_CREATE => Action::ProductCreate(
                ProductCreateAction::from_proto(payload.get_product_create().clone())?,
            ),
            ProductPayload_Action::PRODUCT_UPDATE => Action::ProductUpdate(
                ProductUpdateAction::from_proto(payload.get_product_update().clone())?,
            ),
            ProductPayload_Action::PRODUCT_DELETE => Action::ProductDelete(
                ProductDeleteAction::from_proto(payload.get_product_delete().clone())?,
            ),
            ProductPayload_Action::UNSET_ACTION => {
                return Err(ProtoConversionError::InvalidTypeError(
                    "Cannot convert ProductPayload_Action with type unset".to_string(),
                ));
            }
        };
        Ok(ProductPayload {
            action,
            timestamp: payload.get_timestamp(),
        })
    }
}

impl FromNative<ProductPayload> for protos::product_payload::ProductPayload {
    fn from_native(native: ProductPayload) -> Result<Self, ProtoConversionError> {
        let mut proto = product_payload::ProductPayload::new();

        proto.set_timestamp(*native.timestamp());

        match native.action() {
            Action::ProductCreate(payload) => {
                proto.set_action(ProductPayload_Action::PRODUCT_CREATE);
                proto.set_product_create(payload.clone().into_proto()?);
            }
            Action::ProductUpdate(payload) => {
                proto.set_action(ProductPayload_Action::PRODUCT_UPDATE);
                proto.set_product_update(payload.clone().into_proto()?);
            }
            Action::ProductDelete(payload) => {
                proto.set_action(ProductPayload_Action::PRODUCT_DELETE);
                proto.set_product_delete(payload.clone().into_proto()?);
            }
        }

        Ok(proto)
    }
}

impl FromBytes<ProductPayload> for ProductPayload {
    fn from_bytes(bytes: &[u8]) -> Result<ProductPayload, ProtoConversionError> {
        let proto: product_payload::ProductPayload =
            protobuf::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get ProductPayload from bytes".into(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for ProductPayload {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get ProductPayload from bytes".into(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::product_payload::ProductPayload> for ProductPayload {}
impl IntoNative<ProductPayload> for protos::product_payload::ProductPayload {}

#[derive(Debug)]
pub enum ProductPayloadBuildError {
    MissingField(String),
}

impl StdError for ProductPayloadBuildError {
    fn description(&self) -> &str {
        match *self {
            ProductPayloadBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            ProductPayloadBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for ProductPayloadBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ProductPayloadBuildError::MissingField(ref s) => write!(f, "missing field \"{}\"", s),
        }
    }
}

/// Builder used to create a ProductPayload
#[derive(Default, Clone)]
pub struct ProductPayloadBuilder {
    action: Option<Action>,
    timestamp: Option<u64>,
}

impl ProductPayloadBuilder {
    pub fn new() -> Self {
        ProductPayloadBuilder::default()
    }
    pub fn with_action(mut self, action: Action) -> Self {
        self.action = Some(action);
        self
    }
    pub fn with_timestamp(mut self, value: u64) -> Self {
        self.timestamp = Some(value);
        self
    }
    pub fn build(self) -> Result<ProductPayload, BuilderError> {
        let action = self
            .action
            .ok_or_else(|| BuilderError::MissingField("'action' field is required".into()))?;
        let timestamp = self
            .timestamp
            .ok_or_else(|| BuilderError::MissingField("'timestamp' field is required".into()))?;
        Ok(ProductPayload { action, timestamp })
    }
}

/// Native implementation for ProductCreateAction
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ProductCreateAction {
    product_type: ProductType,
    product_id: String,
    owner: String,
    properties: Vec<PropertyValue>,
}

impl ProductCreateAction {
    pub fn product_type(&self) -> &ProductType {
        &self.product_type
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

impl FromProto<product_payload::ProductCreateAction> for ProductCreateAction {
    fn from_proto(
        proto: product_payload::ProductCreateAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(ProductCreateAction {
            product_type: ProductType::from_proto(proto.get_product_type())?,
            product_id: proto.get_product_id().to_string(),
            owner: proto.get_owner().to_string(),
            properties: proto
                .get_properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::from_proto)
                .collect::<Result<Vec<PropertyValue>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<ProductCreateAction> for product_payload::ProductCreateAction {
    fn from_native(native: ProductCreateAction) -> Result<Self, ProtoConversionError> {
        let mut proto = protos::product_payload::ProductCreateAction::new();
        proto.set_product_type(native.product_type().clone().into_proto()?);
        proto.set_product_id(native.product_id().to_string());
        proto.set_owner(native.owner().to_string());
        proto.set_properties(RepeatedField::from_vec(
            native
                .properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::into_proto)
                .collect::<Result<Vec<protos::schema_state::PropertyValue>, ProtoConversionError>>(
                )?,
        ));
        Ok(proto)
    }
}

impl FromBytes<ProductCreateAction> for ProductCreateAction {
    fn from_bytes(bytes: &[u8]) -> Result<ProductCreateAction, ProtoConversionError> {
        let proto: protos::product_payload::ProductCreateAction = protobuf::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get ProductCreateAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for ProductCreateAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from ProductCreateAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::product_payload::ProductCreateAction> for ProductCreateAction {}
impl IntoNative<ProductCreateAction> for protos::product_payload::ProductCreateAction {}

#[derive(Default, Debug)]
pub struct ProductCreateActionBuilder {
    product_type: Option<ProductType>,
    product_id: Option<String>,
    owner: Option<String>,
    properties: Option<Vec<PropertyValue>>,
}

impl ProductCreateActionBuilder {
    pub fn new() -> Self {
        ProductCreateActionBuilder::default()
    }
    pub fn with_product_type(mut self, value: ProductType) -> Self {
        self.product_type = Some(value);
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
    pub fn build(self) -> Result<ProductCreateAction, BuilderError> {
        let product_type = self.product_type.ok_or_else(|| {
            BuilderError::MissingField("'product_type' field is required".to_string())
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
        Ok(ProductCreateAction {
            product_type,
            product_id,
            owner,
            properties,
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ProductUpdateAction {
    product_type: ProductType,
    product_id: String,
    properties: Vec<PropertyValue>,
}

/// Native implementation for ProductUpdateAction
impl ProductUpdateAction {
    pub fn product_type(&self) -> &ProductType {
        &self.product_type
    }

    pub fn product_id(&self) -> &str {
        &self.product_id
    }

    pub fn properties(&self) -> &[PropertyValue] {
        &self.properties
    }
}

impl FromProto<protos::product_payload::ProductUpdateAction> for ProductUpdateAction {
    fn from_proto(
        proto: protos::product_payload::ProductUpdateAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(ProductUpdateAction {
            product_type: ProductType::from_proto(proto.get_product_type())?,
            product_id: proto.get_product_id().to_string(),
            properties: proto
                .get_properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::from_proto)
                .collect::<Result<Vec<PropertyValue>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<ProductUpdateAction> for protos::product_payload::ProductUpdateAction {
    fn from_native(native: ProductUpdateAction) -> Result<Self, ProtoConversionError> {
        let mut proto = protos::product_payload::ProductUpdateAction::new();
        proto.set_product_type(native.product_type().clone().into_proto()?);
        proto.set_product_id(native.product_id().to_string());
        proto.set_properties(RepeatedField::from_vec(
            native
                .properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::into_proto)
                .collect::<Result<Vec<protos::schema_state::PropertyValue>, ProtoConversionError>>(
                )?,
        ));

        Ok(proto)
    }
}

impl FromBytes<ProductUpdateAction> for ProductUpdateAction {
    fn from_bytes(bytes: &[u8]) -> Result<ProductUpdateAction, ProtoConversionError> {
        let proto: protos::product_payload::ProductUpdateAction = protobuf::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get ProductUpdateAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for ProductUpdateAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from ProductUpdateAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::product_payload::ProductUpdateAction> for ProductUpdateAction {}
impl IntoNative<ProductUpdateAction> for protos::product_payload::ProductUpdateAction {}

/// Builder used to create a ProductUpdateAction
#[derive(Default, Clone)]
pub struct ProductUpdateActionBuilder {
    product_type: Option<ProductType>,
    product_id: Option<String>,
    properties: Vec<PropertyValue>,
}

impl ProductUpdateActionBuilder {
    pub fn new() -> Self {
        ProductUpdateActionBuilder::default()
    }

    pub fn with_product_type(mut self, product_type: ProductType) -> Self {
        self.product_type = Some(product_type);
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

    pub fn build(self) -> Result<ProductUpdateAction, BuilderError> {
        let product_type = self.product_type.ok_or_else(|| {
            BuilderError::MissingField("'product_type' field is required".to_string())
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

        Ok(ProductUpdateAction {
            product_type,
            product_id,
            properties,
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ProductDeleteAction {
    product_type: ProductType,
    product_id: String,
}

/// Native implementation for ProductDeleteAction
impl ProductDeleteAction {
    pub fn product_type(&self) -> &ProductType {
        &self.product_type
    }

    pub fn product_id(&self) -> &str {
        &self.product_id
    }
}

impl FromProto<protos::product_payload::ProductDeleteAction> for ProductDeleteAction {
    fn from_proto(
        proto: protos::product_payload::ProductDeleteAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(ProductDeleteAction {
            product_type: ProductType::from_proto(proto.get_product_type())?,
            product_id: proto.get_product_id().to_string(),
        })
    }
}

impl FromNative<ProductDeleteAction> for protos::product_payload::ProductDeleteAction {
    fn from_native(native: ProductDeleteAction) -> Result<Self, ProtoConversionError> {
        let mut proto = protos::product_payload::ProductDeleteAction::new();
        proto.set_product_type(native.product_type().clone().into_proto()?);
        proto.set_product_id(native.product_id().to_string());
        Ok(proto)
    }
}

impl FromBytes<ProductDeleteAction> for ProductDeleteAction {
    fn from_bytes(bytes: &[u8]) -> Result<ProductDeleteAction, ProtoConversionError> {
        let proto: protos::product_payload::ProductDeleteAction = protobuf::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get ProductDeleteAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for ProductDeleteAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from ProductDeleteAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::product_payload::ProductDeleteAction> for ProductDeleteAction {}
impl IntoNative<ProductDeleteAction> for protos::product_payload::ProductDeleteAction {}

/// Builder used to create a ProductDeleteAction
#[derive(Default, Clone)]
pub struct ProductDeleteActionBuilder {
    product_type: Option<ProductType>,
    product_id: Option<String>,
}

impl ProductDeleteActionBuilder {
    pub fn new() -> Self {
        ProductDeleteActionBuilder::default()
    }

    pub fn with_product_type(mut self, product_type: ProductType) -> Self {
        self.product_type = Some(product_type);
        self
    }

    pub fn with_product_id(mut self, product_id: String) -> Self {
        self.product_id = Some(product_id);
        self
    }

    pub fn build(self) -> Result<ProductDeleteAction, BuilderError> {
        let product_type = self.product_type.ok_or_else(|| {
            BuilderError::MissingField("'product_type' field is required".to_string())
        })?;

        let product_id = self.product_id.ok_or_else(|| {
            BuilderError::MissingField("'product_id' field is required".to_string())
        })?;

        Ok(ProductDeleteAction {
            product_type,
            product_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::schema::state::{DataType, PropertyValueBuilder};
    use std::fmt::Debug;

    #[test]
    // Test that a product create action can be built correctly
    fn test_product_create_builder() {
        let action = ProductCreateActionBuilder::new()
            .with_product_id("688955434684".into()) // GTIN-12
            .with_product_type(ProductType::GS1)
            .with_owner("Target".into())
            .with_properties(make_properties())
            .build()
            .unwrap();

        assert_eq!(action.product_id(), "688955434684");
        assert_eq!(action.owner(), "Target");
        assert_eq!(*action.product_type(), ProductType::GS1);
        assert_eq!(action.properties()[0].name(), "description");
        assert_eq!(*action.properties()[0].data_type(), DataType::String);
        assert_eq!(
            action.properties()[0].string_value(),
            "This is a product description"
        );
        assert_eq!(action.properties()[1].name(), "price");
        assert_eq!(*action.properties()[1].data_type(), DataType::Number);
        assert_eq!(*action.properties()[1].number_value(), 3);
    }

    #[test]
    // Test that a product create action can be converted to bytes and back
    fn test_product_create_into_bytes() {
        let action = ProductCreateActionBuilder::new()
            .with_product_id("688955434684".into()) // GTIN-12
            .with_product_type(ProductType::GS1)
            .with_owner("Target".into())
            .with_properties(make_properties())
            .build()
            .unwrap();

        test_from_bytes(action, ProductCreateAction::from_bytes);
    }

    #[test]
    // Test that a product update action can be built correctly
    fn test_product_update_builder() {
        let action = ProductUpdateActionBuilder::new()
            .with_product_id("688955434684".into()) // GTIN-12
            .with_product_type(ProductType::GS1)
            .with_properties(make_properties())
            .build()
            .unwrap();

        assert_eq!(action.product_id(), "688955434684");
        assert_eq!(*action.product_type(), ProductType::GS1);
        assert_eq!(action.properties()[0].name(), "description");
        assert_eq!(*action.properties()[0].data_type(), DataType::String);
        assert_eq!(
            action.properties()[0].string_value(),
            "This is a product description"
        );
        assert_eq!(action.properties()[1].name(), "price");
        assert_eq!(*action.properties()[1].data_type(), DataType::Number);
        assert_eq!(*action.properties()[1].number_value(), 3);
    }

    #[test]
    // Test that a product update action can be converted to bytes and back
    fn test_product_update_into_bytes() {
        let action = ProductUpdateActionBuilder::new()
            .with_product_id("688955434684".into()) // GTIN-12
            .with_product_type(ProductType::GS1)
            .with_properties(make_properties())
            .build()
            .unwrap();

        test_from_bytes(action, ProductUpdateAction::from_bytes);
    }

    #[test]
    // Test that a product delete action can be built correctly
    fn test_product_delete_builder() {
        let action = ProductDeleteActionBuilder::new()
            .with_product_id("688955434684".into()) // GTIN-12
            .with_product_type(ProductType::GS1)
            .build()
            .unwrap();

        assert_eq!(action.product_id(), "688955434684");
        assert_eq!(*action.product_type(), ProductType::GS1);
    }

    #[test]
    // Test that a product delete action can be converted to bytes and back
    fn test_product_delete_into_bytes() {
        let action = ProductDeleteActionBuilder::new()
            .with_product_id("688955434684".into()) // GTIN-12
            .with_product_type(ProductType::GS1)
            .build()
            .unwrap();

        test_from_bytes(action, ProductDeleteAction::from_bytes);
    }

    #[test]
    // Test that a product payload can be built correctly
    fn test_product_payload_builder() {
        let action = ProductCreateActionBuilder::new()
            .with_product_id("688955434684".into()) // GTIN-12
            .with_product_type(ProductType::GS1)
            .with_owner("Target".into())
            .with_properties(make_properties())
            .build()
            .unwrap();

        let payload = ProductPayloadBuilder::new()
            .with_action(Action::ProductCreate(action.clone()))
            .with_timestamp(0)
            .build()
            .unwrap();

        assert_eq!(*payload.action(), Action::ProductCreate(action));
        assert_eq!(*payload.timestamp(), 0);
    }

    #[test]
    // Test that a product payload can be converted to bytes and back
    fn test_product_payload_bytes() {
        let action = ProductCreateActionBuilder::new()
            .with_product_id("688955434684".into()) // GTIN-12
            .with_product_type(ProductType::GS1)
            .with_owner("Target".into())
            .with_properties(make_properties())
            .build()
            .unwrap();

        let payload = ProductPayloadBuilder::new()
            .with_action(Action::ProductCreate(action.clone()))
            .with_timestamp(0)
            .build()
            .unwrap();

        test_from_bytes(payload, ProductPayload::from_bytes);
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
