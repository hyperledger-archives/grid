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

#[cfg(feature = "diesel")]
pub(in crate) mod diesel;
mod error;

use crate::paging::Paging;

#[cfg(feature = "diesel")]
pub use self::diesel::{DieselConnectionPurchaseOrderStore, DieselPurchaseOrderStore};
pub use error::{PurchaseOrderBuilderError, PurchaseOrderStoreError};

/// Represents a list of Grid Purchase Orders
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PurchaseOrderList {
    pub data: Vec<PurchaseOrder>,
    pub paging: Paging,
}

impl PurchaseOrderList {
    pub fn new(data: Vec<PurchaseOrder>, paging: Paging) -> Self {
        Self { data, paging }
    }
}

/// Represents a Grid Purchase Order
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PurchaseOrder {
    purchase_order_uid: String,
    workflow_state: String,
    buyer_org_id: String,
    seller_org_id: String,
    is_closed: bool,
    accepted_version_id: Option<String>,
    versions: Vec<PurchaseOrderVersion>,
    alternate_ids: Vec<PurchaseOrderAlternateId>,
    created_at: i64,
    workflow_id: String,
    start_commit_num: i64,
    end_commit_num: i64,
    service_id: Option<String>,
}

impl PurchaseOrder {
    /// Returns the UID for the PO
    pub fn purchase_order_uid(&self) -> &str {
        &self.purchase_order_uid
    }

    /// Returns the workflow state for the PO
    pub fn workflow_state(&self) -> &str {
        &self.workflow_state
    }

    /// Returns the buyer's org ID for the PO
    pub fn buyer_org_id(&self) -> &str {
        &self.buyer_org_id
    }

    /// Returns the seller's org ID for the PO
    pub fn seller_org_id(&self) -> &str {
        &self.seller_org_id
    }

    /// Returns the is_closed value for the PO
    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    /// Returns the accepted version ID for the PO
    pub fn accepted_version_id(&self) -> Option<&str> {
        self.accepted_version_id.as_deref()
    }

    /// Returns the versions list for the PO
    pub fn versions(&self) -> Vec<PurchaseOrderVersion> {
        self.versions.to_vec()
    }

    /// Returns the alternate IDs list for the PO
    pub fn alternate_ids(&self) -> Vec<PurchaseOrderAlternateId> {
        self.alternate_ids.to_vec()
    }

    /// Returns the created_at timestamp for the PO
    pub fn created_at(&self) -> &i64 {
        &self.created_at
    }

    /// Returns the created_at timestamp for the PO
    pub fn workflow_id(&self) -> &str {
        &self.workflow_id
    }

    /// Returns the start_commit_num for the PO
    pub fn start_commit_num(&self) -> &i64 {
        &self.start_commit_num
    }

    /// Returns the end_commit_num for the PO
    pub fn end_commit_num(&self) -> &i64 {
        &self.end_commit_num
    }

    /// Returns the service_id for the PO
    pub fn service_id(&self) -> Option<&str> {
        self.service_id.as_deref()
    }
}

#[derive(Default, Clone)]
pub struct PurchaseOrderBuilder {
    purchase_order_uid: String,
    workflow_state: String,
    buyer_org_id: String,
    seller_org_id: String,
    is_closed: bool,
    accepted_version_id: Option<String>,
    versions: Vec<PurchaseOrderVersion>,
    alternate_ids: Vec<PurchaseOrderAlternateId>,
    created_at: i64,
    workflow_id: String,
    start_commit_num: i64,
    end_commit_num: i64,
    service_id: Option<String>,
}

impl PurchaseOrderBuilder {
    /// Sets the unique ID for this PO
    pub fn with_purchase_order_uid(mut self, uid: String) -> Self {
        self.purchase_order_uid = uid;
        self
    }

    /// Sets the workflow state for this PO
    pub fn with_workflow_state(mut self, status: String) -> Self {
        self.workflow_state = status;
        self
    }

    /// Sets the buyer's organization ID for this PO
    pub fn with_buyer_org_id(mut self, org_id: String) -> Self {
        self.buyer_org_id = org_id;
        self
    }

    /// Sets the seller's organization ID for this PO
    pub fn with_seller_org_id(mut self, org_id: String) -> Self {
        self.seller_org_id = org_id;
        self
    }

    /// Sets the is_closed value for this PO
    pub fn with_is_closed(mut self, is_closed: bool) -> Self {
        self.is_closed = is_closed;
        self
    }

