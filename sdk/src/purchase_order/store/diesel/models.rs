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

//! The structs in these files are the rust representations of what are stored
//! in the database. These are similar to the Grid structs that they correspond
//! to but may contain several database-specific fields such as the database ID
//! and fields for managing slowly-changing dimensions. There should be two
//! structs for each database table: one representing a new record which will
//! not contain a field for the database ID, and one representing an existing
//! record which will. The order of the fields in the struct must match with
//! the corresponding table declaration in the schema.rs file in this module.
//!
//! Additionally, conversion methods for converting these structs to their Grid
//! counterparts are provided.

use super::{
    PurchaseOrder, PurchaseOrderAlternateId, PurchaseOrderVersion, PurchaseOrderVersionRevision,
};
use crate::commits::MAX_COMMIT_NUM;
use crate::purchase_order::store::diesel::schema::*;

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "purchase_order"]
pub struct NewPurchaseOrderModel {
    pub purchase_order_uid: String,
    pub workflow_state: String,
    pub buyer_org_id: String,
    pub seller_org_id: String,
    pub is_closed: bool,
    pub accepted_version_id: Option<String>,
    pub created_at: i64,
    pub workflow_id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "purchase_order"]
pub struct PurchaseOrderModel {
    pub id: i64,
    pub purchase_order_uid: String,
    pub workflow_state: String,
    pub buyer_org_id: String,
    pub seller_org_id: String,
    pub is_closed: bool,
    pub accepted_version_id: Option<String>,
    pub created_at: i64,
    pub workflow_id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "purchase_order_version"]
pub struct NewPurchaseOrderVersionModel {
    pub purchase_order_uid: String,
    pub version_id: String,
    pub is_draft: bool,
    pub current_revision_id: i64,
    pub workflow_state: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "purchase_order_version"]
pub struct PurchaseOrderVersionModel {
    pub id: i64,
    pub purchase_order_uid: String,
    pub version_id: String,
    pub is_draft: bool,
    pub current_revision_id: i64,
    pub workflow_state: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "purchase_order_version_revision"]
pub struct NewPurchaseOrderVersionRevisionModel {
    pub purchase_order_uid: String,
    pub version_id: String,
    pub revision_id: i64,
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
    pub purchase_order_uid: String,
    pub version_id: String,
    pub revision_id: i64,
    pub order_xml_v3_4: String,
    pub submitter: String,
    pub created_at: i64,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "purchase_order_alternate_id"]
pub struct NewPurchaseOrderAlternateIdModel {
    pub purchase_order_uid: String,
    pub alternate_id_type: String,
    pub alternate_id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "purchase_order_alternate_id"]
