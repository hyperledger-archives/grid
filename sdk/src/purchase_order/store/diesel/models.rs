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

use crate::pike::store::diesel::schema::*;

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "purchase_order"]
pub struct NewPurchaseOrderModel {
    pub uuid: String,
    pub org_id: String,
    pub workflow_status: String,
    pub is_closed: bool,
    pub accepted_version_id: String,
    pub created_at: i64,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "purchase_order"]
pub struct PurchaseOrderModel {
    pub id: i64,
    pub uuid: String,
    pub org_id: String,
    pub workflow_status: String,
    pub is_closed: bool,
    pub accepted_version_id: String,
    pub created_at: i64,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "purchase_order_version"]
pub struct NewPurchaseOrderVersionModel {
    pub purchase_order_uuid: String,
    pub org_id: String,
    pub version_id: String,
    pub is_draft: bool,
    pub current_revision_id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "purchase_order_version"]
pub struct PurchaseOrderVersionModel {
    pub id: i64,
    pub purchase_order_uuid: String,
    pub org_id: String,
    pub version_id: String,
    pub is_draft: bool,
    pub current_revision_id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "purchase_order_version_revision"]
pub struct NewPurchaseOrderVersionRevisionModel {
    pub version_id: String,
    pub org_id: String,
    pub revision_id: String,
    pub order_xml_v3_4: String,
    pub submitter: String,
    pub created_at: i64,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "purchase_order_version_revision"]
pub struct PurchaseOrderVersionRevisionModel {
    pub id: i64,
    pub version_id: String,
    pub org_id: String,
    pub revision_id: String,
    pub order_xml_v3_4: String,
    pub submitter: String,
    pub created_at: i64,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "purchase_order_alternate_id"]
pub struct NewPurchaseOrderAlternateIDModel {
    pub purchase_order_uuid: String,
    pub org_id: String,
    pub alternate_id_type: String,
    pub alternate_id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "purchase_order_alternate_id"]
pub struct PurchaseOrderAlternateIDModel {
    pub id: i64,
    pub purchase_order_uuid: String,
    pub org_id: String,
    pub alternate_id_type: String,
    pub alternate_id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}