    /// Sets the accepted version for this PO
    pub fn with_accepted_version_id(mut self, version_id: String) -> Self {
        self.accepted_version_id = Some(version_id);
        self
    }

    /// Sets the versions list for this PO
    pub fn with_versions(mut self, versions: Vec<PurchaseOrderVersion>) -> Self {
        self.versions = versions;
        self
    }

    /// Sets the alternate IDs list for this PO
    pub fn with_alternate_ids(mut self, alternate_ids: Vec<PurchaseOrderAlternateId>) -> Self {
        self.alternate_ids = alternate_ids;
        self
    }

    /// Sets the created_at timestamp for this PO
    pub fn with_created_at(mut self, created_at: i64) -> Self {
        self.created_at = created_at;
        self
    }

    /// Sets the workflow type for this PO
    pub fn with_workflow_id(mut self, workflow_id: String) -> Self {
        self.workflow_id = workflow_id;
        self
    }

    /// Sets the start commit number for this PO
    pub fn with_start_commit_number(mut self, start_commit_num: i64) -> Self {
        self.start_commit_num = start_commit_num;
        self
    }

    /// Sets the end commit number for this PO
    pub fn with_end_commit_number(mut self, end_commit_num: i64) -> Self {
        self.end_commit_num = end_commit_num;
        self
    }

    /// Sets the service ID for this PO
    pub fn with_service_id(mut self, service_id: Option<String>) -> Self {
        self.service_id = service_id;
        self
    }

    pub fn build(self) -> Result<PurchaseOrder, PurchaseOrderBuilderError> {
        let PurchaseOrderBuilder {
            purchase_order_uid,
            workflow_state,
            buyer_org_id,
            seller_org_id,
            is_closed,
            accepted_version_id,
            versions,
            alternate_ids,
            created_at,
            workflow_id,
            start_commit_num,
            end_commit_num,
            service_id,
        } = self;

        if purchase_order_uid.is_empty() {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "purchase_order_uid".to_string(),
            ));
        };

        if buyer_org_id.is_empty() {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "buyer_org_id".to_string(),
            ));
        };

        if seller_org_id.is_empty() {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "seller_org_id".to_string(),
            ));
        };

        if workflow_state.is_empty() {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "workflow_state".to_string(),
            ));
        };

        if workflow_id.is_empty() {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "workflow_id".to_string(),
            ));
        };

        if start_commit_num >= end_commit_num {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "start_commit_number must be less than end_commit_num".to_string(),
            ));
        };

        if end_commit_num <= start_commit_num {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "end_commit_number must be greater than start_commit_num".to_string(),
            ));
        };

        Ok(PurchaseOrder {
            purchase_order_uid,
            workflow_state,
            buyer_org_id,
            seller_org_id,
            is_closed,
            accepted_version_id,
            versions,
            alternate_ids,
            created_at,
            workflow_id,
            start_commit_num,
            end_commit_num,
            service_id,
        })
    }
}

/// Represents a list of Grid Purchase Order Versions
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PurchaseOrderVersionList {
    pub data: Vec<PurchaseOrderVersion>,
    pub paging: Paging,
}

impl PurchaseOrderVersionList {
    pub fn new(data: Vec<PurchaseOrderVersion>, paging: Paging) -> Self {
        Self { data, paging }
    }
}

/// Represents a Grid Purchase Order Version
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PurchaseOrderVersion {
    version_id: String,
    is_draft: bool,
    current_revision_id: i64,
    revisions: Vec<PurchaseOrderVersionRevision>,
    workflow_state: String,
    start_commit_num: i64,
    end_commit_num: i64,
    service_id: Option<String>,
}

impl PurchaseOrderVersion {
    /// Returns the version ID for the PO version
    pub fn version_id(&self) -> &str {
        &self.version_id
    }

    /// Returns the draft status for the PO version
    pub fn is_draft(&self) -> bool {
        self.is_draft
    }

    /// Returns the current revision ID for the PO version
    pub fn current_revision_id(&self) -> &i64 {
        &self.current_revision_id
    }

    /// Returns the revisions list for the PO version
    pub fn revisions(&self) -> Vec<PurchaseOrderVersionRevision> {
        self.revisions.to_vec()
    }

    /// Returns the workflow state of the PO version
    pub fn workflow_state(&self) -> &str {
        &self.workflow_state
    }

    /// Returns the start_commit_num for the PO version
    pub fn start_commit_num(&self) -> &i64 {
        &self.start_commit_num
    }

