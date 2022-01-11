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

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::ApplyError;
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
    }
}

use crate::workflow::POWorkflow;
use grid_sdk::protocol::purchase_order::payload::{
    Action, CreatePurchaseOrderPayload, CreateVersionPayload, PayloadRevision,
    PurchaseOrderPayload, UpdatePurchaseOrderPayload, UpdateVersionPayload,
};

pub(crate) fn validate_po_payload(payload: &PurchaseOrderPayload) -> Result<(), ApplyError> {
    match payload.action() {
        Action::CreatePo(payload) => validate_create_po_payload(payload)?,
        Action::UpdatePo(payload) => validate_update_po_payload(payload)?,
        Action::CreateVersion(payload) => validate_create_version_payload(payload)?,
        Action::UpdateVersion(payload) => validate_update_version_payload(payload)?,
    };

    match payload.timestamp() {
        0 => Err(ApplyError::InvalidTransaction(
            "Payload's `timestamp` field is unset".to_string(),
        )),
        _ => Ok(()),
    }
}

// Validate a `CreatePurchaseOrderPayload` has all required fields defined, the values of which are
// also validated.
fn validate_create_po_payload(payload: &CreatePurchaseOrderPayload) -> Result<(), ApplyError> {
    if payload.uid().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`uid` is required to create a purchase order".to_string(),
        ));
    }

    if payload.buyer_org_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`buyer_org_id` is required to create a purchase order".to_string(),
        ));
    }

    if payload.seller_org_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`seller_org_id` is required to create a purchase order".to_string(),
        ));
    }

    if payload.created_at() == 0 {
        return Err(ApplyError::InvalidTransaction(
            "`created_at` is required to create a purchase order".to_string(),
        ));
    }

    if payload.workflow_state().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`workflow_state` is required to create a purchase order".to_string(),
        ));
    }

    if payload.workflow_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`workflow_id` is required to create a purchase order".to_string(),
        ));
    }

    validate_workflow_id(payload.workflow_id().to_string())?;

    if let Some(create_version_payload) = payload.create_version_payload() {
        if create_version_payload.po_uid() != payload.uid() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Version {} must refer to purchase order {} within the payload, \
                refers to purchase order {}",
                create_version_payload.version_id(),
                payload.uid(),
                create_version_payload.po_uid(),
            )));
        }

        validate_create_version_payload(&create_version_payload)?;
    }

    Ok(())
}

fn validate_workflow_id(workflow: String) -> Result<String, ApplyError> {
    let workflow_ids = get_workflow_ids();
    if workflow_ids.iter().any(|w| w == &workflow) {
        Ok(workflow)
    } else {
        Err(ApplyError::InvalidTransaction(format!(
            "No workflow exists with id {}",
            &workflow
        )))
    }
}

fn get_workflow_ids() -> Vec<String> {
    // In the future, this should get workflow names from state
    vec![
        POWorkflow::SystemOfRecord.to_string(),
        POWorkflow::Collaborative.to_string(),
    ]
}

// Validate a `CreateVersionPayload` has all required fields defined
fn validate_create_version_payload(payload: &CreateVersionPayload) -> Result<(), ApplyError> {
    if payload.version_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`version_id` is required to create a purchase order version".to_string(),
        ));
    }

    if payload.po_uid().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`po_uid` is required to create a purchase order version".to_string(),
        ));
    }

    if payload.workflow_state().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`workflow_state` is required to create a purchase order version".to_string(),
        ));
    }

    if payload.revision().revision_id() != 1 {
        return Err(ApplyError::InvalidTransaction(format!(
            "Revision IDs begin incrementing at `1`, attempting to create a new version \
            with revision ID `{}`",
            payload.revision().revision_id()
        )));
    }

    validate_payload_revision(payload.revision())?;

    Ok(())
}

// Validate a `UpdatePurchaseOrderPayload` has all required fields defined
fn validate_update_po_payload(payload: &UpdatePurchaseOrderPayload) -> Result<(), ApplyError> {
    if payload.uid().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`uid` is required to update a purchase order".to_string(),
        ));
    }

    if payload.workflow_state().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`workflow_state` is required to update a purchase order".to_string(),
        ));
    }

    if let Some(accepted_version) = payload.accepted_version_number() {
        if accepted_version.is_empty() {
            return Err(ApplyError::InvalidTransaction(
                "`accepted_version_number` is required to update a purchase order".to_string(),
            ));
        }
    }

    Ok(())
}

