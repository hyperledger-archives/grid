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

use crate::{purchase_order::store::PurchaseOrder, rest_api::resources::paging::v1::Paging};

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
}

impl From<PurchaseOrder> for PurchaseOrderSlice {
    fn from(purchase_order: PurchaseOrder) -> Self {
        Self {
            purchase_order_uid: purchase_order.purchase_order_uid,
            workflow_status: purchase_order.workflow_status,
            accepted_version_id: Some(purchase_order.accepted_version_id),
            version_numbers: Some(
                purchase_order
                    .versions
                    .into_iter()
                    .map(|version| version.version_id)
                    .collect(),
            ),
            is_closed: purchase_order.is_closed,
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
    workflow_status: String,
    is_draft: bool,
    current_revision_number: u64,
    revisions: Vec<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseOrderRevisionSlice {
    revision_number: u64,
    submitter: String,
    created_at: u64,
    order_xml_v3_4: String,
}
