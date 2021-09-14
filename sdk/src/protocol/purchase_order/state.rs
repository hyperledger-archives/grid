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

//! Protocol structs for Purchase Order state

use protobuf::Message;
use protobuf::RepeatedField;

use std::error::Error as StdError;

use crate::protos::purchase_order_state;
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};

/// Native representation of a `PurchaseOrderRevision`
///
/// The purchase order revision contains the editable fields of a purchase order
#[derive(Debug, Clone, PartialEq)]
pub struct PurchaseOrderRevision {
    revision_id: String,
    submitter: String,
    created_at: u64,
    order_xml_v3_4: String,
}

impl PurchaseOrderRevision {
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

    pub fn into_builder(self) -> PurchaseOrderRevisionBuilder {
        PurchaseOrderRevisionBuilder::new()
            .with_revision_id(self.revision_id)
            .with_submitter(self.submitter)
            .with_created_at(self.created_at)
            .with_order_xml_v3_4(self.order_xml_v3_4)
    }
}

impl FromProto<purchase_order_state::PurchaseOrderRevision> for PurchaseOrderRevision {
    fn from_proto(
        mut revision: purchase_order_state::PurchaseOrderRevision,
    ) -> Result<Self, ProtoConversionError> {
        Ok(PurchaseOrderRevision {
            revision_id: revision.take_revision_id(),
            submitter: revision.take_submitter(),
            created_at: revision.get_created_at(),
            order_xml_v3_4: revision.take_order_xml_v3_4(),
        })
    }
}

impl FromNative<PurchaseOrderRevision> for purchase_order_state::PurchaseOrderRevision {
    fn from_native(revision: PurchaseOrderRevision) -> Result<Self, ProtoConversionError> {
        let mut proto = purchase_order_state::PurchaseOrderRevision::new();
        proto.set_revision_id(revision.revision_id().to_string());
        proto.set_submitter(revision.submitter().to_string());
        proto.set_created_at(revision.created_at());
        proto.set_order_xml_v3_4(revision.order_xml_v3_4().to_string());

        Ok(proto)
    }
}

impl IntoBytes for PurchaseOrderRevision {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from PurchaseOrderRevision".to_string(),
            )
        })?;

        Ok(bytes)
    }
}

impl IntoProto<purchase_order_state::PurchaseOrderRevision> for PurchaseOrderRevision {}
impl IntoNative<PurchaseOrderRevision> for purchase_order_state::PurchaseOrderRevision {}

/// Returned if any required fields in a `PurchaseOrderRevision` are not present when being
/// converted from the corresponding builder
#[derive(Debug)]
pub enum PurchaseOrderRevisionBuildError {
    MissingField(String),
}

impl StdError for PurchaseOrderRevisionBuildError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            PurchaseOrderRevisionBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for PurchaseOrderRevisionBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            PurchaseOrderRevisionBuildError::MissingField(ref s) => {
                write!(f, "missing field \"{}\"", s)
            }
        }
    }
}

/// Builder used to create a `PurchaseOrderRevision`
#[derive(Default, Clone, PartialEq)]
pub struct PurchaseOrderRevisionBuilder {
    revision_id: Option<String>,
    submitter: Option<String>,
    created_at: Option<u64>,
    order_xml_v3_4: Option<String>,
}

impl PurchaseOrderRevisionBuilder {
    pub fn new() -> Self {
        PurchaseOrderRevisionBuilder::default()
    }

    pub fn with_revision_id(mut self, revision_id: String) -> Self {
        self.revision_id = Some(revision_id);
        self
    }

    pub fn with_submitter(mut self, submitter: String) -> Self {
        self.submitter = Some(submitter);
        self
    }

    pub fn with_created_at(mut self, created_at: u64) -> Self {
        self.created_at = Some(created_at);
        self
    }

    pub fn with_order_xml_v3_4(mut self, order_xml_v3_4: String) -> Self {
        self.order_xml_v3_4 = Some(order_xml_v3_4);
        self
    }

