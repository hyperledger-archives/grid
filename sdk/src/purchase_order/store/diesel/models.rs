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

use super::{
    PurchaseOrder, PurchaseOrderAlternateId, PurchaseOrderVersion, PurchaseOrderVersionRevision,
};
use crate::commits::MAX_COMMIT_NUM;
use crate::purchase_order::store::diesel::schema::*;

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "purchase_order"]
pub struct NewPurchaseOrderModel {
    pub purchase_order_uid: String,
    pub workflow_status: String,
    pub buyer_org_id: String,
    pub seller_org_id: String,
    pub is_closed: bool,
    pub accepted_version_id: Option<String>,
    pub created_at: i64,
    pub workflow_type: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "purchase_order"]
pub struct PurchaseOrderModel {
    pub id: i64,
    pub purchase_order_uid: String,
    pub workflow_status: String,
    pub buyer_org_id: String,
    pub seller_org_id: String,
    pub is_closed: bool,
    pub accepted_version_id: Option<String>,
    pub created_at: i64,
    pub workflow_type: String,
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
    pub current_revision_id: String,
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
    pub current_revision_id: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Insertable, PartialEq, Queryable, Debug)]
#[table_name = "purchase_order_version_revision"]
pub struct NewPurchaseOrderVersionRevisionModel {
    pub purchase_order_uid: String,
    pub version_id: String,
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
    pub purchase_order_uid: String,
    pub version_id: String,
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
pub struct NewPurchaseOrderAlternateIdModel {
    pub purchase_order_uid: String,
    pub org_id: String,
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
    pub org_id: String,
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
            buyer_org_id: order.buyer_org_id.to_string(),
            seller_org_id: order.seller_org_id.to_string(),
            workflow_status: order.workflow_status.to_string(),
            is_closed: order.is_closed,
            accepted_version_id: order.accepted_version_id,
            created_at: order.created_at,
            workflow_type: order.workflow_type.to_string(),
            start_commit_num: order.start_commit_num,
            end_commit_num: order.end_commit_num,
            service_id: order.service_id,
        }
    }
}

impl From<(PurchaseOrderModel, Vec<PurchaseOrderVersion>)> for PurchaseOrder {
    fn from((order, versions): (PurchaseOrderModel, Vec<PurchaseOrderVersion>)) -> Self {
        Self {
            purchase_order_uid: order.purchase_order_uid.to_string(),
            buyer_org_id: order.buyer_org_id.to_string(),
            seller_org_id: order.seller_org_id.to_string(),
            workflow_status: order.workflow_status.to_string(),
            is_closed: order.is_closed,
            accepted_version_id: order.accepted_version_id,
            versions,
            created_at: order.created_at,
            workflow_type: order.workflow_type.to_string(),
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
    )> for PurchaseOrder
{
    fn from(
        (order, versions, revisions): (
            PurchaseOrderModel,
            Vec<PurchaseOrderVersionModel>,
            Vec<PurchaseOrderVersionRevisionModel>,
        ),
    ) -> Self {
        Self {
            purchase_order_uid: order.purchase_order_uid.to_string(),
            buyer_org_id: order.buyer_org_id.to_string(),
            seller_org_id: order.seller_org_id.to_string(),
            workflow_status: order.workflow_status.to_string(),
            is_closed: order.is_closed,
            accepted_version_id: order.accepted_version_id,
            versions: versions
                .iter()
                .map(|v| PurchaseOrderVersion::from((v, &revisions)))
                .collect(),
            created_at: order.created_at,
            workflow_type: order.workflow_type.to_string(),
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
            current_revision_id: version.current_revision_id.to_string(),
            revisions: revisions
                .iter()
                .filter(|r| r.version_id == version.version_id)
                .map(PurchaseOrderVersionRevision::from)
                .collect(),
            start_commit_num: version.start_commit_num,
            end_commit_num: version.end_commit_num,
            service_id: version.service_id.clone(),
        }
    }
}

impl From<(PurchaseOrderVersionModel, &String, &i64)> for NewPurchaseOrderVersionModel {
    fn from(
        (version, current_revision_id, start_commit_num): (
            PurchaseOrderVersionModel,
            &String,
            &i64,
        ),
    ) -> Self {
        Self {
            purchase_order_uid: version.purchase_order_uid,
            version_id: version.version_id,
            is_draft: version.is_draft,
            current_revision_id: current_revision_id.to_string(),
            start_commit_num: *start_commit_num,
            end_commit_num: MAX_COMMIT_NUM,
            service_id: version.service_id,
        }
    }
}

impl From<&PurchaseOrderVersionRevisionModel> for PurchaseOrderVersionRevision {
    fn from(revision: &PurchaseOrderVersionRevisionModel) -> Self {
        Self {
            revision_id: revision.revision_id.to_string(),
            order_xml_v3_4: revision.order_xml_v3_4.to_string(),
            submitter: revision.submitter.to_string(),
            created_at: revision.created_at,
            start_commit_num: revision.start_commit_num,
            end_commit_num: revision.end_commit_num,
            service_id: revision.service_id.clone(),
        }
    }
}

impl From<PurchaseOrderAlternateId> for NewPurchaseOrderAlternateIdModel {
    fn from(id: PurchaseOrderAlternateId) -> Self {
        Self {
            purchase_order_uid: id.purchase_order_uid.to_string(),
            org_id: id.org_id.to_string(),
            alternate_id_type: id.id_type.to_string(),
            alternate_id: id.id.to_string(),
            start_commit_num: id.start_commit_num,
            end_commit_num: id.end_commit_num,
            service_id: id.service_id,
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
            current_revision_id: version.current_revision_id.to_string(),
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
                revision_id: revision.revision_id.to_string(),
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
