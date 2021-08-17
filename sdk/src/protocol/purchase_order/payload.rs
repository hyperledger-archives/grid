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

use protobuf::Message;

use crate::protocol::errors::BuilderError;
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
    public_key: String,
    timestamp: u64,
}

impl PurchaseOrderPayload {
    pub fn action(&self) -> &Action {
        &self.action
    }

    pub fn public_key(&self) -> &str {
        &self.public_key
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
            public_key: payload.take_public_key(),
            timestamp: payload.get_timestamp(),
        })
    }
}

impl FromNative<PurchaseOrderPayload> for purchase_order_payload::PurchaseOrderPayload {
    fn from_native(native: PurchaseOrderPayload) -> Result<Self, ProtoConversionError> {
        let mut proto = purchase_order_payload::PurchaseOrderPayload::new();

        proto.set_timestamp(native.timestamp());
        proto.set_public_key(native.public_key().to_string());

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
    public_key: Option<String>,
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

    pub fn with_public_key(mut self, public_key: String) -> Self {
        self.public_key = Some(public_key);
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

        let public_key = self
            .public_key
            .ok_or_else(|| BuilderError::MissingField("'public_key' field is required".into()))?;

        let timestamp = self
            .timestamp
            .ok_or_else(|| BuilderError::MissingField("'timestamp' field is required".into()))?;

        Ok(PurchaseOrderPayload {
            action,
            public_key,
            timestamp,
        })
    }
}

/// Native representation of the "create purchase order" payload
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CreatePurchaseOrderPayload {
    org_id: String,
    uuid: String,
    created_at: u64,
    create_version_payload: Option<CreateVersionPayload>,
}

impl CreatePurchaseOrderPayload {
    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    pub fn create_version_payload(&self) -> Option<CreateVersionPayload> {
        self.create_version_payload.clone()
    }
}

impl FromProto<purchase_order_payload::CreatePurchaseOrderPayload> for CreatePurchaseOrderPayload {
    fn from_proto(
        mut proto: purchase_order_payload::CreatePurchaseOrderPayload,
    ) -> Result<Self, ProtoConversionError> {
        let create_version_payload =
            CreateVersionPayload::from_proto(proto.take_create_version_payload()).ok();
        Ok(CreatePurchaseOrderPayload {
            org_id: proto.take_org_id(),
            uuid: proto.take_uuid(),
            created_at: proto.get_created_at(),
            create_version_payload,
        })
    }
}

impl FromNative<CreatePurchaseOrderPayload> for purchase_order_payload::CreatePurchaseOrderPayload {
    fn from_native(native: CreatePurchaseOrderPayload) -> Result<Self, ProtoConversionError> {
        let mut proto = purchase_order_payload::CreatePurchaseOrderPayload::new();
        proto.set_org_id(native.org_id().to_string());
        proto.set_uuid(native.uuid().to_string());
        proto.set_created_at(native.created_at());
        if let Some(create_version) = native.create_version_payload() {
            proto.set_create_version_payload(CreateVersionPayload::into_proto(create_version)?);
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
    org_id: Option<String>,
    uuid: Option<String>,
    created_at: Option<u64>,
    create_version_payload: Option<CreateVersionPayload>,
}

impl CreatePurchaseOrderPayloadBuilder {
    pub fn new() -> Self {
        CreatePurchaseOrderPayloadBuilder::default()
    }

    pub fn with_org_id(mut self, value: String) -> Self {
        self.org_id = Some(value);
        self
    }

    pub fn with_uuid(mut self, value: String) -> Self {
        self.uuid = Some(value);
        self
    }

    pub fn with_created_at(mut self, value: u64) -> Self {
        self.created_at = Some(value);
        self
    }

    pub fn with_create_version_payload(mut self, value: CreateVersionPayload) -> Self {
        self.create_version_payload = Some(value);
        self
    }

    pub fn build(self) -> Result<CreatePurchaseOrderPayload, BuilderError> {
        let org_id = self
            .org_id
            .ok_or_else(|| BuilderError::MissingField("'org_id' field is required".to_string()))?;

        let uuid = self
            .uuid
            .ok_or_else(|| BuilderError::MissingField("'uuid' field is required".to_string()))?;

        let created_at = self.created_at.ok_or_else(|| {
            BuilderError::MissingField("'created_at' field is required".to_string())
        })?;

        Ok(CreatePurchaseOrderPayload {
            org_id,
            uuid,
            created_at,
            create_version_payload: self.create_version_payload,
        })
    }
}

/// Native representation of the "update purchase order" payload
#[derive(Debug, Default, Clone, PartialEq)]
pub struct UpdatePurchaseOrderPayload {
    workflow_status: String,
    is_closed: bool,
    accepted_version_id: String,
    po_uuid: String,
}

impl UpdatePurchaseOrderPayload {
    pub fn workflow_status(&self) -> &str {
        &self.workflow_status
    }

    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    pub fn accepted_version_id(&self) -> &str {
        &self.accepted_version_id
    }

    pub fn po_uuid(&self) -> &str {
        &self.po_uuid
    }
}

impl FromProto<purchase_order_payload::UpdatePurchaseOrderPayload> for UpdatePurchaseOrderPayload {
    fn from_proto(
        mut proto: purchase_order_payload::UpdatePurchaseOrderPayload,
    ) -> Result<Self, ProtoConversionError> {
        Ok(UpdatePurchaseOrderPayload {
            workflow_status: proto.take_workflow_status(),
            is_closed: proto.get_is_closed(),
            accepted_version_id: proto.take_accepted_version_id(),
            po_uuid: proto.take_po_uuid(),
        })
    }
}

impl FromNative<UpdatePurchaseOrderPayload> for purchase_order_payload::UpdatePurchaseOrderPayload {
    fn from_native(native: UpdatePurchaseOrderPayload) -> Result<Self, ProtoConversionError> {
        let mut proto = purchase_order_payload::UpdatePurchaseOrderPayload::new();
        proto.set_workflow_status(native.workflow_status().to_string());
        proto.set_is_closed(native.is_closed());
        proto.set_accepted_version_id(native.accepted_version_id().to_string());
        proto.set_po_uuid(native.po_uuid().to_string());

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
    workflow_status: Option<String>,
    is_closed: Option<bool>,
    accepted_version_id: Option<String>,
    po_uuid: Option<String>,
}

impl UpdatePurchaseOrderPayloadBuilder {
    pub fn new() -> Self {
        UpdatePurchaseOrderPayloadBuilder::default()
    }

    pub fn with_workflow_status(mut self, value: String) -> Self {
        self.workflow_status = Some(value);
        self
    }

    pub fn with_is_closed(mut self, value: bool) -> Self {
        self.is_closed = Some(value);
        self
    }

    pub fn with_accepted_version_id(mut self, value: String) -> Self {
        self.accepted_version_id = Some(value);
        self
    }

    pub fn with_po_uuid(mut self, value: String) -> Self {
        self.po_uuid = Some(value);
        self
    }

    pub fn build(self) -> Result<UpdatePurchaseOrderPayload, BuilderError> {
        let workflow_status = self.workflow_status.ok_or_else(|| {
            BuilderError::MissingField("'workflow_status' field is required".to_string())
        })?;

        let is_closed = self.is_closed.ok_or_else(|| {
            BuilderError::MissingField("'is_closed' field is required".to_string())
        })?;

        let accepted_version_id = self.accepted_version_id.ok_or_else(|| {
            BuilderError::MissingField("'accepted_version_id' field is required".to_string())
        })?;

        let po_uuid = self
            .po_uuid
            .ok_or_else(|| BuilderError::MissingField("'po_uuid' field is required".to_string()))?;

        Ok(UpdatePurchaseOrderPayload {
            workflow_status,
            is_closed,
            accepted_version_id,
            po_uuid,
        })
    }
}

/// Native representation of the revision made in a "create" or "update" version payload
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PayloadRevision {
    revision_id: String,
    submitter: String,
    created_at: u64,
    order_xml_v3_4: String,
}

impl PayloadRevision {
    pub fn revision_id(&self) -> &str {
        &self.revision_id
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
            revision_id: proto.take_revision_id(),
            submitter: proto.take_submitter(),
            created_at: proto.get_created_at(),
            order_xml_v3_4: proto.take_order_xml_v3_4(),
        })
    }
}

impl FromNative<PayloadRevision> for purchase_order_payload::PayloadRevision {
    fn from_native(native: PayloadRevision) -> Result<Self, ProtoConversionError> {
        let mut proto = purchase_order_payload::PayloadRevision::new();
        proto.set_revision_id(native.revision_id().to_string());
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
    revision_id: Option<String>,
    submitter: Option<String>,
    created_at: Option<u64>,
    order_xml_v3_4: Option<String>,
}

impl PayloadRevisionBuilder {
    pub fn new() -> Self {
        PayloadRevisionBuilder::default()
    }

