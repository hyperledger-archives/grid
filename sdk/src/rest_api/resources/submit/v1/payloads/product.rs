// Copyright 2018-2021 Cargill Incorporated
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

use crate::protos;
use crate::protos::{product_payload, product_payload::ProductPayload_Action};
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};

use super::{BuilderError, PropertyValue};

#[derive(Clone, Debug, Deserialize, PartialEq)]
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

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ProductAction {
    ProductCreate(ProductCreateAction),
    ProductUpdate(ProductUpdateAction),
    ProductDelete(ProductDeleteAction),
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ProductPayload {
    action: ProductAction,
    timestamp: u64,
}

impl ProductPayload {
    pub fn new(timestamp: u64, action: ProductAction) -> Self {
        Self { timestamp, action }
    }

    pub fn action(&self) -> &ProductAction {
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
            ProductPayload_Action::PRODUCT_CREATE => ProductAction::ProductCreate(
                ProductCreateAction::from_proto(payload.get_product_create().clone())?,
            ),
            ProductPayload_Action::PRODUCT_UPDATE => ProductAction::ProductUpdate(
                ProductUpdateAction::from_proto(payload.get_product_update().clone())?,
            ),
            ProductPayload_Action::PRODUCT_DELETE => ProductAction::ProductDelete(
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
            ProductAction::ProductCreate(payload) => {
                proto.set_action(ProductPayload_Action::PRODUCT_CREATE);
                proto.set_product_create(payload.clone().into_proto()?);
            }
            ProductAction::ProductUpdate(payload) => {
                proto.set_action(ProductPayload_Action::PRODUCT_UPDATE);
                proto.set_product_update(payload.clone().into_proto()?);
            }
            ProductAction::ProductDelete(payload) => {
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
            Message::parse_from_bytes(bytes).map_err(|_| {
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

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct ProductCreateAction {
    product_namespace: ProductNamespace,
    product_id: String,
    owner: String,
    properties: Vec<PropertyValue>,
}

impl ProductCreateAction {
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

impl FromProto<product_payload::ProductCreateAction> for ProductCreateAction {
    fn from_proto(
        proto: product_payload::ProductCreateAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(ProductCreateAction {
            product_namespace: ProductNamespace::from_proto(proto.get_product_namespace())?,
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
        proto.set_product_namespace(native.product_namespace().clone().into_proto()?);
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
        let proto: protos::product_payload::ProductCreateAction = Message::parse_from_bytes(bytes)
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
    product_namespace: Option<ProductNamespace>,
    product_id: Option<String>,
    owner: Option<String>,
    properties: Option<Vec<PropertyValue>>,
}

impl ProductCreateActionBuilder {
    pub fn new() -> Self {
        ProductCreateActionBuilder::default()
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
    pub fn build(self) -> Result<ProductCreateAction, BuilderError> {
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
        Ok(ProductCreateAction {
            product_namespace,
            product_id,
            owner,
            properties,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct ProductUpdateAction {
    product_namespace: ProductNamespace,
    product_id: String,
    properties: Vec<PropertyValue>,
}

impl ProductUpdateAction {
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

impl FromProto<protos::product_payload::ProductUpdateAction> for ProductUpdateAction {
    fn from_proto(
        proto: protos::product_payload::ProductUpdateAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(ProductUpdateAction {
            product_namespace: ProductNamespace::from_proto(proto.get_product_namespace())?,
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
        proto.set_product_namespace(native.product_namespace().clone().into_proto()?);
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
        let proto: protos::product_payload::ProductUpdateAction = Message::parse_from_bytes(bytes)
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

#[derive(Default, Clone)]
pub struct ProductUpdateActionBuilder {
    product_namespace: Option<ProductNamespace>,
    product_id: Option<String>,
    properties: Vec<PropertyValue>,
}

impl ProductUpdateActionBuilder {
    pub fn new() -> Self {
        ProductUpdateActionBuilder::default()
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

    pub fn build(self) -> Result<ProductUpdateAction, BuilderError> {
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

        Ok(ProductUpdateAction {
            product_namespace,
            product_id,
            properties,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct ProductDeleteAction {
    product_namespace: ProductNamespace,
    product_id: String,
}

impl ProductDeleteAction {
    pub fn product_namespace(&self) -> &ProductNamespace {
        &self.product_namespace
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
            product_namespace: ProductNamespace::from_proto(proto.get_product_namespace())?,
            product_id: proto.get_product_id().to_string(),
        })
    }
}

impl FromNative<ProductDeleteAction> for protos::product_payload::ProductDeleteAction {
    fn from_native(native: ProductDeleteAction) -> Result<Self, ProtoConversionError> {
        let mut proto = protos::product_payload::ProductDeleteAction::new();
        proto.set_product_namespace(native.product_namespace().clone().into_proto()?);
        proto.set_product_id(native.product_id().to_string());
        Ok(proto)
    }
}

impl FromBytes<ProductDeleteAction> for ProductDeleteAction {
    fn from_bytes(bytes: &[u8]) -> Result<ProductDeleteAction, ProtoConversionError> {
        let proto: protos::product_payload::ProductDeleteAction = Message::parse_from_bytes(bytes)
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

#[derive(Default, Clone)]
pub struct ProductDeleteActionBuilder {
    product_namespace: Option<ProductNamespace>,
    product_id: Option<String>,
}

impl ProductDeleteActionBuilder {
    pub fn new() -> Self {
        ProductDeleteActionBuilder::default()
    }

    pub fn with_product_namespace(mut self, product_namespace: ProductNamespace) -> Self {
        self.product_namespace = Some(product_namespace);
        self
    }

    pub fn with_product_id(mut self, product_id: String) -> Self {
        self.product_id = Some(product_id);
        self
    }

    pub fn build(self) -> Result<ProductDeleteAction, BuilderError> {
        let product_namespace = self.product_namespace.ok_or_else(|| {
            BuilderError::MissingField("'product_namespace' field is required".to_string())
        })?;

        let product_id = self.product_id.ok_or_else(|| {
            BuilderError::MissingField("'product_id' field is required".to_string())
        })?;

        Ok(ProductDeleteAction {
            product_namespace,
            product_id,
        })
    }
}