    pub fn build(self) -> Result<PurchaseOrderRevision, PurchaseOrderRevisionBuildError> {
        let revision_id = self.revision_id.ok_or_else(|| {
            PurchaseOrderRevisionBuildError::MissingField(
                "'revision_id' field is required".to_string(),
            )
        })?;

        let submitter = self.submitter.ok_or_else(|| {
            PurchaseOrderRevisionBuildError::MissingField(
                "'submitter' field is required".to_string(),
            )
        })?;

        let created_at = self.created_at.ok_or_else(|| {
            PurchaseOrderRevisionBuildError::MissingField(
                "'created_at' field is required".to_string(),
            )
        })?;

        let order_xml_v3_4 = self.order_xml_v3_4.ok_or_else(|| {
            PurchaseOrderRevisionBuildError::MissingField(
                "'order_xml_v3_4' field is required".to_string(),
            )
        })?;

        Ok(PurchaseOrderRevision {
            revision_id,
            submitter,
            created_at,
            order_xml_v3_4,
        })
    }
}

/// Native representation of a `PurchaseOrderVersion`
///
/// A purchase order version is created everytime updates are made to the purchase order, requiring
/// a new version of the original
#[derive(Debug, Clone, PartialEq)]
pub struct PurchaseOrderVersion {
    version_id: String,
    workflow_status: String,
    is_draft: bool,
    current_revision_id: String,
    revisions: PurchaseOrderRevision,
}

impl PurchaseOrderVersion {
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

    pub fn revisions(&self) -> &PurchaseOrderRevision {
        &self.revisions
    }

    pub fn into_builder(self) -> PurchaseOrderVersionBuilder {
        PurchaseOrderVersionBuilder::new()
            .with_version_id(self.version_id)
            .with_workflow_status(self.workflow_status)
            .with_is_draft(self.is_draft)
            .with_current_revision_id(self.current_revision_id)
            .with_revisions(self.revisions)
    }
}

impl FromProto<purchase_order_state::PurchaseOrderVersion> for PurchaseOrderVersion {
    fn from_proto(
        mut version: purchase_order_state::PurchaseOrderVersion,
    ) -> Result<Self, ProtoConversionError> {
        Ok(PurchaseOrderVersion {
            version_id: version.take_version_id(),
            workflow_status: version.take_workflow_status(),
            is_draft: version.get_is_draft(),
            current_revision_id: version.take_current_revision_id(),
            revisions: PurchaseOrderRevision::from_proto(version.take_revisions())?,
        })
    }
}

impl FromNative<PurchaseOrderVersion> for purchase_order_state::PurchaseOrderVersion {
    fn from_native(version: PurchaseOrderVersion) -> Result<Self, ProtoConversionError> {
        let mut proto = purchase_order_state::PurchaseOrderVersion::new();
        proto.set_version_id(version.version_id().to_string());
        proto.set_workflow_status(version.workflow_status().to_string());
        proto.set_is_draft(version.is_draft());
        proto.set_current_revision_id(version.current_revision_id().to_string());
        proto.set_revisions(version.revisions().clone().into_proto()?);

        Ok(proto)
    }
}

impl IntoBytes for PurchaseOrderVersion {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from PurchaseOrderVersion".to_string(),
            )
        })?;

        Ok(bytes)
    }
}

impl IntoProto<purchase_order_state::PurchaseOrderVersion> for PurchaseOrderVersion {}
impl IntoNative<PurchaseOrderVersion> for purchase_order_state::PurchaseOrderVersion {}

/// Returned if any required fields in a `PurchaseOrderVersion` are not present when being
/// converted from the corresponding builder
#[derive(Debug)]
pub enum PurchaseOrderVersionBuildError {
    MissingField(String),
}

impl StdError for PurchaseOrderVersionBuildError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            PurchaseOrderVersionBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for PurchaseOrderVersionBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            PurchaseOrderVersionBuildError::MissingField(ref s) => {
                write!(f, "missing field \"{}\"", s)
            }
        }
    }
}

/// Builder used to create a `PurchaseOrderVersion`
#[derive(Default, Clone, PartialEq)]
pub struct PurchaseOrderVersionBuilder {
    version_id: Option<String>,
    workflow_status: Option<String>,
    is_draft: Option<bool>,
    current_revision_id: Option<String>,
    revisions: Option<PurchaseOrderRevision>,
}

impl PurchaseOrderVersionBuilder {
    pub fn new() -> Self {
        PurchaseOrderVersionBuilder::default()
    }