pub struct PurchaseOrderAlternateIdModel {
    pub id: i64,
    pub purchase_order_uid: String,
    pub alternate_id_type: String,
    pub alternate_id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

impl From<PurchaseOrder> for NewPurchaseOrderModel {
    fn from(order: PurchaseOrder) -> Self {
        Self {
            purchase_order_uid: order.purchase_order_uid.to_string(),
            workflow_state: order.workflow_state.to_string(),
            buyer_org_id: order.buyer_org_id.to_string(),
            seller_org_id: order.seller_org_id.to_string(),
            is_closed: order.is_closed,
            accepted_version_id: order.accepted_version_id,
            created_at: order.created_at,
            workflow_id: order.workflow_id.to_string(),
            start_commit_num: order.start_commit_num,
            end_commit_num: order.end_commit_num,
            service_id: order.service_id,
        }
    }
}

impl
    From<(
        PurchaseOrderModel,
        Vec<PurchaseOrderVersion>,
        Vec<PurchaseOrderAlternateId>,
    )> for PurchaseOrder
{
    fn from(
        (order, versions, alternate_ids): (
            PurchaseOrderModel,
            Vec<PurchaseOrderVersion>,
            Vec<PurchaseOrderAlternateId>,
        ),
    ) -> Self {
        Self {
            purchase_order_uid: order.purchase_order_uid.to_string(),
            workflow_state: order.workflow_state.to_string(),
            buyer_org_id: order.buyer_org_id.to_string(),
            seller_org_id: order.seller_org_id.to_string(),
            is_closed: order.is_closed,
            accepted_version_id: order.accepted_version_id,
            versions,
            alternate_ids,
            created_at: order.created_at,
            workflow_id: order.workflow_id.to_string(),
            start_commit_num: order.start_commit_num,
            end_commit_num: order.end_commit_num,
            service_id: order.service_id,
        }
    }
}

impl
    From<(
        PurchaseOrderModel,
        Vec<PurchaseOrderVersionModel>,
        Vec<PurchaseOrderVersionRevisionModel>,
        Vec<PurchaseOrderAlternateIdModel>,
    )> for PurchaseOrder
{
    fn from(
        (order, versions, revisions, alternate_ids): (
            PurchaseOrderModel,
            Vec<PurchaseOrderVersionModel>,
            Vec<PurchaseOrderVersionRevisionModel>,
            Vec<PurchaseOrderAlternateIdModel>,
        ),
    ) -> Self {
        Self {
            purchase_order_uid: order.purchase_order_uid.to_string(),
            workflow_state: order.workflow_state.to_string(),
            buyer_org_id: order.buyer_org_id.to_string(),
            seller_org_id: order.seller_org_id.to_string(),
            is_closed: order.is_closed,
            accepted_version_id: order.accepted_version_id,
            versions: versions
                .iter()
                .map(|v| PurchaseOrderVersion::from((v, &revisions)))
                .collect(),
            alternate_ids: alternate_ids
                .iter()
                .map(PurchaseOrderAlternateId::from)
                .collect(),
            created_at: order.created_at,
            workflow_id: order.workflow_id.to_string(),
            start_commit_num: order.start_commit_num,
            end_commit_num: order.end_commit_num,
            service_id: order.service_id,
        }
    }
}

impl
    From<(
        &PurchaseOrderVersionModel,
        &Vec<PurchaseOrderVersionRevisionModel>,
    )> for PurchaseOrderVersion
{
    fn from(
        (version, revisions): (
            &PurchaseOrderVersionModel,
            &Vec<PurchaseOrderVersionRevisionModel>,
        ),
    ) -> Self {
        Self {
            version_id: version.version_id.to_string(),
            is_draft: version.is_draft,
            current_revision_id: version.current_revision_id,
            revisions: revisions
                .iter()
                .filter(|r| r.version_id == version.version_id)
                .map(PurchaseOrderVersionRevision::from)
                .collect(),
            workflow_state: version.workflow_state.to_string(),
            start_commit_num: version.start_commit_num,
            end_commit_num: version.end_commit_num,
            service_id: version.service_id.clone(),
        }
    }
}

impl From<(PurchaseOrderVersionModel, &i64, &i64)> for NewPurchaseOrderVersionModel {
    fn from(
        (version, current_revision_id, start_commit_num): (PurchaseOrderVersionModel, &i64, &i64),
    ) -> Self {
        Self {
            purchase_order_uid: version.purchase_order_uid,
            version_id: version.version_id,
            is_draft: version.is_draft,
            current_revision_id: *current_revision_id,
            workflow_state: version.workflow_state,
            start_commit_num: *start_commit_num,
            end_commit_num: MAX_COMMIT_NUM,
            service_id: version.service_id,
        }
    }
}

impl From<&PurchaseOrderVersionRevisionModel> for PurchaseOrderVersionRevision {
    fn from(revision: &PurchaseOrderVersionRevisionModel) -> Self {
        Self {
            revision_id: revision.revision_id,
            order_xml_v3_4: revision.order_xml_v3_4.to_string(),
            submitter: revision.submitter.to_string(),
            created_at: revision.created_at,
            start_commit_num: revision.start_commit_num,
            end_commit_num: revision.end_commit_num,
            service_id: revision.service_id.clone(),
        }
    }
}

impl From<PurchaseOrderVersionRevisionModel> for PurchaseOrderVersionRevision {
    fn from(revision: PurchaseOrderVersionRevisionModel) -> Self {
        Self {
            revision_id: revision.revision_id,
            order_xml_v3_4: revision.order_xml_v3_4.to_string(),
            submitter: revision.submitter.to_string(),
            created_at: revision.created_at,
            start_commit_num: revision.start_commit_num,
            end_commit_num: revision.end_commit_num,
            service_id: revision.service_id,
        }
    }
}

impl From<PurchaseOrderAlternateId> for NewPurchaseOrderAlternateIdModel {
    fn from(id: PurchaseOrderAlternateId) -> Self {
        Self {
            purchase_order_uid: id.purchase_order_uid.to_string(),
            alternate_id_type: id.id_type.to_string(),
            alternate_id: id.id.to_string(),
            start_commit_num: id.start_commit_num,
            end_commit_num: id.end_commit_num,
            service_id: id.service_id,
        }
    }
}

impl From<&PurchaseOrderAlternateId> for NewPurchaseOrderAlternateIdModel {
    fn from(id: &PurchaseOrderAlternateId) -> Self {
        Self {
            purchase_order_uid: id.purchase_order_uid.to_string(),
            alternate_id_type: id.id_type.to_string(),
            alternate_id: id.id.to_string(),
            start_commit_num: id.start_commit_num,
            end_commit_num: id.end_commit_num,
            service_id: id.service_id.clone(),
        }
    }
}

impl From<&PurchaseOrderAlternateIdModel> for PurchaseOrderAlternateId {
    fn from(id: &PurchaseOrderAlternateIdModel) -> Self {
        Self {
            purchase_order_uid: id.purchase_order_uid.to_string(),
            id_type: id.alternate_id_type.to_string(),
            id: id.alternate_id.to_string(),
            start_commit_num: id.start_commit_num,
            end_commit_num: id.end_commit_num,
            service_id: id.service_id.clone(),
        }
    }
}

pub fn make_purchase_order_versions(order: &PurchaseOrder) -> Vec<NewPurchaseOrderVersionModel> {
    let mut models = Vec::new();
    for version in &order.versions {
        let model = NewPurchaseOrderVersionModel {
            purchase_order_uid: order.purchase_order_uid.to_string(),
            version_id: version.version_id.to_string(),
            is_draft: version.is_draft,
            current_revision_id: version.current_revision_id,
            workflow_state: version.workflow_state.to_string(),
            start_commit_num: version.start_commit_num,
            end_commit_num: MAX_COMMIT_NUM,
            service_id: version.service_id.clone(),
        };

        models.push(model);
    }

    models
}

pub fn make_purchase_order_version_revisions(
    order: &PurchaseOrder,
) -> Vec<NewPurchaseOrderVersionRevisionModel> {
    let mut models = Vec::new();
    for version in &order.versions {
        for revision in &version.revisions {
            let model = NewPurchaseOrderVersionRevisionModel {
                purchase_order_uid: order.purchase_order_uid.to_string(),
                version_id: version.version_id.to_string(),
                revision_id: revision.revision_id,
                order_xml_v3_4: revision.order_xml_v3_4.to_string(),
                submitter: revision.submitter.to_string(),
                created_at: revision.created_at,
                start_commit_num: revision.start_commit_num,
                end_commit_num: MAX_COMMIT_NUM,
                service_id: revision.service_id.clone(),
            };

            models.push(model);
        }
    }

    models
}