    /// Returns the end_commit_num for the PO version
    pub fn end_commit_num(&self) -> &i64 {
        &self.end_commit_num
    }

    /// Returns the service_id for the PO version
    pub fn service_id(&self) -> Option<&str> {
        self.service_id.as_deref()
    }
}

#[derive(Default, Clone)]
pub struct PurchaseOrderVersionBuilder {
    version_id: String,
    is_draft: bool,
    current_revision_id: i64,
    revisions: Vec<PurchaseOrderVersionRevision>,
    workflow_state: String,
    start_commit_num: i64,
    end_commit_num: i64,
    service_id: Option<String>,
}

impl PurchaseOrderVersionBuilder {
    /// Sets the version ID for this PO version
    pub fn with_version_id(mut self, version_id: String) -> Self {
        self.version_id = version_id;
        self
    }

    /// Sets the is_draft value for this PO version
    pub fn with_is_draft(mut self, is_draft: bool) -> Self {
        self.is_draft = is_draft;
        self
    }

    /// Sets the current revision ID for this PO version
    pub fn with_current_revision_id(mut self, revision_id: i64) -> Self {
        self.current_revision_id = revision_id;
        self
    }

    /// Sets the revisions list for this PO version
    pub fn with_revisions(mut self, revisions: Vec<PurchaseOrderVersionRevision>) -> Self {
        self.revisions = revisions;
        self
    }

    /// Sets the workflow state for this PO version
    pub fn with_workflow_state(mut self, workflow_state: String) -> Self {
        self.workflow_state = workflow_state;
        self
    }

    /// Sets the start commit number for this PO version
    pub fn with_start_commit_number(mut self, start_commit_num: i64) -> Self {
        self.start_commit_num = start_commit_num;
        self
    }

    /// Sets the end commit number for this PO version
    pub fn with_end_commit_number(mut self, end_commit_num: i64) -> Self {
        self.end_commit_num = end_commit_num;
        self
    }

    /// Sets the service ID for this PO version
    pub fn with_service_id(mut self, service_id: Option<String>) -> Self {
        self.service_id = service_id;
        self
    }

    pub fn build(self) -> Result<PurchaseOrderVersion, PurchaseOrderBuilderError> {
        let PurchaseOrderVersionBuilder {
            version_id,
            is_draft,
            current_revision_id,
            revisions,
            workflow_state,
            start_commit_num,
            end_commit_num,
            service_id,
        } = self;

        if version_id.is_empty() {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "version_id".to_string(),
            ));
        };

        if current_revision_id <= 0 {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "current_revision_id must be greater than 0".to_string(),
            ));
        };

        if revisions.is_empty() {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "revisions".to_string(),
            ));
        };

        if workflow_state.is_empty() {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "workflow_state".to_string(),
            ));
        };

        if start_commit_num >= end_commit_num {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "start_commit_number must be less than end_commit_num".to_string(),
            ));
        };

        if end_commit_num <= start_commit_num {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "end_commit_number must be greater than start_commit_num".to_string(),
            ));
        };

        Ok(PurchaseOrderVersion {
            version_id,
            is_draft,
            current_revision_id,
            revisions,
            workflow_state,
            start_commit_num,
            end_commit_num,
            service_id,
        })
    }
}

/// Represents a list of Grid Purchase Order Revisions
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PurchaseOrderVersionRevisionList {
    pub data: Vec<PurchaseOrderVersionRevision>,
    pub paging: Paging,
}

impl PurchaseOrderVersionRevisionList {
    pub fn new(data: Vec<PurchaseOrderVersionRevision>, paging: Paging) -> Self {
        Self { data, paging }
    }
}