    pub fn with_version_id(mut self, version_id: String) -> Self {
        self.version_id = Some(version_id);
        self
    }

    pub fn with_workflow_status(mut self, workflow_status: String) -> Self {
        self.workflow_status = Some(workflow_status);
        self
    }

    pub fn with_is_draft(mut self, is_draft: bool) -> Self {
        self.is_draft = Some(is_draft);
        self
    }

    pub fn with_current_revision_id(mut self, current_revision_id: String) -> Self {
        self.current_revision_id = Some(current_revision_id);
        self
    }

    pub fn with_revisions(mut self, revisions: PurchaseOrderRevision) -> Self {
        self.revisions = Some(revisions);
        self
    }

    pub fn build(self) -> Result<PurchaseOrderVersion, PurchaseOrderVersionBuildError> {
        let version_id = self.version_id.ok_or_else(|| {
            PurchaseOrderVersionBuildError::MissingField(
                "'version_id' field is required".to_string(),
            )
        })?;

        let workflow_status = self.workflow_status.ok_or_else(|| {
            PurchaseOrderVersionBuildError::MissingField(
                "'workflow_status' field is required".to_string(),
            )
        })?;

        let is_draft = self.is_draft.ok_or_else(|| {
            PurchaseOrderVersionBuildError::MissingField("'is_draft' field is required".to_string())
        })?;

        let current_revision_id = self.current_revision_id.ok_or_else(|| {
            PurchaseOrderVersionBuildError::MissingField(
                "'current_revision_id' field is required".to_string(),
            )
        })?;

        let revisions = self.revisions.ok_or_else(|| {
            PurchaseOrderVersionBuildError::MissingField(
                "'revisions' field is required".to_string(),
            )
        })?;

        Ok(PurchaseOrderVersion {
            version_id,
            workflow_status,
            is_draft,
            current_revision_id,
            revisions,
        })
    }
}

/// Native representation of a `PurchaseOrder`
///
/// Purchase orders in real-life trade scenarios are represented by `PurchaseOrder`
#[derive(Debug, Clone, PartialEq)]
pub struct PurchaseOrder {
    uid: String,
    workflow_status: String,
    versions: Vec<PurchaseOrderVersion>,
    accepted_version_number: String,
    created_at: u64,
    is_closed: bool,
    buyer_org_id: String,
    seller_org_id: String,
}

impl PurchaseOrder {
    pub fn uid(&self) -> &str {
        &self.uid
    }

    pub fn workflow_status(&self) -> &str {
        &self.workflow_status
    }

    pub fn versions(&self) -> &[PurchaseOrderVersion] {
        &self.versions
    }

    pub fn accepted_version_number(&self) -> &str {
        &self.accepted_version_number
    }

    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    pub fn buyer_org_id(&self) -> &str {
        &self.buyer_org_id
    }

    pub fn seller_org_id(&self) -> &str {
        &self.seller_org_id
    }

    pub fn into_builder(self) -> PurchaseOrderBuilder {
        PurchaseOrderBuilder::new()
            .with_uid(self.uid)
            .with_workflow_status(self.workflow_status)
            .with_versions(self.versions)
            .with_accepted_version_number(self.accepted_version_number)
            .with_created_at(self.created_at)
            .with_is_closed(self.is_closed)
            .with_buyer_org_id(self.buyer_org_id)
            .with_seller_org_id(self.seller_org_id)
    }
}

impl FromProto<purchase_order_state::PurchaseOrder> for PurchaseOrder {
    fn from_proto(
        mut order: purchase_order_state::PurchaseOrder,
    ) -> Result<Self, ProtoConversionError> {
        Ok(PurchaseOrder {
            uid: order.take_uid(),
            workflow_status: order.take_workflow_status(),
            versions: order
                .take_versions()
                .into_iter()
                .map(PurchaseOrderVersion::from_proto)
                .collect::<Result<_, _>>()?,
            accepted_version_number: order.take_accepted_version_number(),
            created_at: order.get_created_at(),
            is_closed: order.get_is_closed(),
            buyer_org_id: order.take_buyer_org_id(),
            seller_org_id: order.take_seller_org_id(),
        })
    }
}

