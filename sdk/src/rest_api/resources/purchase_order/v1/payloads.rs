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

use crate::{
    purchase_order::store::{PurchaseOrder, PurchaseOrderVersion, PurchaseOrderVersionRevision},
    rest_api::resources::paging::v1::Paging,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseOrderSlice {
    pub purchase_order_uid: String,
    pub workflow_status: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_version_id: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_numbers: Option<Vec<String>>,
    pub is_closed: bool,
    pub workflow_type: String,
}

impl From<PurchaseOrder> for PurchaseOrderSlice {
    fn from(purchase_order: PurchaseOrder) -> Self {
        Self {
            purchase_order_uid: purchase_order.purchase_order_uid().to_string(),
            workflow_status: purchase_order.workflow_status().to_string(),
            accepted_version_id: purchase_order.accepted_version_id().map(ToOwned::to_owned),
            version_numbers: Some(
                purchase_order
                    .versions()
                    .into_iter()
                    .map(|version| version.version_id().to_string())
                    .collect(),
            ),
            is_closed: purchase_order.is_closed(),
            workflow_type: purchase_order.workflow_type().to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseOrderListSlice {
    pub data: Vec<PurchaseOrderSlice>,
    pub paging: Paging,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseOrderVersionSlice {
    version_id: String,
    is_draft: bool,
    current_revision_id: String,
    revisions: Vec<String>,
    service_id: Option<String>,
}

impl From<PurchaseOrderVersion> for PurchaseOrderVersionSlice {
    fn from(purchase_order_version: PurchaseOrderVersion) -> Self {
        Self {
            version_id: purchase_order_version.version_id().to_string(),
            is_draft: purchase_order_version.is_draft(),
            current_revision_id: purchase_order_version.current_revision_id().to_string(),
            revisions: purchase_order_version
                .revisions()
                .into_iter()
                .map(|version| version.revision_id().to_string())
                .collect(),
            service_id: purchase_order_version.service_id().map(str::to_owned),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseOrderVersionListSlice {
    pub data: Vec<PurchaseOrderVersionSlice>,
    pub paging: Paging,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseOrderRevisionSlice {
    revision_id: i64,
    submitter: String,
    created_at: i64,
    order_xml_v3_4: String,
}

impl From<PurchaseOrderVersionRevision> for PurchaseOrderRevisionSlice {
    fn from(purchase_order_revision: PurchaseOrderVersionRevision) -> Self {
        Self {
            revision_id: *purchase_order_revision.revision_id(),
            submitter: purchase_order_revision.submitter().to_string(),
            created_at: purchase_order_revision.created_at(),
            order_xml_v3_4: purchase_order_revision.order_xml_v3_4().to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseOrderRevisionListSlice {
    pub data: Vec<PurchaseOrderRevisionSlice>,
    pub paging: Paging,
}