/// Represents a Grid Purchase Order Version Revision
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PurchaseOrderVersionRevision {
    pub revision_id: i64,
    pub order_xml_v3_4: String,
    pub submitter: String,
    pub created_at: i64,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

impl PurchaseOrderVersionRevision {
    /// Returns the revision ID for the revision
    pub fn revision_id(&self) -> &i64 {
        &self.revision_id
    }

    /// Returns the order XML for the revision
    pub fn order_xml_v3_4(&self) -> &str {
        &self.order_xml_v3_4
    }

    /// Returns the submitter for the revision
    pub fn submitter(&self) -> &str {
        &self.submitter
    }

    /// Returns the created_at timestamp for the revision
    pub fn created_at(&self) -> i64 {
        self.created_at
    }

    /// Returns the start_commit_num for the revision
    pub fn start_commit_num(&self) -> &i64 {
        &self.start_commit_num
    }

    /// Returns the end_commit_num for the revision
    pub fn end_commit_num(&self) -> &i64 {
        &self.end_commit_num
    }

    /// Returns the service_id for the revision
    pub fn service_id(&self) -> Option<&str> {
        self.service_id.as_deref()
    }
}

#[derive(Default, Clone)]
pub struct PurchaseOrderVersionRevisionBuilder {
    revision_id: i64,
    order_xml_v3_4: String,
    submitter: String,
    created_at: i64,
    start_commit_num: i64,
    end_commit_num: i64,
    service_id: Option<String>,
}

impl PurchaseOrderVersionRevisionBuilder {
    /// Sets the revision ID for this revision
    pub fn with_revision_id(mut self, revision_id: i64) -> Self {
        self.revision_id = revision_id;
        self
    }

    /// Sets the order XML v3.4 for this revision
    pub fn with_order_xml_v3_4(mut self, xml: String) -> Self {
        self.order_xml_v3_4 = xml;
        self
    }

    /// Sets the submitter for this revision
    pub fn with_submitter(mut self, submitter: String) -> Self {
        self.submitter = submitter;
        self
    }

    /// Sets the created_at timestamp for this revision
    pub fn with_created_at(mut self, created_at: i64) -> Self {
        self.created_at = created_at;
        self
    }

    /// Sets the start commit number for this revision
    pub fn with_start_commit_number(mut self, start_commit_num: i64) -> Self {
        self.start_commit_num = start_commit_num;
        self
    }

    /// Sets the end commit number for this revision
    pub fn with_end_commit_number(mut self, end_commit_num: i64) -> Self {
        self.end_commit_num = end_commit_num;
        self
    }

    /// Sets the service ID for this revision
    pub fn with_service_id(mut self, service_id: Option<String>) -> Self {
        self.service_id = service_id;
        self
    }

    pub fn build(self) -> Result<PurchaseOrderVersionRevision, PurchaseOrderBuilderError> {
        let PurchaseOrderVersionRevisionBuilder {
            revision_id,
            order_xml_v3_4,
            submitter,
            created_at,
            start_commit_num,
            end_commit_num,
            service_id,
        } = self;

        if revision_id <= 0 {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "revision_id must be greater than 0".to_string(),
            ));
        };

        if order_xml_v3_4.is_empty() {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "order_xml_v3_4".to_string(),
            ));
        };

        if submitter.is_empty() {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "submitter".to_string(),
            ));
        };

        if start_commit_num >= end_commit_num {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "start_commit_number must be less than end_commit_num".to_string(),
            ));
        };

        if end_commit_num <= start_commit_num {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "end_commit_number must be greater than start_commit_num".to_string(),
            ));
        };

        Ok(PurchaseOrderVersionRevision {
            revision_id,
            order_xml_v3_4,
            submitter,
            created_at,
            start_commit_num,
            end_commit_num,
            service_id,
        })
    }
}

/// Represents a list of Grid Purchase Order Alternate IDs
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PurchaseOrderAlternateIdList {
    pub data: Vec<PurchaseOrderAlternateId>,
    pub paging: Paging,
}

impl PurchaseOrderAlternateIdList {
    pub fn new(data: Vec<PurchaseOrderAlternateId>, paging: Paging) -> Self {
        Self { data, paging }
    }
}

/// Represents a Grid Purchase Order Alternate ID
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PurchaseOrderAlternateId {
    purchase_order_uid: String,
    id_type: String,
    id: String,
    start_commit_num: i64,
    end_commit_num: i64,
    service_id: Option<String>,
}

impl PurchaseOrderAlternateId {
    /// Returns the purchase order UID for the PO alternate ID
    pub fn purchase_order_uid(&self) -> &str {
        &self.purchase_order_uid
    }

    /// Returns the ID type for the PO alternate ID
    pub fn id_type(&self) -> &str {
        &self.id_type
    }

    /// Returns the ID for the PO alternate ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the start_commit_num for the PO alternate ID
    pub fn start_commit_num(&self) -> &i64 {
        &self.start_commit_num
    }

    /// Returns the end_commit_num for the PO alternate ID
    pub fn end_commit_num(&self) -> &i64 {
        &self.end_commit_num
    }

    /// Returns the service_id for the PO alternate ID
    pub fn service_id(&self) -> Option<&str> {
        self.service_id.as_deref()
    }
}

