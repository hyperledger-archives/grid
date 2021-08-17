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
        use sabre_sdk::TransactionContext;
        use sabre_sdk::TransactionHandler;
        use sabre_sdk::TpProcessRequest;
        use sabre_sdk::{WasmPtr, execute_entrypoint};
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
        use sawtooth_sdk::processor::handler::TransactionContext;
        use sawtooth_sdk::processor::handler::TransactionHandler;
        use sawtooth_sdk::messages::processor::TpProcessRequest;
    }
}

use grid_sdk::protos::FromBytes;
use grid_sdk::{
    pike::permissions::PermissionChecker,
    protocol::purchase_order::{
        payload::{
            Action, CreatePurchaseOrderPayload, CreateVersionPayload, PurchaseOrderPayload,
            UpdatePurchaseOrderPayload, UpdateVersionPayload,
        },
        state::{PurchaseOrderBuilder, PurchaseOrderRevisionBuilder, PurchaseOrderVersionBuilder},
    },
    purchase_order::addressing::GRID_PURCHASE_ORDER_NAMESPACE,
};

use crate::permissions::{permission_to_perm_string, Permission};
use crate::state::PurchaseOrderState;

#[cfg(target_arch = "wasm32")]
fn apply(
    request: &TpProcessRequest,
    context: &mut dyn TransactionContext,
) -> Result<bool, ApplyError> {
    let handler = PurchaseOrderTransactionHandler::new();
    match handler.apply(request, context) {
        Ok(_) => Ok(true),
        Err(err) => {
            info!("{} received {}", handler.family_name(), err);
            Err(err)
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub unsafe fn entrypoint(payload: WasmPtr, signer: WasmPtr, signature: WasmPtr) -> i32 {
    execute_entrypoint(payload, signer, signature, apply)
}

#[derive(Default)]
pub struct PurchaseOrderTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

impl PurchaseOrderTransactionHandler {
    pub fn new() -> Self {
        Self {
            family_name: "grid_purchase_order".to_string(),
            family_versions: vec!["1".to_string()],
            namespaces: vec![GRID_PURCHASE_ORDER_NAMESPACE.to_string()],
        }
    }
}

impl TransactionHandler for PurchaseOrderTransactionHandler {
    fn family_name(&self) -> String {
        self.family_name.clone()
    }

    fn family_versions(&self) -> Vec<String> {
        self.family_versions.clone()
    }

    fn namespaces(&self) -> Vec<String> {
        self.namespaces.clone()
    }

    fn apply(
        &self,
        request: &TpProcessRequest,
        context: &mut dyn TransactionContext,
    ) -> Result<(), ApplyError> {
        let payload = PurchaseOrderPayload::from_bytes(request.get_payload()).map_err(|err| {
            ApplyError::InvalidTransaction(format!("Cannot build po payload: {}", err))
        })?;
        let signer = request.get_header().get_signer_public_key();
        let mut state = PurchaseOrderState::new(context);
        let perm_checker = PermissionChecker::new(context);

        info!("Purchase Order Payload {:?}", payload.action());

        match payload.action() {
            Action::CreatePo(create_po_payload) => {
                create_purchase_order(create_po_payload, signer, &mut state, &perm_checker)
            }
            Action::UpdatePo(update_po_payload) => {
                update_purchase_order(update_po_payload, signer, &mut state, &perm_checker)
            }
            Action::CreateVersion(create_version_payload) => {
                create_version(create_version_payload, signer, &mut state, &perm_checker)
            }
            Action::UpdateVersion(update_version_payload) => {
                update_version(update_version_payload, signer, &mut state, &perm_checker)
            }
        }
    }
}

fn create_purchase_order(
    payload: &CreatePurchaseOrderPayload,
    signer: &str,
    state: &mut PurchaseOrderState,
    perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    let org_id = if payload.org_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Organization ID required".into(),
        ));
    } else {
        payload.org_id()
    };

    let po_uuid = if payload.uuid().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Purchase Order UUID required".into(),
        ));
    } else {
        payload.uuid()
    };

    // Check that the organization owning the purchase order exists
    state.get_organization(org_id)?.ok_or_else(|| {
        ApplyError::InvalidTransaction(format!("Organization {} does not exist", org_id))
    })?;
    // Validate the signer exists
    let agent = state.get_agent(signer)?.ok_or_else(|| {
        ApplyError::InvalidTransaction(format!("The signer is not an Agent: {}", signer))
    })?;

    if state.get_purchase_order(po_uuid)?.is_some() {
        return Err(ApplyError::InvalidTransaction(format!(
            "Purchase Order already exists: {}",
            po_uuid,
        )));
    }

    // Check signing agent's permission
    check_permission(
        perm_checker,
        signer,
        &permission_to_perm_string(Permission::CanCreatePo),
        agent.org_id(),
    )?;

    let versions = match payload.create_version_payload() {
        Some(payload) => {
            let payload_revision = payload.revision();
            let revision = PurchaseOrderRevisionBuilder::new()
                .with_revision_id(payload_revision.revision_id().to_string())
                .with_submitter(signer.to_string())
                .with_created_at(payload_revision.created_at())
                .with_order_xml_v3_4(payload_revision.order_xml_v3_4().to_string())
                .build()
                .map_err(|err| {
                    ApplyError::InvalidTransaction(format!(
                        "Cannot build purchase order revision: {}",
                        err
                    ))
                })?;
            let mut version_builder = PurchaseOrderVersionBuilder::new()
                .with_version_id(payload.version_id().to_string())
                .with_is_draft(payload.is_draft())
                .with_current_revision_id(revision.revision_id().to_string())
                .with_revisions(revision)
                .with_po_uuid(po_uuid.to_string());

            if payload.is_draft() {
                version_builder = version_builder.with_workflow_status("Editable".to_string());
            } else {
                version_builder = version_builder.with_workflow_status("Proposed".to_string());
            }

            vec![version_builder.build().map_err(|err| {
                ApplyError::InvalidTransaction(format!(
                    "Cannot build purchase order version: {}",
                    err
                ))
            })?]
        }
        None => vec![],
    };

    let purchase_order = PurchaseOrderBuilder::new()
        .with_org_id(org_id.to_string())
        .with_uuid(po_uuid.to_string())
        .with_versions(versions)
        .with_workflow_status("Issued".to_string())
        .with_created_at(payload.created_at())
        .with_is_closed(false)
        .build()
        .map_err(|err| {
            ApplyError::InvalidTransaction(format!("Cannot build purchase order: {}", err))
        })?;

    state.set_purchase_order(po_uuid, purchase_order)?;

    Ok(())
}

fn update_purchase_order(
    _payload: &UpdatePurchaseOrderPayload,
    _signer: &str,
    _state: &mut PurchaseOrderState,
    _perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    unimplemented!();
}

fn create_version(
    _payload: &CreateVersionPayload,
    _signer: &str,
    _state: &mut PurchaseOrderState,
    _perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    unimplemented!();
}

fn update_version(
    _payload: &UpdateVersionPayload,
    _signer: &str,
    _state: &mut PurchaseOrderState,
    _perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    unimplemented!();
}

fn check_permission(
    perm_checker: &PermissionChecker,
    signer: &str,
    permission: &str,
    po_owner: &str,
) -> Result<(), ApplyError> {
    match perm_checker.has_permission(signer, permission, po_owner) {
        Ok(true) => Ok(()),
        Ok(false) => Err(ApplyError::InvalidTransaction(format!(
            "The signer \"{}\" does not have the \"{}\" permission for org \"{}\"",
            signer, permission, po_owner
        ))),
        Err(e) => Err(ApplyError::InvalidTransaction(format!(
            "Permission check failed: {}",
            e
        ))),
    }
}