// Validate a `UpdateVersionPayload` has all required fields defined
fn validate_update_version_payload(payload: &UpdateVersionPayload) -> Result<(), ApplyError> {
    if payload.version_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`version_id` is required to update a purchase order version".to_string(),
        ));
    }

    if payload.po_uid().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`po_uid` is required to update a purchase order version".to_string(),
        ));
    }

    if payload.workflow_state().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`workflow_state` is required to update a purchase order version".to_string(),
        ));
    }

    validate_payload_revision(payload.revision())?;

    Ok(())
}

// Validate a `PayloadRevision` has all required fields defined
fn validate_payload_revision(revision: &PayloadRevision) -> Result<(), ApplyError> {
    if revision.submitter().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`submitter` is required for a po revision".to_string(),
        ));
    }

    if revision.order_xml_v3_4().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`order_xml_v3_4` is required for a po revision".to_string(),
        ));
    }

    if revision.revision_id() == 0 {
        return Err(ApplyError::InvalidTransaction(
            "`revision_id` must be greater than 0".to_string(),
        ));
    }

    if revision.created_at() == 0 {
        return Err(ApplyError::InvalidTransaction(
            "Invalid `created_at` value, must not be 0".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use grid_sdk::protos::{self, IntoNative};

    use crate::workflow::POWorkflow;

    const SUBMITTER: &str = "submitter";
    const XML_TEST_STRING: &str = "XML_DATA";

    #[test]
    /// Validates that a `PayloadRevision` is only valid when all fields are accurately defined.
    /// The test follows these steps:
    ///
    /// 1. Create a `PayloadRevision` protobuf message and fill in all fields with valid data
    /// 2. Assert this `PayloadRevision` successfully validates
    ///
    /// This test validates that a fully-defined `PayloadRevision` is able to be validated.
    fn test_validate_payload_revision_valid() {
        let mut payload_revision_proto = protos::purchase_order_payload::PayloadRevision::new();
        payload_revision_proto.set_revision_id(1);
        payload_revision_proto.set_submitter(SUBMITTER.to_string());
        payload_revision_proto.set_created_at(1);
        payload_revision_proto.set_order_xml_v3_4(XML_TEST_STRING.to_string());
        let revision_native = payload_revision_proto
            .clone()
            .into_native()
            .expect("Unable to create protocol PayloadRevision");
        // Validate the payload revision
        assert!(validate_payload_revision(&revision_native).is_ok());
    }

    #[test]
    /// Validates that a `PayloadRevision` with an undefined `submitter` is not validated.
    /// The test follows these steps:
    ///
    /// 1. Create a `PayloadRevision` protobuf message and define all fields except for `submitter`.
    /// 2. Assert this `PayloadRevision` does not successfully validate
    ///
    /// This test validates that a `PayloadRevision` without the `submitter` field defined does
    /// not validate successfully.
    fn test_validate_payload_revision_invalid_submitter() {
        let mut payload_revision_proto = protos::purchase_order_payload::PayloadRevision::new();
        payload_revision_proto.set_revision_id(1);
        payload_revision_proto.set_created_at(1);
        payload_revision_proto.set_order_xml_v3_4(XML_TEST_STRING.to_string());
        let revision_native = payload_revision_proto
            .clone()
            .into_native()
            .expect("Unable to create protocol PayloadRevision");
        // Validate the payload revision will produce an error
        assert!(validate_payload_revision(&revision_native).is_err());
    }

    #[test]
    /// Validates that a `PayloadRevision` with an undefined `revision_id` is not validated.
    /// The test follows these steps:
    ///
    /// 1. Create a `PayloadRevision` protobuf message and fill in all fields except `revision_id`.
    /// 2. Assert this `PayloadRevision` does not successfully validate
    ///
    /// This test validates that a `PayloadRevision` without the `revision_id` field defined does
    /// not validate successfully.
    fn test_validate_payload_revision_invalid_revision_id() {
        let mut payload_revision_proto = protos::purchase_order_payload::PayloadRevision::new();
        payload_revision_proto.set_submitter(SUBMITTER.to_string());
        payload_revision_proto.set_created_at(1);
        payload_revision_proto.set_order_xml_v3_4(XML_TEST_STRING.to_string());
        let revision_native = payload_revision_proto
            .clone()
            .into_native()
            .expect("Unable to create protocol PayloadRevision");
        // Validate the payload revision will produce an error
        assert!(validate_payload_revision(&revision_native).is_err());
    }

    #[test]
    /// Validates that a `PayloadRevision` with an undefined `created_at` is not validated.
    /// The test follows these steps:
    ///
    /// 1. Create a `PayloadRevision` protobuf message and fill in all fields except `created_at`.
    /// 2. Assert this `PayloadRevision` does not successfully validate
    ///
    /// This test validates that a `PayloadRevision` without the `created_at` field defined does
    /// not validate successfully.
    fn test_validate_payload_revision_invalid_created_at() {
        let mut payload_revision_proto = protos::purchase_order_payload::PayloadRevision::new();
        payload_revision_proto.set_revision_id(1);
        payload_revision_proto.set_submitter(SUBMITTER.to_string());
        payload_revision_proto.set_order_xml_v3_4(XML_TEST_STRING.to_string());
        let revision_native = payload_revision_proto
            .clone()
            .into_native()
            .expect("Unable to create protocol PayloadRevision");
        // Validate the payload revision will produce an error
        assert!(validate_payload_revision(&revision_native).is_err());
    }

    #[test]
    /// Validates that a `PayloadRevision` with an undefined `order_xml_v3_4` is not validated.
    /// The test follows these steps:
    ///
    /// 1. Create a `PayloadRevision` protobuf message and fill in all fields except
    ///    `order_xml_v3_4`.
    /// 2. Assert this `PayloadRevision` does not successfully validate
    ///
    /// This test validates that a `PayloadRevision` without the `order_xml_v3_4` field defined
    /// does not validate successfully.
    fn test_validate_payload_revision_invalid_order_xml_v3_4() {
        let mut payload_revision_proto = protos::purchase_order_payload::PayloadRevision::new();
        payload_revision_proto.set_revision_id(1);
        payload_revision_proto.set_submitter(SUBMITTER.to_string());
        payload_revision_proto.set_created_at(1);
        let revision_native = payload_revision_proto
            .clone()
            .into_native()
            .expect("Unable to create protocol PayloadRevision");
        // Validate the payload revision will produce an error
        assert!(validate_payload_revision(&revision_native).is_err());
    }

    #[test]
    /// Validates that an `UpdateVersionPayload` with all fields defined is successfully validated.
    /// The test follows these steps:
    ///
    /// 1. Create an `UpdateVersionPayload` protobuf message and define all fields
    /// 2. Assert this `UpdateVersionPayload` successfully validates
    ///
    /// This test validates that a `UpdateVersionPayload` with all fields correctly defined is able
    /// to be validated.
    fn test_validate_update_version_payload_valid() {
        let mut payload_revision_proto = protos::purchase_order_payload::PayloadRevision::new();
        payload_revision_proto.set_revision_id(2);
        payload_revision_proto.set_submitter(SUBMITTER.to_string());
        payload_revision_proto.set_created_at(1);
        payload_revision_proto.set_order_xml_v3_4(XML_TEST_STRING.to_string());

        let mut update_version_payload =
            protos::purchase_order_payload::UpdateVersionPayload::new();
        update_version_payload.set_version_id("01".to_string());
        update_version_payload.set_po_uid("PO-01".to_string());
        update_version_payload.set_workflow_state("proposed".to_string());
        update_version_payload.set_revision(payload_revision_proto);
        let version_native = update_version_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol UpdateVersionPayload");
        // Validate the update version payload is successful
        assert!(validate_update_version_payload(&version_native).is_ok());
    }

    #[test]
    /// Validates that an `UpdateVersionPayload` with an undefined `version_id` is not able to be
    /// validated. The test follows these steps:
    ///
    /// 1. Create an `UpdateVersionPayload` protobuf message and define all fields except the
    ///    `version_id` field
    /// 2. Assert this `UpdateVersionPayload` does not successfully validate
    ///
    /// This test validates that a `UpdateVersionPayload` with an undefined `version_id` field
    /// produces an error on validation.
    fn test_validate_update_version_payload_invalid_version_id() {
        let mut payload_revision_proto = protos::purchase_order_payload::PayloadRevision::new();
        payload_revision_proto.set_revision_id(2);
        payload_revision_proto.set_submitter(SUBMITTER.to_string());
        payload_revision_proto.set_created_at(1);
        payload_revision_proto.set_order_xml_v3_4(XML_TEST_STRING.to_string());

        let mut update_version_payload =
            protos::purchase_order_payload::UpdateVersionPayload::new();
        update_version_payload.set_po_uid("PO-01".to_string());
        update_version_payload.set_workflow_state("proposed".to_string());
        update_version_payload.set_revision(payload_revision_proto);
        let version_native = update_version_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol UpdateVersionPayload");
        // Validate the update version payload is not successful
        assert!(validate_update_version_payload(&version_native).is_err());
    }

    #[test]
    /// Validates that an `UpdateVersionPayload` with an undefined `po_uid` is not able to be
    /// validated. The test follows these steps:
    ///
    /// 1. Create an `UpdateVersionPayload` protobuf message and define all fields except the
    ///    `po_uid` field
    /// 2. Assert this `UpdateVersionPayload` does not successfully validate
    ///
    /// This test validates that a `UpdateVersionPayload` with an undefined `po_uid` field
    /// produces an error on validation.
    fn test_validate_update_version_payload_invalid_po_uid() {
        let mut payload_revision_proto = protos::purchase_order_payload::PayloadRevision::new();
        payload_revision_proto.set_revision_id(2);
        payload_revision_proto.set_submitter(SUBMITTER.to_string());
        payload_revision_proto.set_created_at(1);
        payload_revision_proto.set_order_xml_v3_4(XML_TEST_STRING.to_string());

        let mut update_version_payload =
            protos::purchase_order_payload::UpdateVersionPayload::new();
        update_version_payload.set_version_id("01".to_string());
        update_version_payload.set_workflow_state("proposed".to_string());
        update_version_payload.set_revision(payload_revision_proto);
        let version_native = update_version_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol UpdateVersionPayload");
        // Validate the update version payload is not successful
        assert!(validate_update_version_payload(&version_native).is_err());
    }

    #[test]
    /// Validates that an `UpdateVersionPayload` with an undefined `workflow_state` is not able
    /// to be validated. The test follows these steps:
    ///
    /// 1. Create an `UpdateVersionPayload` protobuf message and define all fields except the
    ///    `workflow_state` field
    /// 2. Assert this `UpdateVersionPayload` does not successfully validate
    ///
    /// This test validates that a `UpdateVersionPayload` with an undefined `workflow_state`
    /// field produces an error on validation.
    fn test_validate_update_version_payload_invalid_workflow_state() {
        let mut payload_revision_proto = protos::purchase_order_payload::PayloadRevision::new();
        payload_revision_proto.set_revision_id(2);
        payload_revision_proto.set_submitter(SUBMITTER.to_string());
        payload_revision_proto.set_created_at(1);
        payload_revision_proto.set_order_xml_v3_4(XML_TEST_STRING.to_string());

        let mut update_version_payload =
            protos::purchase_order_payload::UpdateVersionPayload::new();
        update_version_payload.set_version_id("01".to_string());
        update_version_payload.set_po_uid("PO-01".to_string());
        update_version_payload.set_revision(payload_revision_proto);
        let version_native = update_version_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol UpdateVersionPayload");
        // Validate the update version payload is not successful
        assert!(validate_update_version_payload(&version_native).is_err());
    }

    #[test]
    /// Validates that an `UpdateVersionPayload` with an undefined `revision` is not
    /// able to be validated. The test follows these steps:
    ///
    /// 1. Create an `UpdateVersionPayload` protobuf message and define all fields except the
    ///    `revision` field
    /// 2. Assert this `UpdateVersionPayload` does not successfully validate
    ///
    /// This test validates that a `UpdateVersionPayload` with an undefined `revision`
    /// field produces an error on validation.
    fn test_validate_update_version_payload_invalid_revision() {
        let mut update_version_payload =
            protos::purchase_order_payload::UpdateVersionPayload::new();
        update_version_payload.set_version_id("01".to_string());
        update_version_payload.set_po_uid("PO-01".to_string());
        update_version_payload.set_workflow_state("proposed".to_string());
        let version_native = update_version_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol UpdateVersionPayload");
        // Validate the update version payload is not successful
        assert!(validate_update_version_payload(&version_native).is_err());
    }

    #[test]
    /// Validates that an `CreateVersionPayload` with all fields defined is able to be validated.
    /// The test follows these steps:
    ///
    /// 1. Create a `CreateVersionPayload` protobuf message with all valid fields
    /// 2. Assert this `CreateVersionPayload` successfully validates
    ///
    /// This test validates that a `CreateVersionPayload` with all fields defined is able to
    /// successfully validate.
    fn test_validate_create_version_payload_valid() {
        let mut payload_revision_proto = protos::purchase_order_payload::PayloadRevision::new();
        payload_revision_proto.set_revision_id(1);
        payload_revision_proto.set_submitter(SUBMITTER.to_string());
        payload_revision_proto.set_created_at(1);
        payload_revision_proto.set_order_xml_v3_4(XML_TEST_STRING.to_string());

        let mut create_version_payload =
            protos::purchase_order_payload::CreateVersionPayload::new();
        create_version_payload.set_version_id("01".to_string());
        create_version_payload.set_po_uid("PO-01".to_string());
        create_version_payload.set_workflow_state("proposed".to_string());
        create_version_payload.set_revision(payload_revision_proto);
        let payload_native = create_version_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol CreateVersionPayload");
        // Validate the create version payload is successful
        assert!(validate_create_version_payload(&payload_native).is_ok());
    }

    #[test]
    /// Validates that a `CreateVersionPayload` with an undefined `version_id` is not
    /// able to be validated. The test follows these steps:
    ///
    /// 1. Create a `CreateVersionPayload` protobuf message and defined all fields except the
    ///    `version_id` field
    /// 2. Assert this `CreateVersionPayload` does not successfully validate
    ///
    /// This test validates that a `CreateVersionPayload` with an undefined `version_id`
    /// field produces an error on validation.
    fn test_validate_create_version_payload_invalid_version_id() {
        let mut payload_revision_proto = protos::purchase_order_payload::PayloadRevision::new();
        payload_revision_proto.set_revision_id(1);
        payload_revision_proto.set_submitter(SUBMITTER.to_string());
        payload_revision_proto.set_created_at(1);
        payload_revision_proto.set_order_xml_v3_4(XML_TEST_STRING.to_string());

        let mut create_version_payload =
            protos::purchase_order_payload::CreateVersionPayload::new();
        create_version_payload.set_po_uid("PO-01".to_string());
        create_version_payload.set_workflow_state("proposed".to_string());
        create_version_payload.set_revision(payload_revision_proto);
        let payload_native = create_version_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol CreateVersionPayload");
        // Validate the create version payload is successful
        assert!(validate_create_version_payload(&payload_native).is_err());
    }

    #[test]
    /// Validates that a `CreateVersionPayload` with an undefined `po_uid` is not
    /// able to be validated. The test follows these steps:
    ///
    /// 1. Create a `CreateVersionPayload` protobuf message and define all fields except the
    ///    `po_uid` field
    /// 2. Assert this `CreateVersionPayload` does not successfully validate
    ///
    /// This test validates that a `CreateVersionPayload` with an undefined `po_uid`
    /// field produces an error on validation.
    fn test_validate_create_version_payload_invalid_po_uid() {
        let mut payload_revision_proto = protos::purchase_order_payload::PayloadRevision::new();
        payload_revision_proto.set_revision_id(1);
        payload_revision_proto.set_submitter(SUBMITTER.to_string());
        payload_revision_proto.set_created_at(1);
        payload_revision_proto.set_order_xml_v3_4(XML_TEST_STRING.to_string());

        let mut create_version_payload =
            protos::purchase_order_payload::CreateVersionPayload::new();
        create_version_payload.set_version_id("01".to_string());
        create_version_payload.set_workflow_state("proposed".to_string());
        create_version_payload.set_revision(payload_revision_proto);
        let payload_native = create_version_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol CreateVersionPayload");
        // Validate the create version payload is successful
        assert!(validate_create_version_payload(&payload_native).is_err());
    }

    #[test]
    /// Validates that a `CreateVersionPayload` with an undefined `workflow_state` is not
    /// able to be validated. The test follows these steps:
    ///
    /// 1. Create a `CreateVersionPayload` protobuf message and define all fields except the
    ///    `workflow_state` field
    /// 2. Assert this `CreateVersionPayload` does not successfully validate
    ///
    /// This test validates that a `CreateVersionPayload` with an undefined `workflow_state`
    /// field produces an error on validation.
    fn test_validate_create_version_payload_invalid_workflow_state() {
        let mut payload_revision_proto = protos::purchase_order_payload::PayloadRevision::new();
        payload_revision_proto.set_revision_id(1);
        payload_revision_proto.set_submitter(SUBMITTER.to_string());
        payload_revision_proto.set_created_at(1);
        payload_revision_proto.set_order_xml_v3_4(XML_TEST_STRING.to_string());

        let mut create_version_payload =
            protos::purchase_order_payload::CreateVersionPayload::new();
        create_version_payload.set_version_id("01".to_string());
        create_version_payload.set_po_uid("PO-01".to_string());
        create_version_payload.set_revision(payload_revision_proto);
        let payload_native = create_version_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol CreateVersionPayload");
        // Validate the create version payload is successful
        assert!(validate_create_version_payload(&payload_native).is_err());
    }

    #[test]
    /// Validates that a `CreateVersionPayload` with an undefined `revision` is not
    /// able to be validated. The test follows these steps:
    ///
    /// 1. Create a `CreateVersionPayload` protobuf message and define all fields except the
    ///    `revision` field
    /// 2. Assert this `CreateVersionPayload` does not successfully validate
    ///
    /// This test validates that a `CreateVersionPayload` with an undefined `revision`
    /// field produces an error on validation.
    fn test_validate_create_version_payload_invalid_revision() {
        let mut create_version_payload =
            protos::purchase_order_payload::CreateVersionPayload::new();
        create_version_payload.set_version_id("01".to_string());
        create_version_payload.set_po_uid("PO-01".to_string());
        create_version_payload.set_workflow_state("proposed".to_string());
        let payload_native = create_version_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol CreateVersionPayload");
        // Validate the create version payload is successful
        assert!(validate_create_version_payload(&payload_native).is_err());
    }

    #[test]
    /// Validates that an `UpdatePurchaseOrderPayload` with all fields defined is successfully
    /// validated. The test follows these steps:
    ///
    /// 1. Create an `UpdatePurchaseOrderPayload` protobuf message and define all fields
    /// 2. Assert this `UpdatePurchaseOrderPayload` successfully validates
    ///
    /// This test validates that a `UpdatePurchaseOrderPayload` with all fields correctly defined
    /// is able to be validated.
    fn test_validate_update_po_payload_valid() {
        let mut update_po_payload =
            protos::purchase_order_payload::UpdatePurchaseOrderPayload::new();
        update_po_payload.set_po_uid("PO-01".to_string());
        update_po_payload.set_workflow_state("issued".to_string());
        let payload_native = update_po_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol UpdatePurchaseOrderPayload");
        // Validate the update po payload is successful
        assert!(validate_update_po_payload(&payload_native).is_ok());
    }

    #[test]
    /// Validates that an `UpdatePurchaseOrderPayload` with an undefined `po_uid` field is unable
    /// to be validated. The test follows these steps:
    ///
    /// 1. Create an `UpdatePurchaseOrderPayload` protobuf message and define all fields except
    ///    the `po_uid` field
    /// 2. Assert this `UpdatePurchaseOrderPayload` does not validate
    ///
    /// This test validates that a `UpdatePurchaseOrderPayload` with an undefined `po_uid` field is
    /// unable to be validated
    fn test_validate_update_po_payload_invalid_po_uid() {
        let mut update_po_payload =
            protos::purchase_order_payload::UpdatePurchaseOrderPayload::new();
        update_po_payload.set_workflow_state("issued".to_string());
        let payload_native = update_po_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol UpdatePurchaseOrderPayload");
        // Validate the update po payload is not successful
        assert!(validate_update_po_payload(&payload_native).is_err());
    }

    #[test]
    /// Validates that an `UpdatePurchaseOrderPayload` with an undefined `workflow_state` field
    /// is unable to be validated. The test follows these steps:
    ///
    /// 1. Create an `UpdatePurchaseOrderPayload` protobuf message and define all fields except
    ///    the `workflow_state` field
    /// 2. Assert this `UpdatePurchaseOrderPayload` does not validate
    ///
    /// This test validates that a `UpdatePurchaseOrderPayload` with an undefined `workflow_state`
    /// field is unable to be validated
    fn test_validate_update_po_payload_invalid_workflow_state() {
        let mut update_po_payload =
            protos::purchase_order_payload::UpdatePurchaseOrderPayload::new();
        update_po_payload.set_po_uid("PO-01".to_string());
        let payload_native = update_po_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol UpdatePurchaseOrderPayload");
        // Validate the update po payload is not successful
        assert!(validate_update_po_payload(&payload_native).is_err());
    }

    #[test]
    /// Validates that a `CreatePurchaseOrderPayload` with all fields defined is successfully
    /// validated. The test follows these steps:
    ///
    /// 1. Create an `CreatePurchaseOrderPayload` protobuf message and define the necessary fields
    /// 2. Assert this `CreatePurchaseOrderPayload` successfully validates
    ///
    /// This test validates that a `CreatePurchaseOrderPayload` with all fields correctly defined
    /// is able to be validated.
    fn test_validate_create_po_payload_valid() {
        let mut create_po_payload =
            protos::purchase_order_payload::CreatePurchaseOrderPayload::new();
        create_po_payload.set_uid("PO-01".to_string());
        create_po_payload.set_created_at(1);
        create_po_payload.set_buyer_org_id("buyer".to_string());
        create_po_payload.set_seller_org_id("seller".to_string());
        create_po_payload.set_workflow_state("issued".to_string());
        create_po_payload.set_workflow_id(POWorkflow::SystemOfRecord.to_string());
        let payload_native = create_po_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol CreatePurchaseOrderPayload");
        // Validate the create po payload is successful
        assert!(validate_create_po_payload(&payload_native).is_ok());
    }

    #[test]
    /// Validates that a `CreatePurchaseOrderPayload` with an undefind `uid` field is not
    /// validated. The test follows these steps:
    ///
    /// 1. Create an `CreatePurchaseOrderPayload` protobuf message, leaving the `uid` field
    ///    undefined and filling in the remaining values
    /// 2. Assert this `CreatePurchaseOrderPayload` does not successfully validate
    ///
    /// This test validates that a `CreatePurchaseOrderPayload` with an undefined `uid` field is
    /// not succesfully validated.
    fn test_validate_create_po_payload_invalid_uid() {
        let mut create_po_payload =
            protos::purchase_order_payload::CreatePurchaseOrderPayload::new();
        create_po_payload.set_created_at(1);
        create_po_payload.set_buyer_org_id("buyer".to_string());
        create_po_payload.set_seller_org_id("seller".to_string());
        create_po_payload.set_workflow_state("issued".to_string());
        create_po_payload.set_workflow_id(POWorkflow::SystemOfRecord.to_string());
        let payload_native = create_po_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol CreatePurchaseOrderPayload");
        // Validate the create po payload is not successful
        assert!(validate_create_po_payload(&payload_native).is_err());
    }

    #[test]
    /// Validates that a `CreatePurchaseOrderPayload` with an undefind `created_at` field is
    /// not validated. The test follows these steps:
    ///
    /// 1. Create an `CreatePurchaseOrderPayload` protobuf message, leaving the `created_at`
    ///    field undefined and filling in the remaining values
    /// 2. Assert this `CreatePurchaseOrderPayload` does not successfully validate
    ///
    /// This test validates that a `CreatePurchaseOrderPayload` with an undefined `created_at`
    /// field is not succesfully validated.
    fn test_validate_create_po_payload_invalid_created_at() {
        let mut create_po_payload =
            protos::purchase_order_payload::CreatePurchaseOrderPayload::new();
        create_po_payload.set_uid("PO-01".to_string());
        create_po_payload.set_buyer_org_id("buyer".to_string());
        create_po_payload.set_seller_org_id("seller".to_string());
        create_po_payload.set_workflow_state("issued".to_string());
        create_po_payload.set_workflow_id(POWorkflow::SystemOfRecord.to_string());
        let payload_native = create_po_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol CreatePurchaseOrderPayload");
        // Validate the create po payload is not successful
        assert!(validate_create_po_payload(&payload_native).is_err());
    }

    #[test]
    /// Validates that a `CreatePurchaseOrderPayload` with an undefind `buyer_org_id` field is
    /// not validated. The test follows these steps:
    ///
    /// 1. Create an `CreatePurchaseOrderPayload` protobuf message, leaving the `buyer_org_id`
    ///    field undefined and filling in the remaining values
    /// 2. Assert this `CreatePurchaseOrderPayload` does not successfully validate
    ///
    /// This test validates that a `CreatePurchaseOrderPayload` with an undefined `buyer_org_id`
    /// field is not succesfully validated.
    fn test_validate_create_po_payload_invalid_buyer_org() {
        let mut create_po_payload =
            protos::purchase_order_payload::CreatePurchaseOrderPayload::new();
        create_po_payload.set_uid("PO-01".to_string());
        create_po_payload.set_created_at(1);
        create_po_payload.set_seller_org_id("seller".to_string());
        create_po_payload.set_workflow_state("issued".to_string());
        create_po_payload.set_workflow_id(POWorkflow::SystemOfRecord.to_string());
        let payload_native = create_po_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol CreatePurchaseOrderPayload");
        // Validate the create po payload is not successful
        assert!(validate_create_po_payload(&payload_native).is_err());
    }

    #[test]
    /// Validates that a `CreatePurchaseOrderPayload` with an undefind `seller_org_id` field is
    /// not validated. The test follows these steps:
    ///
    /// 1. Create an `CreatePurchaseOrderPayload` protobuf message, leaving the `seller_org_id`
    ///    field undefined and filling in the remaining values
    /// 2. Assert this `CreatePurchaseOrderPayload` does not successfully validate
    ///
    /// This test validates that a `CreatePurchaseOrderPayload` with an undefined `seller_org_id`
    /// field is not succesfully validated.
    fn test_validate_create_po_payload_invalid_seller_org() {
        let mut create_po_payload =
            protos::purchase_order_payload::CreatePurchaseOrderPayload::new();
        create_po_payload.set_uid("PO-01".to_string());
        create_po_payload.set_created_at(1);
        create_po_payload.set_buyer_org_id("buyer".to_string());
        create_po_payload.set_workflow_state("issued".to_string());
        create_po_payload.set_workflow_id(POWorkflow::SystemOfRecord.to_string());
        let payload_native = create_po_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol CreatePurchaseOrderPayload");
        // Validate the create po payload is not successful
        assert!(validate_create_po_payload(&payload_native).is_err());
    }

    #[test]
    /// Validates that a `CreatePurchaseOrderPayload` with an undefind `workflow_state` field is
    /// not validated. The test follows these steps:
    ///
    /// 1. Create an `CreatePurchaseOrderPayload` protobuf message, leaving the `workflow_state`
    ///    field undefined and filling in the remaining values
    /// 2. Assert this `CreatePurchaseOrderPayload` does not successfully validate
    ///
    /// This test validates that a `CreatePurchaseOrderPayload` with an undefined `workflow_state`
    /// field is not succesfully validated.
    fn test_validate_create_po_payload_invalid_workflow_state() {
        let mut create_po_payload =
            protos::purchase_order_payload::CreatePurchaseOrderPayload::new();
        create_po_payload.set_uid("PO-01".to_string());
        create_po_payload.set_created_at(1);
        create_po_payload.set_buyer_org_id("buyer".to_string());
        create_po_payload.set_seller_org_id("seller".to_string());
        create_po_payload.set_workflow_id(POWorkflow::SystemOfRecord.to_string());
        let payload_native = create_po_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol CreatePurchaseOrderPayload");
        // Validate the create po payload is not successful
        assert!(validate_create_po_payload(&payload_native).is_err());
    }

    #[test]
    /// Validates that a `CreatePurchaseOrderPayload` with an undefind `workflow_id` field is not
    /// validated. The test follows these steps:
    ///
    /// 1. Create an `CreatePurchaseOrderPayload` protobuf message, leaving the `workflow_id` field
    ///    undefined and filling in the remaining values
    /// 2. Assert this `CreatePurchaseOrderPayload` does not successfully validate
    ///
    /// This test validates that a `CreatePurchaseOrderPayload` with an undefined `workflow_id`
    /// field is not succesfully validated.
    fn test_validate_create_po_payload_invalid_workflow_id() {
        let mut create_po_payload =
            protos::purchase_order_payload::CreatePurchaseOrderPayload::new();
        create_po_payload.set_created_at(1);
        create_po_payload.set_buyer_org_id("buyer".to_string());
        create_po_payload.set_seller_org_id("seller".to_string());
        create_po_payload.set_workflow_state("issued".to_string());
        let payload_native = create_po_payload
            .clone()
            .into_native()
            .expect("Unable to create protocol CreatePurchaseOrderPayload");
        // Validate the create po payload is not successful
        assert!(validate_create_po_payload(&payload_native).is_err());
    }
}