#[derive(Default, Clone)]
pub struct PurchaseOrderAlternateIdBuilder {
    purchase_order_uid: String,
    id_type: String,
    id: String,
    start_commit_num: i64,
    end_commit_num: i64,
    service_id: Option<String>,
}

impl PurchaseOrderAlternateIdBuilder {
    /// Sets the purchase order UID for this alternate ID
    pub fn with_purchase_order_uid(mut self, uid: String) -> Self {
        self.purchase_order_uid = uid;
        self
    }

    /// Sets the ID type for this alternate ID
    pub fn with_id_type(mut self, id_type: String) -> Self {
        self.id_type = id_type;
        self
    }

    /// Sets the ID for this alternate ID
    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    /// Sets the start commit number for this alternate ID
    pub fn with_start_commit_number(mut self, start_commit_num: i64) -> Self {
        self.start_commit_num = start_commit_num;
        self
    }

    /// Sets the end commit number for this alternate ID
    pub fn with_end_commit_number(mut self, end_commit_num: i64) -> Self {
        self.end_commit_num = end_commit_num;
        self
    }

    /// Sets the service ID for this alternate ID
    pub fn with_service_id(mut self, service_id: Option<String>) -> Self {
        self.service_id = service_id;
        self
    }

    pub fn build(self) -> Result<PurchaseOrderAlternateId, PurchaseOrderBuilderError> {
        let PurchaseOrderAlternateIdBuilder {
            purchase_order_uid,
            id_type,
            id,
            start_commit_num,
            end_commit_num,
            service_id,
        } = self;

        if purchase_order_uid.is_empty() {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "purchase_order_uid".to_string(),
            ));
        };

        if id_type.is_empty() {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "id_type".to_string(),
            ));
        };

        if id.is_empty() {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "id".to_string(),
            ));
        };

        if start_commit_num >= end_commit_num {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "start_commit_number must be less than end_commit_num".to_string(),
            ));
        };

        if end_commit_num <= start_commit_num {
            return Err(PurchaseOrderBuilderError::MissingRequiredField(
                "end_commit_number must be greater than start_commit_num".to_string(),
            ));
        };

        Ok(PurchaseOrderAlternateId {
            purchase_order_uid,
            id_type,
            id,
            start_commit_num,
            end_commit_num,
            service_id,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListPOFilters {
    pub buyer_org_id: Option<String>,
    pub seller_org_id: Option<String>,
    pub has_accepted_version: Option<bool>,
    pub is_open: Option<bool>,
    // Comma separated list of alternate IDs in the format <id_type>:<id>
    pub alternate_ids: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListVersionFilters {
    pub is_accepted: Option<bool>,
    pub is_draft: Option<bool>,
}

pub trait PurchaseOrderStore {
    /// Adds a purchase order to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `order` - The purchase order to be added
    fn add_purchase_order(&self, order: PurchaseOrder) -> Result<(), PurchaseOrderStoreError>;

    /// Lists purchase orders from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `filters` - Optional filters for the POs: `buyer_org_id`,
    ///    `seller_org_id`, `has_accepted_version`, `is_open`, and
    ///    `alternate_ids`
    ///  * `service_id` - The service ID
    ///  * `offset` - The index of the first in storage to retrieve
    ///  * `limit` - The number of items to retrieve from the offset
    fn list_purchase_orders(
        &self,
        filters: ListPOFilters,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderList, PurchaseOrderStoreError>;

    /// Lists purchase order versions from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `po_uid`   - The uid of the purchase order to get versions for
    ///  * `filters` - Optional filters for the PO versions: `is_accepted` and
    ///    `is_draft`
    ///  * `service_id` - The service ID
    ///  * `offset` - The index of the first in storage to retrieve
    ///  * `limit` - The number of items to retrieve from the offset
    fn list_purchase_order_versions(
        &self,
        po_uid: &str,
        filters: ListVersionFilters,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderVersionList, PurchaseOrderStoreError>;

    /// Fetches a purchase order from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `purchase_order_uid`   - The uid of the purchase order
    ///  * `version_id` - Optional filter for version
    ///  * `revision_number` - Optional filter for version revision
    ///  * `service_id` - The service id
    fn get_purchase_order(
        &self,
        purchase_order_uid: &str,
        version_id: Option<&str>,
        revision_number: Option<i64>,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrder>, PurchaseOrderStoreError>;

    /// Fetches a purchase order version from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `po_uid`    - The uid of the purchase order the version belongs to
    ///  * `version_id` - The ID of the version to fetch
    ///  * `service_id` - The service ID
    fn get_purchase_order_version(
        &self,
        po_uid: &str,
        version_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrderVersion>, PurchaseOrderStoreError>;

    /// Fetches a purchase order revision from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `po_uid`    - The uid of the purchase order the revision belongs to
    ///  * `version_id` - The ID of the version the revision is for
    ///  * `revision_id` - The ID of the revision to fetch
    ///  * `service_id` - The service ID
    fn get_purchase_order_revision(
        &self,
        po_uid: &str,
        version_id: &str,
        revision_id: &i64,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrderVersionRevision>, PurchaseOrderStoreError>;

    /// Lists purchase order revisions from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `po_uid`    - The uid of the purchase order the revisions belongs to
    ///  * `version_id` - The ID of the version the revisions are for
    ///  * `service_id` - The service ID
    ///  * `offset` - The index of the first in storage to retrieve
    ///  * `limit` - The number of items to retrieve from the offset
    fn list_purchase_order_revisions(
        &self,
        po_uid: &str,
        version_id: &str,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderVersionRevisionList, PurchaseOrderStoreError>;

    /// Lists alternate IDs for a purchase order from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `purchase_order_uid` - The purchase order to fetch alternate IDs for
    ///  * `service_id` - The service id
    ///  * `offset` - The index of the first in storage to retrieve
    ///  * `limit` - The number of items to retrieve from the offset
    fn list_alternate_ids_for_purchase_order(
        &self,
        purchase_order_uid: &str,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderAlternateIdList, PurchaseOrderStoreError>;

    /// Fetches a latest revision ID for a purchase order version from the
    /// underlying storage
    ///
    /// # Arguments
    ///
    ///  * `purchase_order_uid` - The UID of the purchase order the revision belongs to
    ///  * `version_id` - The ID of the version the revision is for
    ///  * `service_id` - The service ID
    fn get_latest_revision_id(
        &self,
        purchase_order_uid: &str,
        version_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<i64>, PurchaseOrderStoreError>;
}

impl<PS> PurchaseOrderStore for Box<PS>
where
    PS: PurchaseOrderStore + ?Sized,
{
    fn add_purchase_order(&self, order: PurchaseOrder) -> Result<(), PurchaseOrderStoreError> {
        (**self).add_purchase_order(order)
    }

    fn list_purchase_orders(
        &self,
        filters: ListPOFilters,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderList, PurchaseOrderStoreError> {
        (**self).list_purchase_orders(filters, service_id, offset, limit)
    }

    fn list_purchase_order_versions(
        &self,
        po_uid: &str,
        filters: ListVersionFilters,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderVersionList, PurchaseOrderStoreError> {
        (**self).list_purchase_order_versions(po_uid, filters, service_id, offset, limit)
    }

    fn get_purchase_order(
        &self,
        purchase_order_uid: &str,
        version_id: Option<&str>,
        revision_number: Option<i64>,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrder>, PurchaseOrderStoreError> {
        (**self).get_purchase_order(purchase_order_uid, version_id, revision_number, service_id)
    }

    fn get_purchase_order_version(
        &self,
        po_uid: &str,
        version_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrderVersion>, PurchaseOrderStoreError> {
        (**self).get_purchase_order_version(po_uid, version_id, service_id)
    }

    fn get_purchase_order_revision(
        &self,
        po_uid: &str,
        version_id: &str,
        revision_id: &i64,
        service_id: Option<&str>,
    ) -> Result<Option<PurchaseOrderVersionRevision>, PurchaseOrderStoreError> {
        (**self).get_purchase_order_revision(po_uid, version_id, revision_id, service_id)
    }

    fn list_purchase_order_revisions(
        &self,
        po_uid: &str,
        version_id: &str,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderVersionRevisionList, PurchaseOrderStoreError> {
        (**self).list_purchase_order_revisions(po_uid, version_id, service_id, offset, limit)
    }

    fn list_alternate_ids_for_purchase_order(
        &self,
        purchase_order_uid: &str,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderAlternateIdList, PurchaseOrderStoreError> {
        (**self).list_alternate_ids_for_purchase_order(
            purchase_order_uid,
            service_id,
            offset,
            limit,
        )
    }

    fn get_latest_revision_id(
        &self,
        purchase_order_uid: &str,
        version_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<i64>, PurchaseOrderStoreError> {
        (**self).get_latest_revision_id(purchase_order_uid, version_id, service_id)
    }
}