    pub fn with_revision_id(mut self, value: String) -> Self {
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
    is_draft: bool,
    revision: PayloadRevision,
}

impl CreateVersionPayload {
    pub fn version_id(&self) -> &str {
        &self.version_id
    }

    pub fn is_draft(&self) -> bool {
        self.is_draft
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
            is_draft: proto.get_is_draft(),
            revision: PayloadRevision::from_proto(proto.take_revision())?,
        })
    }
}

impl FromNative<CreateVersionPayload> for purchase_order_payload::CreateVersionPayload {
    fn from_native(native: CreateVersionPayload) -> Result<Self, ProtoConversionError> {
        let mut proto = purchase_order_payload::CreateVersionPayload::new();
        proto.set_version_id(native.version_id().to_string());
        proto.set_is_draft(native.is_draft());
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
    is_draft: Option<bool>,
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

    pub fn with_is_draft(mut self, value: bool) -> Self {
        self.is_draft = Some(value);
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

        let is_draft = self.is_draft.ok_or_else(|| {
            BuilderError::MissingField("'is_draft' field is required".to_string())
        })?;

        let revision = self.revision.ok_or_else(|| {
            BuilderError::MissingField("'revision' field is required".to_string())
        })?;

        Ok(CreateVersionPayload {
            version_id,
            is_draft,
            revision,
        })
    }
}

/// Native representation of the "update version" payload
#[derive(Debug, Default, Clone, PartialEq)]
pub struct UpdateVersionPayload {
    version_id: String,
    workflow_status: String,
    is_draft: bool,
    current_revision_id: String,
    revision: PayloadRevision,
}

impl UpdateVersionPayload {
    pub fn version_id(&self) -> &str {
        &self.version_id
    }

