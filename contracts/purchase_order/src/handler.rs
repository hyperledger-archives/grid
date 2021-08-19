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

use crate::state::PurchaseOrderState;
use grid_sdk::protos::FromBytes;
use grid_sdk::{
    pike::permissions::PermissionChecker,
    protocol::purchase_order::payload::{
        Action, CreatePurchaseOrderPayload, CreateVersionPayload, PurchaseOrderPayload,
        UpdatePurchaseOrderPayload, UpdateVersionPayload,
    },
    purchase_order::addressing::GRID_PURCHASE_ORDER_NAMESPACE,
    workflow::WorkflowState,
};

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
    _payload: &CreatePurchaseOrderPayload,
    _signer: &str,
    _state: &mut PurchaseOrderState,
    _perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    unimplemented!();
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

#[allow(dead_code)]
fn check_permission_with_workflow(
    perm_checker: &PermissionChecker,
    permission: &str,
    signer: &str,
    record_owner: &str,
    workflow_state: WorkflowState,
    desired_state: &str,
) -> Result<(), ApplyError> {
    let agent = perm_checker.get_agent(signer).map_err(|err| {
        ApplyError::InternalError(format!(
            "Could not fetch agent to check permissions: {}",
            err
        ))
    })?;

    if agent.is_none() {
        return Err(ApplyError::InternalError(format!(
            "Could not fetch agent with public key {}",
            signer
        )));
    }

    let agent = agent.unwrap();

    let mut agent_perms = Vec::new();
    agent.roles().iter().for_each(|r| {
        let mut org_id = Some(record_owner);
        if r.contains('.') {
            org_id = None;
        }

        let role = perm_checker.get_role(r, org_id).ok().flatten();

        if let Some(role) = role {
            agent_perms.extend_from_slice(role.permissions());
        }
    });

    let permissions = workflow_state.expand_permissions(agent_perms.as_slice());

    if !permissions.contains(&permission.to_string()) {
        return Err(ApplyError::InvalidTransaction(format!(
            "Signer {} does not have permission {}",
            signer, permission
        )));
    }

    let can_transition = workflow_state.can_transition(desired_state.to_string(), agent_perms);

    if !can_transition {
        return Err(ApplyError::InvalidTransaction(format!(
            "Signer {} does not have permission to transition to state {}",
            signer, desired_state
        )));
    }

    let aliases = workflow_state.get_aliases_by_permission(permission);

    let mut has_permission = false;
    let mut has_permission_err = None;

    for alias in aliases {
        match perm_checker.has_permission(signer, &alias, record_owner) {
            Ok(true) => {
                has_permission = true;
            }
            Ok(false) => {}
            Err(err) => {
                has_permission_err = Some(ApplyError::InvalidTransaction(format!(
                    "Permission check failed: {}",
                    err
                )));
            }
        }
    }

    if let Some(has_permission_err) = has_permission_err {
        return Err(has_permission_err);
    }

    if !has_permission {
        return Err(ApplyError::InvalidTransaction(format!(
            "Signer {} does not have permission {}",
            signer, permission
        )));
    }

    Ok(())
}
