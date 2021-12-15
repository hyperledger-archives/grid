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

//! This module provides the data types for the reqwest-backed client
//! implementation. These must be able to be converted into their
//! corresponding structs in the corresponding client module.

use crate::client::purchase_order::{
    AlternateId as ClientAlternateId, PurchaseOrder as ClientPurchaseOrder,
    PurchaseOrderRevision as ClientPurchaseOrderRevision,
    PurchaseOrderVersion as ClientPurchaseOrderVersion,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PurchaseOrder {
    purchase_order_uid: String,
    workflow_state: String,
    buyer_org_id: String,
    seller_org_id: String,
    is_closed: bool,
    alternate_ids: Vec<AlternateId>,
    accepted_version_id: Option<String>,
    versions: Vec<PurchaseOrderVersion>,
    created_at: i64,
    workflow_type: String,
}

impl From<&PurchaseOrder> for ClientPurchaseOrder {
    fn from(d: &PurchaseOrder) -> Self {
        Self {
            purchase_order_uid: d.purchase_order_uid.to_string(),
            workflow_state: d.workflow_state.to_string(),
            buyer_org_id: d.buyer_org_id.to_string(),
            seller_org_id: d.seller_org_id.to_string(),
            is_closed: d.is_closed,
            alternate_ids: d
                .alternate_ids
                .iter()
                .map(ClientAlternateId::from)
                .collect(),
            accepted_version_id: d.accepted_version_id.as_ref().map(String::from),
            versions: d
                .versions
                .iter()
                .map(ClientPurchaseOrderVersion::from)
                .collect(),
            created_at: d.created_at,
            workflow_type: d.workflow_type.to_string(),
        }
    }
}

impl From<PurchaseOrder> for ClientPurchaseOrder {
    fn from(d: PurchaseOrder) -> Self {
        Self {
            purchase_order_uid: d.purchase_order_uid.to_string(),
            workflow_state: d.workflow_state.to_string(),
            buyer_org_id: d.buyer_org_id.to_string(),
            seller_org_id: d.seller_org_id.to_string(),
            is_closed: d.is_closed,
            alternate_ids: d
                .alternate_ids
                .iter()
                .map(ClientAlternateId::from)
                .collect(),
            accepted_version_id: d.accepted_version_id.as_ref().map(String::from),
            versions: d
                .versions
                .iter()
                .map(ClientPurchaseOrderVersion::from)
                .collect(),
            created_at: d.created_at,
            workflow_type: d.workflow_type.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PurchaseOrderVersion {
    version_id: String,
    workflow_state: String,
    is_draft: bool,
    current_revision_id: u64,
    revisions: Vec<PurchaseOrderRevision>,
}

impl From<&PurchaseOrderVersion> for ClientPurchaseOrderVersion {
    fn from(d: &PurchaseOrderVersion) -> Self {
        Self {
            version_id: d.version_id.to_string(),
            workflow_state: d.workflow_state.to_string(),
            is_draft: d.is_draft,
            current_revision_id: d.current_revision_id,
            revisions: d
                .revisions
                .iter()
                .map(ClientPurchaseOrderRevision::from)
                .collect(),
        }
    }
}

impl From<PurchaseOrderVersion> for ClientPurchaseOrderVersion {
    fn from(d: PurchaseOrderVersion) -> Self {
        Self {
            version_id: d.version_id.to_string(),
            workflow_state: d.workflow_state.to_string(),
            is_draft: d.is_draft,
            current_revision_id: d.current_revision_id,
            revisions: d
                .revisions
                .iter()
                .map(ClientPurchaseOrderRevision::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PurchaseOrderRevision {
    revision_id: u64,
    order_xml_v3_4: String,
    submitter: String,
    created_at: i64,
}

impl From<&PurchaseOrderRevision> for ClientPurchaseOrderRevision {
    fn from(d: &PurchaseOrderRevision) -> Self {
        Self {
            revision_id: d.revision_id,
            order_xml_v3_4: d.order_xml_v3_4.to_string(),
            submitter: d.submitter.to_string(),
            created_at: d.created_at,
        }
    }
}

impl From<PurchaseOrderRevision> for ClientPurchaseOrderRevision {
    fn from(d: PurchaseOrderRevision) -> Self {
        Self {
            revision_id: d.revision_id,
            order_xml_v3_4: d.order_xml_v3_4.to_string(),
            submitter: d.submitter.to_string(),
            created_at: d.created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlternateId {
    pub purchase_order_uid: String,
    pub id_type: String,
    pub id: String,
}

impl From<&AlternateId> for ClientAlternateId {
    fn from(d: &AlternateId) -> Self {
        Self {
            purchase_order_uid: d.purchase_order_uid.to_string(),
            alternate_id_type: d.id_type.to_string(),
            alternate_id: d.id.to_string(),
        }
    }
}