impl FromNative<PurchaseOrder> for purchase_order_state::PurchaseOrder {
    fn from_native(order: PurchaseOrder) -> Result<Self, ProtoConversionError> {
        let mut proto = purchase_order_state::PurchaseOrder::new();
        proto.set_uid(order.uid().to_string());
        proto.set_workflow_status(order.workflow_status().to_string());
        proto.set_versions(RepeatedField::from_vec(
            order
                .versions()
                .to_vec()
                .into_iter()
                .map(|version| version.into_proto())
                .collect::<Result<_, _>>()?,
        ));
        proto.set_accepted_version_number(order.accepted_version_number().to_string());
        proto.set_created_at(order.created_at());
        proto.set_is_closed(order.is_closed());
        proto.set_buyer_org_id(order.buyer_org_id().to_string());
        proto.set_seller_org_id(order.seller_org_id().to_string());

        Ok(proto)
    }
}

impl IntoBytes for PurchaseOrder {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from PurchaseOrder".to_string(),
            )
        })?;

        Ok(bytes)
    }
}

impl IntoProto<purchase_order_state::PurchaseOrder> for PurchaseOrder {}
impl IntoNative<PurchaseOrder> for purchase_order_state::PurchaseOrder {}

/// Returned if any required fields in a `PurchaseOrder` are not present when being converted from
/// the corresponding builder
#[derive(Debug)]
pub enum PurchaseOrderBuildError {
    MissingField(String),
    EmptyVec(String),
}

impl StdError for PurchaseOrderBuildError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            PurchaseOrderBuildError::MissingField(_) => None,
            PurchaseOrderBuildError::EmptyVec(_) => None,
        }
    }
}

impl std::fmt::Display for PurchaseOrderBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            PurchaseOrderBuildError::MissingField(ref s) => write!(f, "missing field \"{}\"", s),
            PurchaseOrderBuildError::EmptyVec(ref s) => write!(f, "\"{}\" must not be empty", s),
        }
    }
}

/// Builder used to create a `PurchaseOrder`
#[derive(Default, Clone, PartialEq)]
pub struct PurchaseOrderBuilder {
    uid: Option<String>,
    workflow_status: Option<String>,
    versions: Option<Vec<PurchaseOrderVersion>>,
    accepted_version_number: Option<String>,
    created_at: Option<u64>,
    is_closed: Option<bool>,
    buyer_org_id: Option<String>,
    seller_org_id: Option<String>,
}

impl PurchaseOrderBuilder {
    pub fn new() -> Self {
        PurchaseOrderBuilder::default()
    }

    pub fn with_uid(mut self, uid: String) -> Self {
        self.uid = Some(uid);
        self
    }

    pub fn with_workflow_status(mut self, workflow_status: String) -> Self {
        self.workflow_status = Some(workflow_status);
        self
    }

    pub fn with_versions(mut self, versions: Vec<PurchaseOrderVersion>) -> Self {
        self.versions = Some(versions);
        self
    }

    pub fn with_accepted_version_number(mut self, accepted_version_number: String) -> Self {
        self.accepted_version_number = Some(accepted_version_number);
        self
    }

    pub fn with_created_at(mut self, created_at: u64) -> Self {
        self.created_at = Some(created_at);
        self
    }

    pub fn with_is_closed(mut self, is_closed: bool) -> Self {
        self.is_closed = Some(is_closed);
        self
    }

    pub fn with_buyer_org_id(mut self, buyer: String) -> Self {
        self.buyer_org_id = Some(buyer);
        self
    }

    pub fn with_seller_org_id(mut self, seller: String) -> Self {
        self.seller_org_id = Some(seller);
        self
    }

