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

use grid_sdk::protocol::purchase_order::payload::{
    CreatePurchaseOrderPayload, CreateVersionPayload, PayloadRevision, PurchaseOrderPayload,
    UpdatePurchaseOrderPayload, UpdateVersionPayload,
};

fn _validate_po_payload(payload: &PurchaseOrderPayload) -> Result<(), ApplyError> {
    match payload.timestamp() {
        0 => Err(ApplyError::InvalidTransaction(
            "Payload's `timestamp` field is unset".to_string(),
        )),
        _ => Ok(()),
    }
}

// Validate a `CreatePurchaseOrderPayload` has all required fields defined
fn _validate_create_po_payload(payload: &CreatePurchaseOrderPayload) -> Result<(), ApplyError> {
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

    if let Some(create_version_payload) = payload.create_version_payload() {
        _validate_create_version_payload(&create_version_payload)?;
    }

    Ok(())
}

// Validate a `UpdatePurchaseOrderPayload` has all required fields defined
fn _validate_update_po_payload(payload: &UpdatePurchaseOrderPayload) -> Result<(), ApplyError> {
    if payload.uid().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`uid` is required to update a purchase order".to_string(),
        ));
    }

    if payload.workflow_status().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`workflow_status` is required to update a purchase order".to_string(),
        ));
    }

    if let Some(accepted_version) = payload.accepted_version_number() {
        if accepted_version.is_empty() {
            return Err(ApplyError::InvalidTransaction(
                "`workflow_status` is required to update a purchase order".to_string(),
            ));
        }
    }

    Ok(())
}

// Validate a `PayloadRevision` has all required fields defined
fn _validate_payload_revision(revision: &PayloadRevision) -> Result<(), ApplyError> {
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

// Validate a `CreateVersionPayload` has all required fields defined
fn _validate_create_version_payload(payload: &CreateVersionPayload) -> Result<(), ApplyError> {
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

    if payload.workflow_status().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`workflow_status` is required to create a purchase order version".to_string(),
        ));
    }

    _validate_payload_revision(payload.revision())?;

    Ok(())
}

// Validate a `UpdateVersionPayload` has all required fields defined
fn _validate_update_version_payload(payload: &UpdateVersionPayload) -> Result<(), ApplyError> {
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

    if payload.workflow_status().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "`workflow_status` is required to update a purchase order version".to_string(),
        ));
    }

    if payload.current_revision_id() != payload.revision().revision_id() {
        return Err(ApplyError::InvalidTransaction(
            "Updated version's `current_revision_id` and `revision` `revision_id` are not the same"
                .to_string(),
        ));
    }

    _validate_payload_revision(payload.revision())?;

    Ok(())
}