    pub fn workflow_status(&self) -> &str {
        &self.workflow_status
    }

    pub fn is_draft(&self) -> bool {
        self.is_draft
    }

    pub fn current_revision_id(&self) -> &str {
        &self.current_revision_id
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
            workflow_status: proto.take_workflow_status(),
            is_draft: proto.get_is_draft(),
            current_revision_id: proto.take_current_revision_id(),
            revision: PayloadRevision::from_proto(proto.take_revision())?,
        })
    }
}

impl FromNative<UpdateVersionPayload> for purchase_order_payload::UpdateVersionPayload {
    fn from_native(native: UpdateVersionPayload) -> Result<Self, ProtoConversionError> {
        let mut proto = purchase_order_payload::UpdateVersionPayload::new();
        proto.set_version_id(native.version_id().to_string());
        proto.set_workflow_status(native.workflow_status().to_string());
        proto.set_is_draft(native.is_draft());
        proto.set_current_revision_id(native.current_revision_id().to_string());
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
    workflow_status: Option<String>,
    is_draft: Option<bool>,
    current_revision_id: Option<String>,
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

    pub fn with_workflow_status(mut self, value: String) -> Self {
        self.workflow_status = Some(value);
        self
    }

    pub fn with_is_draft(mut self, value: bool) -> Self {
        self.is_draft = Some(value);
        self
    }

    pub fn with_current_revision_id(mut self, value: String) -> Self {
        self.current_revision_id = Some(value);
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

        let workflow_status = self.workflow_status.ok_or_else(|| {
            BuilderError::MissingField("'workflow_status' field is required".to_string())
        })?;

        let is_draft = self.is_draft.ok_or_else(|| {
            BuilderError::MissingField("'is_draft' field is required".to_string())
        })?;

        let current_revision_id = self.current_revision_id.ok_or_else(|| {
            BuilderError::MissingField("'current_revision_id' field is required".to_string())
        })?;

        let revision = self.revision.ok_or_else(|| {
            BuilderError::MissingField("'revision' field is required".to_string())
        })?;

        Ok(UpdateVersionPayload {
            version_id,
            workflow_status,
            is_draft,
            current_revision_id,
            revision,
        })
    }
}