    pub fn build(self) -> Result<PurchaseOrder, PurchaseOrderBuildError> {
        let uid = self.uid.ok_or_else(|| {
            PurchaseOrderBuildError::MissingField("'uid' field is required".to_string())
        })?;

        let workflow_status = self.workflow_status.ok_or_else(|| {
            PurchaseOrderBuildError::MissingField("'workflow_status' field is required".to_string())
        })?;

        let versions = self.versions.ok_or_else(|| {
            PurchaseOrderBuildError::EmptyVec("'versions' field is required".to_string())
        })?;

        let accepted_version_number = self.accepted_version_number.ok_or_else(|| {
            PurchaseOrderBuildError::MissingField(
                "'accepted_version_number' field is required".to_string(),
            )
        })?;

        let created_at = self.created_at.ok_or_else(|| {
            PurchaseOrderBuildError::MissingField("'created_at' field is required".to_string())
        })?;

        let is_closed = self.is_closed.ok_or_else(|| {
            PurchaseOrderBuildError::MissingField("'is_closed' field is required".to_string())
        })?;

        let buyer_org_id = self.buyer_org_id.ok_or_else(|| {
            PurchaseOrderBuildError::MissingField("'buyer_org_id' field is required".to_string())
        })?;

        let seller_org_id = self.seller_org_id.ok_or_else(|| {
            PurchaseOrderBuildError::MissingField("'seller_org_id' field is required".to_string())
        })?;

        Ok(PurchaseOrder {
            uid,
            workflow_status,
            versions,
            accepted_version_number,
            created_at,
            is_closed,
            buyer_org_id,
            seller_org_id,
        })
    }
}

/// Native representation of a list of `PurchaseOrder`s
#[derive(Debug, Clone, PartialEq)]
pub struct PurchaseOrderList {
    purchase_orders: Vec<PurchaseOrder>,
}

impl PurchaseOrderList {
    pub fn purchase_orders(&self) -> &[PurchaseOrder] {
        &self.purchase_orders
    }

    pub fn into_builder(self) -> PurchaseOrderListBuilder {
        PurchaseOrderListBuilder::new().with_purchase_orders(self.purchase_orders)
    }
}

impl FromProto<purchase_order_state::PurchaseOrderList> for PurchaseOrderList {
    fn from_proto(
        order_list: purchase_order_state::PurchaseOrderList,
    ) -> Result<Self, ProtoConversionError> {
        Ok(PurchaseOrderList {
            purchase_orders: order_list
                .get_purchase_orders()
                .to_vec()
                .into_iter()
                .map(PurchaseOrder::from_proto)
                .collect::<Result<Vec<PurchaseOrder>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<PurchaseOrderList> for purchase_order_state::PurchaseOrderList {
    fn from_native(order_list: PurchaseOrderList) -> Result<Self, ProtoConversionError> {
        let mut order_list_proto = purchase_order_state::PurchaseOrderList::new();

        order_list_proto.set_purchase_orders(RepeatedField::from_vec(
            order_list
                .purchase_orders()
                .to_vec()
                .into_iter()
                .map(PurchaseOrder::into_proto)
                .collect::<Result<Vec<purchase_order_state::PurchaseOrder>, ProtoConversionError>>(
                )?,
        ));

        Ok(order_list_proto)
    }
}

impl FromBytes<PurchaseOrderList> for PurchaseOrderList {
    fn from_bytes(bytes: &[u8]) -> Result<PurchaseOrderList, ProtoConversionError> {
        let proto: purchase_order_state::PurchaseOrderList = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get PurchaseOrderList from bytes".to_string(),
                )
            })?;

        proto.into_native()
    }
}

impl IntoBytes for PurchaseOrderList {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from PurchaseOrderList".to_string(),
            )
        })?;

        Ok(bytes)
    }
}

impl IntoProto<purchase_order_state::PurchaseOrderList> for PurchaseOrderList {}
impl IntoNative<PurchaseOrderList> for purchase_order_state::PurchaseOrderList {}

/// Builder used to create a list of `PurchaseOrder`s
#[derive(Default, Clone)]
pub struct PurchaseOrderListBuilder {
    purchase_orders: Option<Vec<PurchaseOrder>>,
}

impl PurchaseOrderListBuilder {
    pub fn new() -> Self {
        PurchaseOrderListBuilder::default()
    }

    pub fn with_purchase_orders(mut self, purchase_orders: Vec<PurchaseOrder>) -> Self {
        self.purchase_orders = Some(purchase_orders);
        self
    }

    pub fn build(self) -> Result<PurchaseOrderList, PurchaseOrderBuildError> {
        let purchase_orders = self
            .purchase_orders
            .ok_or_else(|| PurchaseOrderBuildError::MissingField("purchase_orders".to_string()))?;

        let purchase_orders = {
            if purchase_orders.is_empty() {
                return Err(PurchaseOrderBuildError::EmptyVec(
                    "purchase_orders".to_string(),
                ));
            } else {
                purchase_orders
            }
        };

        Ok(PurchaseOrderList { purchase_orders })
    }
}

