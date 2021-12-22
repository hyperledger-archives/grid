// Copyright 2021 Cargill Incorporated
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

//! Protocol structs for Purchase Order transaction payloads

use protobuf::{Message, RepeatedField};

use crate::protocol::errors::BuilderError;
use crate::protocol::purchase_order::state::PurchaseOrderAlternateId;
use crate::protos;
use crate::protos::purchase_order_payload;
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};

/// The Purchase Order payload's action envelope
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    CreatePo(CreatePurchaseOrderPayload),
    UpdatePo(UpdatePurchaseOrderPayload),
    CreateVersion(CreateVersionPayload),
    UpdateVersion(UpdateVersionPayload),
}

/// Native representation of a Purchase Order payload
#[derive(Debug, Clone, PartialEq)]
pub struct PurchaseOrderPayload {
    action: Action,
    timestamp: u64,
}

impl PurchaseOrderPayload {
    pub fn action(&self) -> &Action {
        &self.action
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl FromProto<purchase_order_payload::PurchaseOrderPayload> for PurchaseOrderPayload {
    fn from_proto(
        mut payload: purchase_order_payload::PurchaseOrderPayload,
    ) -> Result<Self, ProtoConversionError> {
        let action = match payload.get_action() {
            purchase_order_payload::PurchaseOrderPayload_Action::CREATE_PO => Action::CreatePo(
                CreatePurchaseOrderPayload::from_proto(payload.take_create_po_payload())?,
            ),
            purchase_order_payload::PurchaseOrderPayload_Action::UPDATE_PO => Action::UpdatePo(
                UpdatePurchaseOrderPayload::from_proto(payload.take_update_po_payload())?,
            ),
            purchase_order_payload::PurchaseOrderPayload_Action::CREATE_VERSION => {
                Action::CreateVersion(CreateVersionPayload::from_proto(
                    payload.take_create_version_payload(),
                )?)
            }
            purchase_order_payload::PurchaseOrderPayload_Action::UPDATE_VERSION => {
                Action::UpdateVersion(UpdateVersionPayload::from_proto(
                    payload.take_update_version_payload(),
                )?)
            }
            purchase_order_payload::PurchaseOrderPayload_Action::UNSET_ACTION => {
                return Err(ProtoConversionError::InvalidTypeError(
                    "Cannot convert PurchaseOrderPayload_Action with type unset".to_string(),
                ));
            }
        };
        Ok(PurchaseOrderPayload {
            action,
            timestamp: payload.get_timestamp(),
        })
    }
}

impl FromNative<PurchaseOrderPayload> for purchase_order_payload::PurchaseOrderPayload {
    fn from_native(native: PurchaseOrderPayload) -> Result<Self, ProtoConversionError> {
        let mut proto = purchase_order_payload::PurchaseOrderPayload::new();
        proto.set_timestamp(native.timestamp());

        match native.action() {
            Action::CreatePo(payload) => {
                proto.set_action(purchase_order_payload::PurchaseOrderPayload_Action::CREATE_PO);
                proto.set_create_po_payload(payload.clone().into_proto()?);
            }
            Action::UpdatePo(payload) => {
                proto.set_action(purchase_order_payload::PurchaseOrderPayload_Action::UPDATE_PO);
                proto.set_update_po_payload(payload.clone().into_proto()?);
            }
            Action::CreateVersion(payload) => {
                proto.set_action(
                    purchase_order_payload::PurchaseOrderPayload_Action::CREATE_VERSION,
                );
                proto.set_create_version_payload(payload.clone().into_proto()?);
            }
            Action::UpdateVersion(payload) => {
                proto.set_action(
                    purchase_order_payload::PurchaseOrderPayload_Action::UPDATE_VERSION,
                );
                proto.set_update_version_payload(payload.clone().into_proto()?);
            }
        }

        Ok(proto)
    }
}

impl FromBytes<PurchaseOrderPayload> for PurchaseOrderPayload {
    fn from_bytes(bytes: &[u8]) -> Result<PurchaseOrderPayload, ProtoConversionError> {
        let proto: purchase_order_payload::PurchaseOrderPayload = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get PurchaseOrderPayload from bytes".into(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for PurchaseOrderPayload {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from PurchaseOrderPayload".into(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<purchase_order_payload::PurchaseOrderPayload> for PurchaseOrderPayload {}
impl IntoNative<PurchaseOrderPayload> for purchase_order_payload::PurchaseOrderPayload {}

/// Builder used to create a Purchase Order payload
#[derive(Default, Clone)]
pub struct PurchaseOrderPayloadBuilder {
    action: Option<Action>,
    timestamp: Option<u64>,
}

impl PurchaseOrderPayloadBuilder {
    pub fn new() -> Self {
        PurchaseOrderPayloadBuilder::default()
    }

    pub fn with_action(mut self, action: Action) -> Self {
        self.action = Some(action);
        self
    }

    pub fn with_timestamp(mut self, value: u64) -> Self {
        self.timestamp = Some(value);
        self
    }

    pub fn build(self) -> Result<PurchaseOrderPayload, BuilderError> {
        let action = self
            .action
            .ok_or_else(|| BuilderError::MissingField("'action' field is required".into()))?;

        let timestamp = self
            .timestamp
            .ok_or_else(|| BuilderError::MissingField("'timestamp' field is required".into()))?;

        Ok(PurchaseOrderPayload { action, timestamp })
    }
}

/// Native representation of the "create purchase order" payload
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CreatePurchaseOrderPayload {
    uid: String,
    created_at: u64,
    buyer_org_id: String,
    seller_org_id: String,
    workflow_state: String,
    alternate_ids: Vec<PurchaseOrderAlternateId>,
    create_version_payload: Option<CreateVersionPayload>,
    workflow_id: String,
}

impl CreatePurchaseOrderPayload {
    pub fn uid(&self) -> &str {
        &self.uid
    }

    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    pub fn buyer_org_id(&self) -> &str {
        &self.buyer_org_id
    }

    pub fn seller_org_id(&self) -> &str {
        &self.seller_org_id
    }

    pub fn workflow_state(&self) -> &str {
        &self.workflow_state
    }

    pub fn alternate_ids(&self) -> &[PurchaseOrderAlternateId] {
        &self.alternate_ids
    }

    pub fn create_version_payload(&self) -> Option<CreateVersionPayload> {
        self.create_version_payload.clone()
    }

    pub fn workflow_id(&self) -> &str {
        &self.workflow_id
    }
}

impl FromProto<purchase_order_payload::CreatePurchaseOrderPayload> for CreatePurchaseOrderPayload {
    fn from_proto(
        mut proto: purchase_order_payload::CreatePurchaseOrderPayload,
    ) -> Result<Self, ProtoConversionError> {
        let create_version_payload = if proto
            .get_create_version_payload()
            .get_version_id()
            .is_empty()
        {
            None
        } else {
            Some(CreateVersionPayload::from_proto(
                proto.take_create_version_payload(),
            )?)
        };

        Ok(CreatePurchaseOrderPayload {
            uid: proto.take_uid(),
            created_at: proto.get_created_at(),
            buyer_org_id: proto.take_buyer_org_id(),
            seller_org_id: proto.take_seller_org_id(),
            workflow_state: proto.take_workflow_state(),
            alternate_ids: proto
                .get_alternate_ids()
                .to_vec()
                .into_iter()
                .map(PurchaseOrderAlternateId::from_proto)
                .collect::<Result<Vec<PurchaseOrderAlternateId>, ProtoConversionError>>()?,
            create_version_payload,
            workflow_id: proto.take_workflow_id(),
        })
    }
}

impl FromNative<CreatePurchaseOrderPayload> for purchase_order_payload::CreatePurchaseOrderPayload {
    fn from_native(native: CreatePurchaseOrderPayload) -> Result<Self, ProtoConversionError> {
        let mut proto = purchase_order_payload::CreatePurchaseOrderPayload::new();
        proto.set_uid(native.uid().to_string());
        proto.set_created_at(native.created_at());
        proto.set_buyer_org_id(native.buyer_org_id().to_string());
        proto.set_seller_org_id(native.seller_org_id().to_string());
        proto.set_alternate_ids(RepeatedField::from_vec(
            native
                .alternate_ids()
                .to_vec()
                .into_iter()
                .map(PurchaseOrderAlternateId::into_proto)
                .collect::<Result<
                    Vec<protos::purchase_order_state::PurchaseOrderAlternateId>,
                    ProtoConversionError,
                >>()?,
        ));
        proto.set_workflow_state(native.workflow_state().to_string());
        proto.set_workflow_id(native.workflow_id().to_string());

        if let Some(payload) = native.create_version_payload() {
            let proto_payload: purchase_order_payload::CreateVersionPayload =
                payload.into_proto()?;
            proto.set_create_version_payload(proto_payload);
        }

        Ok(proto)
    }
}

impl FromBytes<CreatePurchaseOrderPayload> for CreatePurchaseOrderPayload {
    fn from_bytes(bytes: &[u8]) -> Result<CreatePurchaseOrderPayload, ProtoConversionError> {
        let proto: purchase_order_payload::CreatePurchaseOrderPayload =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get CreatePurchaseOrderPayload from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for CreatePurchaseOrderPayload {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from CreatePurchaseOrderPayload".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<purchase_order_payload::CreatePurchaseOrderPayload> for CreatePurchaseOrderPayload {}
impl IntoNative<CreatePurchaseOrderPayload> for purchase_order_payload::CreatePurchaseOrderPayload {}

/// Builder used to create the "create purchase order" payload
#[derive(Default, Debug)]
pub struct CreatePurchaseOrderPayloadBuilder {
    uid: Option<String>,
    created_at: Option<u64>,
    buyer_org_id: Option<String>,
    seller_org_id: Option<String>,
    workflow_state: Option<String>,
    alternate_ids: Vec<PurchaseOrderAlternateId>,
    create_version_payload: Option<CreateVersionPayload>,
    workflow_id: Option<String>,
}

impl CreatePurchaseOrderPayloadBuilder {
    pub fn new() -> Self {
        CreatePurchaseOrderPayloadBuilder::default()
    }

    pub fn with_uid(mut self, value: String) -> Self {
        self.uid = Some(value);
        self
    }

    pub fn with_created_at(mut self, value: u64) -> Self {
        self.created_at = Some(value);
        self
    }

    pub fn with_buyer_org_id(mut self, value: String) -> Self {
        self.buyer_org_id = Some(value);
        self
    }

    pub fn with_seller_org_id(mut self, value: String) -> Self {
        self.seller_org_id = Some(value);
        self
    }

    pub fn with_workflow_state(mut self, value: String) -> Self {
        self.workflow_state = Some(value);
        self
    }

    pub fn with_alternate_ids(mut self, alternate_ids: Vec<PurchaseOrderAlternateId>) -> Self {
        self.alternate_ids = alternate_ids;
        self
    }

    pub fn with_create_version_payload(mut self, payload: CreateVersionPayload) -> Self {
        self.create_version_payload = Some(payload);
        self
    }

    pub fn with_workflow_id(mut self, value: String) -> Self {
        self.workflow_id = Some(value);
        self
    }

    pub fn build(self) -> Result<CreatePurchaseOrderPayload, BuilderError> {
        let uid = self
            .uid
            .ok_or_else(|| BuilderError::MissingField("'uid' field is required".to_string()))?;

        let created_at = self.created_at.ok_or_else(|| {
            BuilderError::MissingField("'created_at' field is required".to_string())
        })?;

        let buyer_org_id = self.buyer_org_id.ok_or_else(|| {
            BuilderError::MissingField("'buyer_org_id' field is required".to_string())
        })?;

        let seller_org_id = self.seller_org_id.ok_or_else(|| {
            BuilderError::MissingField("'seller_org_id' field is required".to_string())
        })?;

        let workflow_state = self.workflow_state.ok_or_else(|| {
            BuilderError::MissingField("'workflow_state' field is required".to_string())
        })?;

        let alternate_ids = self.alternate_ids;

        let create_version_payload = self.create_version_payload;

        let workflow_id = self.workflow_id.ok_or_else(|| {
            BuilderError::MissingField("'workflow_id' field is required".to_string())
        })?;

        Ok(CreatePurchaseOrderPayload {
            uid,
            created_at,
            buyer_org_id,
            seller_org_id,
            workflow_state,
            alternate_ids,
            create_version_payload,
            workflow_id,
        })
    }
}

/// Native representation of the "update purchase order" payload
#[derive(Debug, Default, Clone, PartialEq)]
pub struct UpdatePurchaseOrderPayload {
    uid: String,
    workflow_state: String,
    is_closed: bool,
    accepted_version_number: Option<String>,
    alternate_ids: Vec<PurchaseOrderAlternateId>,
}

impl UpdatePurchaseOrderPayload {
    pub fn uid(&self) -> &str {
        &self.uid
    }

    pub fn workflow_state(&self) -> &str {
        &self.workflow_state
    }

    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    pub fn accepted_version_number(&self) -> Option<&str> {
        self.accepted_version_number.as_deref()
    }

    pub fn alternate_ids(&self) -> &[PurchaseOrderAlternateId] {
        &self.alternate_ids
    }
}

impl FromProto<purchase_order_payload::UpdatePurchaseOrderPayload> for UpdatePurchaseOrderPayload {
    fn from_proto(
        mut proto: purchase_order_payload::UpdatePurchaseOrderPayload,
    ) -> Result<Self, ProtoConversionError> {
        Ok(UpdatePurchaseOrderPayload {
            uid: proto.take_po_uid(),
            workflow_state: proto.take_workflow_state(),
            is_closed: proto.get_is_closed(),
            accepted_version_number: match proto.get_accepted_version_number().is_empty() {
                false => Some(proto.take_accepted_version_number()),
                true => None,
            },
            alternate_ids: proto
                .get_alternate_ids()
                .to_vec()
                .into_iter()
                .map(PurchaseOrderAlternateId::from_proto)
                .collect::<Result<Vec<PurchaseOrderAlternateId>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<UpdatePurchaseOrderPayload> for purchase_order_payload::UpdatePurchaseOrderPayload {
    fn from_native(native: UpdatePurchaseOrderPayload) -> Result<Self, ProtoConversionError> {
        let mut proto = purchase_order_payload::UpdatePurchaseOrderPayload::new();
        proto.set_po_uid(native.uid().to_string());
        proto.set_workflow_state(native.workflow_state().to_string());
        proto.set_is_closed(native.is_closed());
        proto.set_accepted_version_number(
            native.accepted_version_number().unwrap_or("").to_string(),
        );
        proto.set_alternate_ids(RepeatedField::from_vec(
            native
                .alternate_ids()
                .to_vec()
                .into_iter()
                .map(PurchaseOrderAlternateId::into_proto)
                .collect::<Result<
                    Vec<protos::purchase_order_state::PurchaseOrderAlternateId>,
                    ProtoConversionError,
                >>()?,
        ));
        Ok(proto)
    }
}

impl FromBytes<UpdatePurchaseOrderPayload> for UpdatePurchaseOrderPayload {
    fn from_bytes(bytes: &[u8]) -> Result<UpdatePurchaseOrderPayload, ProtoConversionError> {
        let proto: purchase_order_payload::UpdatePurchaseOrderPayload =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get UpdatePurchaseOrderPayload from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for UpdatePurchaseOrderPayload {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from UpdatePurchaseOrderPayload".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<purchase_order_payload::UpdatePurchaseOrderPayload> for UpdatePurchaseOrderPayload {}
impl IntoNative<UpdatePurchaseOrderPayload> for purchase_order_payload::UpdatePurchaseOrderPayload {}

/// Builder used to create the "update purchase order" payload
#[derive(Default, Debug)]
pub struct UpdatePurchaseOrderPayloadBuilder {
    uid: Option<String>,
    workflow_state: Option<String>,
    is_closed: Option<bool>,
    accepted_version_number: Option<String>,
    alternate_ids: Vec<PurchaseOrderAlternateId>,
}

impl UpdatePurchaseOrderPayloadBuilder {
    pub fn new() -> Self {
        UpdatePurchaseOrderPayloadBuilder::default()
    }

    pub fn with_uid(mut self, value: String) -> Self {
        self.uid = Some(value);
        self
    }

    pub fn with_workflow_state(mut self, value: String) -> Self {
        self.workflow_state = Some(value);
        self
    }

    pub fn with_is_closed(mut self, value: bool) -> Self {
        self.is_closed = Some(value);
        self
    }

    pub fn with_accepted_version_number(mut self, value: Option<String>) -> Self {
        self.accepted_version_number = value;
        self
    }

    pub fn with_alternate_ids(mut self, value: Vec<PurchaseOrderAlternateId>) -> Self {
        self.alternate_ids = value;
        self
    }

    pub fn build(self) -> Result<UpdatePurchaseOrderPayload, BuilderError> {
        let uid = self
            .uid
            .ok_or_else(|| BuilderError::MissingField("'uid' field is required".to_string()))?;

        let workflow_state = self.workflow_state.ok_or_else(|| {
            BuilderError::MissingField("'workflow_state' field is required".to_string())
        })?;

        let is_closed = self.is_closed.ok_or_else(|| {
            BuilderError::MissingField("'is_closed' field is required".to_string())
        })?;

        let alternate_ids = self.alternate_ids;

        Ok(UpdatePurchaseOrderPayload {
            uid,
            workflow_state,
            is_closed,
            accepted_version_number: self.accepted_version_number,
            alternate_ids,
        })
    }
}

/// Native representation of the revision made in a "create" or "update" version payload
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PayloadRevision {
    revision_id: u64,
    submitter: String,
    created_at: u64,
    order_xml_v3_4: String,
}

impl PayloadRevision {
    pub fn revision_id(&self) -> u64 {
        self.revision_id
    }

    pub fn submitter(&self) -> &str {
        &self.submitter
    }

    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    pub fn order_xml_v3_4(&self) -> &str {
        &self.order_xml_v3_4
    }
}

impl FromProto<purchase_order_payload::PayloadRevision> for PayloadRevision {
    fn from_proto(
        mut proto: purchase_order_payload::PayloadRevision,
    ) -> Result<Self, ProtoConversionError> {
        Ok(PayloadRevision {
            revision_id: proto.get_revision_id(),
            submitter: proto.take_submitter(),
            created_at: proto.get_created_at(),
            order_xml_v3_4: proto.take_order_xml_v3_4(),
        })
    }
}

impl FromNative<PayloadRevision> for purchase_order_payload::PayloadRevision {
    fn from_native(native: PayloadRevision) -> Result<Self, ProtoConversionError> {
        let mut proto = purchase_order_payload::PayloadRevision::new();
        proto.set_revision_id(native.revision_id());
        proto.set_submitter(native.submitter().to_string());
        proto.set_created_at(native.created_at());
        proto.set_order_xml_v3_4(native.order_xml_v3_4().to_string());

        Ok(proto)
    }
}

impl FromBytes<PayloadRevision> for PayloadRevision {
    fn from_bytes(bytes: &[u8]) -> Result<PayloadRevision, ProtoConversionError> {
        let proto: purchase_order_payload::PayloadRevision = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get PayloadRevision from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for PayloadRevision {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from PayloadRevision".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<purchase_order_payload::PayloadRevision> for PayloadRevision {}
impl IntoNative<PayloadRevision> for purchase_order_payload::PayloadRevision {}

/// Builder used to create the revision object made in a "create" or "update" version payload
#[derive(Default, Debug)]
pub struct PayloadRevisionBuilder {
    revision_id: Option<u64>,
    submitter: Option<String>,
    created_at: Option<u64>,
    order_xml_v3_4: Option<String>,
}

impl PayloadRevisionBuilder {
    pub fn new() -> Self {
        PayloadRevisionBuilder::default()
    }

    pub fn with_revision_id(mut self, value: u64) -> Self {
        self.revision_id = Some(value);
        self
    }

    pub fn with_submitter(mut self, value: String) -> Self {
        self.submitter = Some(value);
        self
    }

    pub fn with_created_at(mut self, value: u64) -> Self {
        self.created_at = Some(value);
        self
    }

    pub fn with_order_xml_v3_4(mut self, value: String) -> Self {
        self.order_xml_v3_4 = Some(value);
        self
    }

    pub fn build(self) -> Result<PayloadRevision, BuilderError> {
        let revision_id = self.revision_id.ok_or_else(|| {
            BuilderError::MissingField("'revision_id' field is required".to_string())
        })?;

        let submitter = self.submitter.ok_or_else(|| {
            BuilderError::MissingField("'submitter' field is required".to_string())
        })?;

        let created_at = self.created_at.ok_or_else(|| {
            BuilderError::MissingField("'created_at' field is required".to_string())
        })?;

        let order_xml_v3_4 = self.order_xml_v3_4.ok_or_else(|| {
            BuilderError::MissingField("'order_xml_v3_4' field is required".to_string())
        })?;

        Ok(PayloadRevision {
            revision_id,
            submitter,
            created_at,
            order_xml_v3_4,
        })
    }
}

/// Native representation of the "create version" payload
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CreateVersionPayload {
    version_id: String,
    po_uid: String,
    is_draft: bool,
    workflow_state: String,
    revision: PayloadRevision,
}

impl CreateVersionPayload {
    pub fn version_id(&self) -> &str {
        &self.version_id
    }

    pub fn po_uid(&self) -> &str {
        &self.po_uid
    }

    pub fn is_draft(&self) -> bool {
        self.is_draft
    }

    pub fn workflow_state(&self) -> &str {
        &self.workflow_state
    }

    pub fn revision(&self) -> &PayloadRevision {
        &self.revision
    }
}

impl FromProto<purchase_order_payload::CreateVersionPayload> for CreateVersionPayload {
    fn from_proto(
        mut proto: purchase_order_payload::CreateVersionPayload,
    ) -> Result<Self, ProtoConversionError> {
        Ok(CreateVersionPayload {
            version_id: proto.take_version_id(),
            po_uid: proto.take_po_uid(),
            is_draft: proto.get_is_draft(),
            workflow_state: proto.take_workflow_state(),
            revision: PayloadRevision::from_proto(proto.take_revision())?,
        })
    }
}

impl FromNative<CreateVersionPayload> for purchase_order_payload::CreateVersionPayload {
    fn from_native(native: CreateVersionPayload) -> Result<Self, ProtoConversionError> {
        let mut proto = purchase_order_payload::CreateVersionPayload::new();
        proto.set_version_id(native.version_id().to_string());
        proto.set_po_uid(native.po_uid().to_string());
        proto.set_is_draft(native.is_draft());
        proto.set_workflow_state(native.workflow_state().to_string());
        proto.set_revision(native.revision().clone().into_proto()?);

        Ok(proto)
    }
}

impl FromBytes<CreateVersionPayload> for CreateVersionPayload {
    fn from_bytes(bytes: &[u8]) -> Result<CreateVersionPayload, ProtoConversionError> {
        let proto: purchase_order_payload::CreateVersionPayload = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get CreateVersionPayload from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for CreateVersionPayload {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from CreateVersionPayload".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<purchase_order_payload::CreateVersionPayload> for CreateVersionPayload {}
impl IntoNative<CreateVersionPayload> for purchase_order_payload::CreateVersionPayload {}

/// Builder used to create a "create version" payload
#[derive(Default, Debug)]
pub struct CreateVersionPayloadBuilder {
    version_id: Option<String>,
    po_uid: Option<String>,
    is_draft: Option<bool>,
    workflow_state: Option<String>,
    revision: Option<PayloadRevision>,
}

impl CreateVersionPayloadBuilder {
    pub fn new() -> Self {
        CreateVersionPayloadBuilder::default()
    }

    pub fn with_version_id(mut self, value: String) -> Self {
        self.version_id = Some(value);
        self
    }

    pub fn with_po_uid(mut self, value: String) -> Self {
        self.po_uid = Some(value);
        self
    }

    pub fn with_is_draft(mut self, value: bool) -> Self {
        self.is_draft = Some(value);
        self
    }

    pub fn with_workflow_state(mut self, value: String) -> Self {
        self.workflow_state = Some(value);
        self
    }

    pub fn with_revision(mut self, value: PayloadRevision) -> Self {
        self.revision = Some(value);
        self
    }

    pub fn build(self) -> Result<CreateVersionPayload, BuilderError> {
        let version_id = self.version_id.ok_or_else(|| {
            BuilderError::MissingField("'version_id' field is required".to_string())
        })?;

        let po_uid = self
            .po_uid
            .ok_or_else(|| BuilderError::MissingField("'po_uid' field is required".to_string()))?;

        let is_draft = self.is_draft.ok_or_else(|| {
            BuilderError::MissingField("'is_draft' field is required".to_string())
        })?;

        let workflow_state = self.workflow_state.ok_or_else(|| {
            BuilderError::MissingField("'workflow_state' field is required".to_string())
        })?;

        let revision = self.revision.ok_or_else(|| {
            BuilderError::MissingField("'revision' field is required".to_string())
        })?;

        Ok(CreateVersionPayload {
            version_id,
            po_uid,
            is_draft,
            workflow_state,
            revision,
        })
    }
}

/// Native representation of the "update version" payload
#[derive(Debug, Default, Clone, PartialEq)]
pub struct UpdateVersionPayload {
    version_id: String,
    po_uid: String,
    workflow_state: String,
    is_draft: bool,
    revision: PayloadRevision,
}

impl UpdateVersionPayload {
    pub fn version_id(&self) -> &str {
        &self.version_id
    }

    pub fn po_uid(&self) -> &str {
        &self.po_uid
    }

    pub fn workflow_state(&self) -> &str {
        &self.workflow_state
    }

    pub fn is_draft(&self) -> bool {
        self.is_draft
    }

    pub fn revision(&self) -> &PayloadRevision {
        &self.revision
    }
}

impl FromProto<purchase_order_payload::UpdateVersionPayload> for UpdateVersionPayload {
    fn from_proto(
        mut proto: purchase_order_payload::UpdateVersionPayload,
    ) -> Result<Self, ProtoConversionError> {
        Ok(UpdateVersionPayload {
            version_id: proto.take_version_id(),
            po_uid: proto.take_po_uid(),
            workflow_state: proto.take_workflow_state(),
            is_draft: proto.get_is_draft(),
            revision: PayloadRevision::from_proto(proto.take_revision())?,
        })
    }
}

impl FromNative<UpdateVersionPayload> for purchase_order_payload::UpdateVersionPayload {
    fn from_native(native: UpdateVersionPayload) -> Result<Self, ProtoConversionError> {
        let mut proto = purchase_order_payload::UpdateVersionPayload::new();
        proto.set_version_id(native.version_id().to_string());
        proto.set_po_uid(native.po_uid().to_string());
        proto.set_workflow_state(native.workflow_state().to_string());
        proto.set_is_draft(native.is_draft());
        proto.set_revision(native.revision().clone().into_proto()?);

        Ok(proto)
    }
}

impl FromBytes<UpdateVersionPayload> for UpdateVersionPayload {
    fn from_bytes(bytes: &[u8]) -> Result<UpdateVersionPayload, ProtoConversionError> {
        let proto: purchase_order_payload::UpdateVersionPayload = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get UpdateVersionPayload from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for UpdateVersionPayload {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from UpdateVersionPayload".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<purchase_order_payload::UpdateVersionPayload> for UpdateVersionPayload {}
impl IntoNative<UpdateVersionPayload> for purchase_order_payload::UpdateVersionPayload {}

/// Builder used to create the "update version" payload
#[derive(Default, Debug)]
pub struct UpdateVersionPayloadBuilder {
    version_id: Option<String>,
    po_uid: Option<String>,
    workflow_state: Option<String>,
    is_draft: Option<bool>,
    revision: Option<PayloadRevision>,
}

impl UpdateVersionPayloadBuilder {
    pub fn new() -> Self {
        UpdateVersionPayloadBuilder::default()
    }

    pub fn with_version_id(mut self, value: String) -> Self {
        self.version_id = Some(value);
        self
    }

    pub fn with_po_uid(mut self, value: String) -> Self {
        self.po_uid = Some(value);
        self
    }

    pub fn with_workflow_state(mut self, value: String) -> Self {
        self.workflow_state = Some(value);
        self
    }

    pub fn with_is_draft(mut self, value: bool) -> Self {
        self.is_draft = Some(value);
        self
    }

    pub fn with_revision(mut self, value: PayloadRevision) -> Self {
        self.revision = Some(value);
        self
    }

    pub fn build(self) -> Result<UpdateVersionPayload, BuilderError> {
        let version_id = self.version_id.ok_or_else(|| {
            BuilderError::MissingField("'version_id' field is required".to_string())
        })?;

        let po_uid = self
            .po_uid
            .ok_or_else(|| BuilderError::MissingField("'po_uid' field is required".to_string()))?;

        let workflow_state = self.workflow_state.ok_or_else(|| {
            BuilderError::MissingField("'workflow_state' field is required".to_string())
        })?;

        let is_draft = self.is_draft.ok_or_else(|| {
            BuilderError::MissingField("'is_draft' field is required".to_string())
        })?;

        let revision = self.revision.ok_or_else(|| {
            BuilderError::MissingField("'revision' field is required".to_string())
        })?;

        Ok(UpdateVersionPayload {
            version_id,
            po_uid,
            workflow_state,
            is_draft,
            revision,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::protocol::purchase_order::state::PurchaseOrderAlternateIdBuilder;

    /// Validate UpdatePurchaseOrderPayload protobuf translates to native
    #[test]
    fn update_po_transforms_to_update_po_protobuf_correctly() {
        let alternate_ids = vec![PurchaseOrderAlternateIdBuilder::new()
            .with_id_type("type".to_string())
            .with_id("id".to_string())
            .with_purchase_order_uid("uid".to_string())
            .build()
            .unwrap()];

        // Validate with all fields filled out
        let proto_update_po = UpdatePurchaseOrderPayload {
            uid: "uid".to_string(),
            workflow_state: "status".to_string(),
            is_closed: true,
            accepted_version_number: Some("version".to_string()),
            alternate_ids,
        }
        .into_proto()
        .expect("could not transform into proto");
        assert_eq!(proto_update_po.get_po_uid(), "uid");
        assert_eq!(proto_update_po.get_workflow_state(), "status");
        assert!(proto_update_po.get_is_closed());
        assert_eq!(proto_update_po.get_accepted_version_number(), "version");

        // Validate with optional fields not filled out
        let proto_update_po = UpdatePurchaseOrderPayload {
            uid: "uid".to_string(),
            workflow_state: "status".to_string(),
            is_closed: true,
            accepted_version_number: None,
            alternate_ids: Vec::new(),
        }
        .into_proto()
        .expect("could not transform into proto");
        assert_eq!(proto_update_po.get_accepted_version_number(), "");
    }

    /// Validate UpdatePurchaseOrderPayload native translates to protobuf
    #[test]
    fn update_po_protobuf_transforms_to_update_po_correctly() {
        let alternate_ids = vec![PurchaseOrderAlternateIdBuilder::new()
            .with_id_type("type".to_string())
            .with_id("id".to_string())
            .with_purchase_order_uid("uid".to_string())
            .build()
            .unwrap()
            .into_proto()
            .unwrap()];

        // Validate with all fields filled out
        let mut proto_update_po = purchase_order_payload::UpdatePurchaseOrderPayload::new();
        proto_update_po.set_po_uid("uid".to_string());
        proto_update_po.set_workflow_state("status".to_string());
        proto_update_po.set_is_closed(true);
        proto_update_po.set_accepted_version_number("version".to_string());
        proto_update_po.set_alternate_ids(RepeatedField::from(alternate_ids.to_vec()));

        let update_po = proto_update_po
            .into_native()
            .expect("could not transform into native");
        assert_eq!(update_po.uid(), "uid");
        assert_eq!(update_po.workflow_state(), "status");
        assert!(update_po.is_closed());
        assert_eq!(update_po.accepted_version_number(), Some("version"));
        assert_eq!(
            update_po.alternate_ids(),
            alternate_ids
                .to_vec()
                .into_iter()
                .map(|id| PurchaseOrderAlternateId::from_proto(id).unwrap())
                .collect::<Vec<PurchaseOrderAlternateId>>()
        );

        // Validate with optional fields not filled out
        let mut proto_update_po = purchase_order_payload::UpdatePurchaseOrderPayload::new();
        proto_update_po.set_po_uid("uid".to_string());
        proto_update_po.set_workflow_state("status".to_string());
        proto_update_po.set_is_closed(true);
        proto_update_po.set_accepted_version_number("".to_string());

        let update_po = proto_update_po
            .into_native()
            .expect("could not transform into native");
        assert_eq!(update_po.accepted_version_number(), None);
        assert_eq!(update_po.alternate_ids(), Vec::new());
    }
}
