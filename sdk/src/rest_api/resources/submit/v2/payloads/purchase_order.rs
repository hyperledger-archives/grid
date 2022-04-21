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

use super::TransactionPayload;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PurchaseOrderAction {
    CreatePo(CreatePurchaseOrderAction),
    UpdatePo(UpdatePurchaseOrderAction),
    CreateVersion(CreateVersionAction),
    UpdateVersion(UpdateVersionAction),
}

impl PurchaseOrderAction {
    pub fn into_inner(self) -> Box<dyn TransactionPayload> {
        unimplemented!();
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct PurchaseOrderPayload {
    #[serde(flatten)]
    action: PurchaseOrderAction,
    timestamp: u64,
}

impl PurchaseOrderPayload {
    pub fn action(&self) -> &PurchaseOrderAction {
        &self.action
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn into_transaction_payload(self) -> Box<dyn TransactionPayload> {
        self.action.into_inner()
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct CreatePurchaseOrderAction {
    uid: String,
    created_at: u64,
    buyer_org_id: String,
    seller_org_id: String,
    workflow_state: String,
    alternate_ids: Vec<PurchaseOrderAlternateId>,
    create_version_payload: Option<CreateVersionAction>,
    workflow_id: String,
}

impl CreatePurchaseOrderAction {
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

    pub fn create_version_payload(&self) -> Option<CreateVersionAction> {
        self.create_version_payload.clone()
    }

    pub fn workflow_id(&self) -> &str {
        &self.workflow_id
    }
}

#[derive(Default, Debug)]
pub struct CreatePurchaseOrderActionBuilder {
    uid: Option<String>,
    created_at: Option<u64>,
    buyer_org_id: Option<String>,
    seller_org_id: Option<String>,
    workflow_state: Option<String>,
    alternate_ids: Vec<PurchaseOrderAlternateId>,
    create_version_payload: Option<CreateVersionAction>,
    workflow_id: Option<String>,
}

impl CreatePurchaseOrderActionBuilder {
    pub fn new() -> Self {
        CreatePurchaseOrderActionBuilder::default()
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

    pub fn with_create_version_payload(mut self, payload: CreateVersionAction) -> Self {
        self.create_version_payload = Some(payload);
        self
    }

    pub fn with_workflow_id(mut self, value: String) -> Self {
        self.workflow_id = Some(value);
        self
    }

    pub fn build(self) -> Result<CreatePurchaseOrderAction, BuilderError> {
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

        Ok(CreatePurchaseOrderAction {
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

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct UpdatePurchaseOrderAction {
    uid: String,
    workflow_state: String,
    is_closed: bool,
    accepted_version_number: Option<String>,
    alternate_ids: Vec<PurchaseOrderAlternateId>,
    version_updates: Vec<UpdateVersionAction>,
}

impl UpdatePurchaseOrderAction {
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

    pub fn version_updates(&self) -> &[UpdateVersionAction] {
        &self.version_updates
    }
}

#[derive(Default, Debug)]
pub struct UpdatePurchaseOrderActionBuilder {
    uid: Option<String>,
    workflow_state: Option<String>,
    is_closed: Option<bool>,
    accepted_version_number: Option<String>,
    alternate_ids: Vec<PurchaseOrderAlternateId>,
    version_updates: Vec<UpdateVersionAction>,
}

impl UpdatePurchaseOrderActionBuilder {
    pub fn new() -> Self {
        UpdatePurchaseOrderActionBuilder::default()
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

    pub fn with_version_updates(mut self, value: Vec<UpdateVersionAction>) -> Self {
        self.version_updates = value;
        self
    }

    pub fn build(self) -> Result<UpdatePurchaseOrderAction, BuilderError> {
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

        let version_updates = self.version_updates;

        Ok(UpdatePurchaseOrderAction {
            uid,
            workflow_state,
            is_closed,
            accepted_version_number: self.accepted_version_number,
            alternate_ids,
            version_updates,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct CreateVersionAction {
    version_id: String,
    po_uid: String,
    is_draft: bool,
    workflow_state: String,
    revision: PayloadRevision,
}

impl CreateVersionAction {
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

#[derive(Default, Debug)]
pub struct CreateVersionActionBuilder {
    version_id: Option<String>,
    po_uid: Option<String>,
    is_draft: Option<bool>,
    workflow_state: Option<String>,
    revision: Option<PayloadRevision>,
}

impl CreateVersionActionBuilder {
    pub fn new() -> Self {
        CreateVersionActionBuilder::default()
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

    pub fn build(self) -> Result<CreateVersionAction, BuilderError> {
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

        Ok(CreateVersionAction {
            version_id,
            po_uid,
            is_draft,
            workflow_state,
            revision,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct UpdateVersionAction {
    version_id: String,
    po_uid: String,
    workflow_state: String,
    is_draft: bool,
    revision: PayloadRevision,
}

impl UpdateVersionAction {
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

#[derive(Default, Debug)]
pub struct UpdateVersionActionBuilder {
    version_id: Option<String>,
    po_uid: Option<String>,
    workflow_state: Option<String>,
    is_draft: Option<bool>,
    revision: Option<PayloadRevision>,
}

impl UpdateVersionActionBuilder {
    pub fn new() -> Self {
        UpdateVersionActionBuilder::default()
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

    pub fn build(self) -> Result<UpdateVersionAction, BuilderError> {
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

        Ok(UpdateVersionAction {
            version_id,
            po_uid,
            workflow_state,
            is_draft,
            revision,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
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

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct PurchaseOrderAlternateId {
    id_type: String,
    id: String,
    purchase_order_uid: String,
}

impl PurchaseOrderAlternateId {
    pub fn new(purchase_order_uid: &str, alternate_id_type: &str, alternate_id: &str) -> Self {
        Self {
            purchase_order_uid: purchase_order_uid.to_string(),
            id_type: alternate_id_type.to_string(),
            id: alternate_id.to_string(),
        }
    }

    pub fn id_type(&self) -> &str {
        &self.id_type
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn purchase_order_uid(&self) -> &str {
        &self.purchase_order_uid
    }
}

#[derive(Default, Debug)]
pub struct PurchaseOrderAlternateIdBuilder {
    id_type: Option<String>,
    id: Option<String>,
    purchase_order_uid: Option<String>,
}

impl PurchaseOrderAlternateIdBuilder {
    pub fn new() -> Self {
        PurchaseOrderAlternateIdBuilder::default()
    }

    pub fn with_id_type(mut self, id_type: String) -> Self {
        self.id_type = Some(id_type);
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_purchase_order_uid(mut self, po_uid: String) -> Self {
        self.purchase_order_uid = Some(po_uid);
        self
    }

    pub fn build(self) -> Result<PurchaseOrderAlternateId, BuilderError> {
        let id_type = self
            .id_type
            .ok_or_else(|| BuilderError::MissingField("'id_type' field is required".to_string()))?;

        let id = self
            .id
            .ok_or_else(|| BuilderError::MissingField("'id' field is required".to_string()))?;

        let purchase_order_uid = self.purchase_order_uid.ok_or_else(|| {
            BuilderError::MissingField("'purchase_order_uid' field is required".to_string())
        })?;

        Ok(PurchaseOrderAlternateId {
            id_type,
            id,
            purchase_order_uid,
        })
    }
}