/// Native representation of a `PurchaseOrderAlternateId`
///
/// An `AlternateId` is a separate identifier from the `PurchaseOrder`'s unique identifier and
/// associated `org_id`. This enables certain smart contracts to identify a `PurchaseOrder` within
/// its own context.
#[derive(Debug, Clone, PartialEq)]
pub struct PurchaseOrderAlternateId {
    id_type: String,
    id: String,
    org_id: String,
}

impl PurchaseOrderAlternateId {
    pub fn id_type(&self) -> &str {
        &self.id_type
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn org_id(&self) -> &str {
        &self.org_id
    }

    pub fn into_builder(self) -> PurchaseOrderAlternateIdBuilder {
        PurchaseOrderAlternateIdBuilder::new()
            .with_id_type(self.id_type)
            .with_id(self.id)
            .with_org_id(self.org_id)
    }
}

impl FromProto<purchase_order_state::PurchaseOrderAlternateId> for PurchaseOrderAlternateId {
    fn from_proto(
        mut alt_id: purchase_order_state::PurchaseOrderAlternateId,
    ) -> Result<Self, ProtoConversionError> {
        Ok(PurchaseOrderAlternateId {
            id_type: alt_id.take_id_type(),
            id: alt_id.take_id(),
            org_id: alt_id.take_org_id(),
        })
    }
}

impl FromNative<PurchaseOrderAlternateId> for purchase_order_state::PurchaseOrderAlternateId {
    fn from_native(alt_id: PurchaseOrderAlternateId) -> Result<Self, ProtoConversionError> {
        let mut proto = purchase_order_state::PurchaseOrderAlternateId::new();
        proto.set_id_type(alt_id.id_type().to_string());
        proto.set_id(alt_id.id().to_string());
        proto.set_org_id(alt_id.org_id().to_string());

        Ok(proto)
    }
}

impl IntoBytes for PurchaseOrderAlternateId {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from PurchaseOrderAlternateId".to_string(),
            )
        })?;

        Ok(bytes)
    }
}

impl IntoProto<purchase_order_state::PurchaseOrderAlternateId> for PurchaseOrderAlternateId {}
impl IntoNative<PurchaseOrderAlternateId> for purchase_order_state::PurchaseOrderAlternateId {}

/// Returned if any required fields in a `PurchaseOrderAlternateId` are not present when being
/// converted from the corresponding builder
#[derive(Debug)]
pub enum PurchaseOrderAlternateIdBuildError {
    MissingField(String),
    EmptyVec(String),
}

impl StdError for PurchaseOrderAlternateIdBuildError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            PurchaseOrderAlternateIdBuildError::MissingField(_) => None,
            PurchaseOrderAlternateIdBuildError::EmptyVec(_) => None,
        }
    }
}

impl std::fmt::Display for PurchaseOrderAlternateIdBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            PurchaseOrderAlternateIdBuildError::MissingField(ref s) => {
                write!(f, "missing field \"{}\"", s)
            }
            PurchaseOrderAlternateIdBuildError::EmptyVec(ref s) => {
                write!(f, "\"{}\" must not be empty", s)
            }
        }
    }
}

/// Builder used to create a `PurchaseOrderAlternateId`
#[derive(Default, Clone, PartialEq)]
pub struct PurchaseOrderAlternateIdBuilder {
    id_type: Option<String>,
    id: Option<String>,
    org_id: Option<String>,
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

    pub fn with_org_id(mut self, org_id: String) -> Self {
        self.org_id = Some(org_id);
        self
    }

    pub fn build(self) -> Result<PurchaseOrderAlternateId, PurchaseOrderAlternateIdBuildError> {
        let id_type = self.id_type.ok_or_else(|| {
            PurchaseOrderAlternateIdBuildError::MissingField(
                "'id_type' field is required".to_string(),
            )
        })?;

        let id = self.id.ok_or_else(|| {
            PurchaseOrderAlternateIdBuildError::MissingField("'id' field is required".to_string())
        })?;

        let org_id = self.org_id.ok_or_else(|| {
            PurchaseOrderAlternateIdBuildError::MissingField(
                "'org_id' field is required".to_string(),
            )
        })?;

        Ok(PurchaseOrderAlternateId {
            id_type,
            id,
            org_id,
        })
    }
}

/// Native representation of a list of `PurchaseOrderAlternateId`s
#[derive(Debug, Clone, PartialEq)]
pub struct PurchaseOrderAlternateIdList {
    alternate_ids: Vec<PurchaseOrderAlternateId>,
}

impl PurchaseOrderAlternateIdList {
    pub fn alternate_ids(&self) -> &[PurchaseOrderAlternateId] {
        &self.alternate_ids
    }

    pub fn into_builder(self) -> PurchaseOrderAlternateIdListBuilder {
        PurchaseOrderAlternateIdListBuilder::new().with_alternate_ids(self.alternate_ids)
    }
}

impl FromProto<purchase_order_state::PurchaseOrderAlternateIdList>
    for PurchaseOrderAlternateIdList
{
    fn from_proto(
        id_list: purchase_order_state::PurchaseOrderAlternateIdList,
    ) -> Result<Self, ProtoConversionError> {
        Ok(PurchaseOrderAlternateIdList {
            alternate_ids: id_list
                .get_alternate_ids()
                .to_vec()
                .into_iter()
                .map(PurchaseOrderAlternateId::from_proto)
                .collect::<Result<Vec<PurchaseOrderAlternateId>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<PurchaseOrderAlternateIdList>
    for purchase_order_state::PurchaseOrderAlternateIdList
{
    fn from_native(id_list: PurchaseOrderAlternateIdList) -> Result<Self, ProtoConversionError> {
        let mut id_list_proto = purchase_order_state::PurchaseOrderAlternateIdList::new();

        id_list_proto.set_alternate_ids(
            RepeatedField::from_vec(
                id_list
                    .alternate_ids()
                    .to_vec()
                    .into_iter()
                    .map(PurchaseOrderAlternateId::into_proto)
                    .collect::<Result<
                        Vec<purchase_order_state::PurchaseOrderAlternateId>,
                        ProtoConversionError,
                    >>()?,
            ),
        );

        Ok(id_list_proto)
    }
}

impl FromBytes<PurchaseOrderAlternateIdList> for PurchaseOrderAlternateIdList {
    fn from_bytes(bytes: &[u8]) -> Result<PurchaseOrderAlternateIdList, ProtoConversionError> {
        let proto: purchase_order_state::PurchaseOrderAlternateIdList =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get PurchaseOrderAlternateIdList from bytes".to_string(),
                )
            })?;

        proto.into_native()
    }
}

impl IntoBytes for PurchaseOrderAlternateIdList {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from  PurchaseOrderAlternateIdList".to_string(),
            )
        })?;

        Ok(bytes)
    }
}

impl IntoProto<purchase_order_state::PurchaseOrderAlternateIdList>
    for PurchaseOrderAlternateIdList
{
}
impl IntoNative<PurchaseOrderAlternateIdList>
    for purchase_order_state::PurchaseOrderAlternateIdList
{
}

/// Builder used to create a list of `PurchaseOrderAlternateId`s
#[derive(Default, Clone)]
pub struct PurchaseOrderAlternateIdListBuilder {
    alternate_ids: Option<Vec<PurchaseOrderAlternateId>>,
}

impl PurchaseOrderAlternateIdListBuilder {
    pub fn new() -> Self {
        PurchaseOrderAlternateIdListBuilder::default()
    }

    pub fn with_alternate_ids(mut self, alternate_ids: Vec<PurchaseOrderAlternateId>) -> Self {
        self.alternate_ids = Some(alternate_ids);
        self
    }

    pub fn build(self) -> Result<PurchaseOrderAlternateIdList, PurchaseOrderAlternateIdBuildError> {
        let alternate_ids = self.alternate_ids.ok_or_else(|| {
            PurchaseOrderAlternateIdBuildError::MissingField("alternate_ids".to_string())
        })?;

        let alternate_ids = {
            if alternate_ids.is_empty() {
                return Err(PurchaseOrderAlternateIdBuildError::EmptyVec(
                    "alternate_ids".to_string(),
                ));
            } else {
                alternate_ids
            }
        };

        Ok(PurchaseOrderAlternateIdList { alternate_ids })
    }
}
