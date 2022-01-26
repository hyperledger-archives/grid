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

use std::collections::HashMap;

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
    pike::{addressing::GRID_PIKE_NAMESPACE, permissions::PermissionChecker},
    protocol::purchase_order::{
        payload::{
            Action, CreatePurchaseOrderPayload, CreateVersionPayload, PurchaseOrderPayload,
            UpdatePurchaseOrderPayload, UpdateVersionPayload,
        },
        state::{
            PurchaseOrderBuilder, PurchaseOrderRevision, PurchaseOrderRevisionBuilder,
            PurchaseOrderVersion, PurchaseOrderVersionBuilder,
        },
    },
    purchase_order::addressing::GRID_PURCHASE_ORDER_NAMESPACE,
    workflow::{Workflow, WorkflowState},
};

use crate::payload::validate_po_payload;
use crate::permissions::Permission;
use crate::state::PurchaseOrderState;
use crate::workflow::{get_workflow, WorkflowConstraint};

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
            namespaces: vec![
                GRID_PURCHASE_ORDER_NAMESPACE.to_string(),
                GRID_PIKE_NAMESPACE.to_string(),
            ],
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
            ApplyError::InvalidTransaction(format!(
                "Cannot deserialize purchase order payload: {}",
                err
            ))
        })?;
        validate_po_payload(&payload)?;
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
    /* ------------------ Access current state ------------------------ */
    let buyer_org_id = payload.buyer_org_id().to_string();
    let seller_org_id = payload.seller_org_id().to_string();
    let workflow_id = payload.workflow_id().to_string();

    // Check that the organizations owning the purchase order exist
    state.get_organization(&buyer_org_id)?.ok_or_else(|| {
        ApplyError::InvalidTransaction(format!("Organization {} does not exist", &buyer_org_id))
    })?;
    state.get_organization(&seller_org_id)?.ok_or_else(|| {
        ApplyError::InvalidTransaction(format!("Organization {} does not exist", &seller_org_id))
    })?;
    // Retrieve agent from state, error if agent does not exist
    let agent = state.get_agent(signer)?.ok_or_else(|| {
        ApplyError::InvalidTransaction(format!("The signer is not an Agent: {}", signer))
    })?;
    // Validate the purchase order does not already exist
    let po_uid = payload.uid();
    if state.get_purchase_order(po_uid)?.is_some() {
        return Err(ApplyError::InvalidTransaction(format!(
            "Purchase Order already exists: {}",
            po_uid,
        )));
    }
    // Validate the workflow type of the purchase order
    let workflow = get_workflow(&workflow_id).ok_or_else(|| {
        ApplyError::InvalidTransaction(format!("Cannot build `{}` Workflow", workflow_id))
    })?;

    // Retrieve the workflow for this purchase order
    let po_subworkflow = workflow.subworkflow("po").ok_or_else(|| {
        ApplyError::InvalidTransaction(format!(
            "Workflow `{}` does not contain a `po` subworkflow",
            payload.workflow_id(),
        ))
    })?;

    // Retrieve the workflow for this purchase order
    let version_subworkflow = workflow.subworkflow("version").ok_or_else(|| {
        ApplyError::InvalidTransaction(format!(
            "Workflow `{}` does not contain a `po` subworkflow",
            payload.workflow_id(),
        ))
    })?;

    // Get the desired workflow state of this purchase order
    let desired_po_workflow_state =
        po_subworkflow
            .state(payload.workflow_state())
            .ok_or_else(|| {
                ApplyError::InvalidTransaction(format!(
                    "Workflow state `{}` does not exist in `po` subworkflow",
                    payload.workflow_state()
                ))
            })?;

    /* ------------------ Verify submitter's permissions ------------------------ */
    let (versions, version_desired_state): (Vec<_>, Option<WorkflowState>) =
        match payload.create_version_payload() {
            Some(payload_version) => {
                let perm_string = Permission::CanCreatePoVersion.to_string();
                let desired_version_workflow_state = version_subworkflow
                    .state(payload_version.workflow_state())
                    .ok_or_else(|| {
                        ApplyError::InvalidTransaction(format!(
                            "Workflow state `{}` does not exist in version subworkflow",
                            payload_version.workflow_state()
                        ))
                    })?;
                let perm_result = perm_checker
                    .check_permission_to_enter_workflow(
                        &perm_string,
                        signer,
                        agent.org_id(),
                        version_subworkflow.start_state().ok_or_else(|| {
                            ApplyError::InvalidTransaction(
                                "Workflow start state does not exist in version subworkflow"
                                    .to_string(),
                            )
                        })?,
                        payload_version.workflow_state(),
                    )
                    .map_err(|err| {
                        ApplyError::InvalidTransaction(format!(
                            "Unable to check agent's permission: {}",
                            err
                        ))
                    })?;
                if !perm_result {
                    return Err(ApplyError::InvalidTransaction(format!(
                        "Agent {} does not have permission {} for organization {} to create a \
                        version with state {}",
                        signer,
                        &perm_string,
                        agent.org_id(),
                        payload_version.workflow_state()
                    )));
                }
                let payload_revision = payload_version.revision();

                let revision = PurchaseOrderRevisionBuilder::new()
                    .with_revision_id(payload_revision.revision_id())
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

                (
                    vec![PurchaseOrderVersionBuilder::new()
                        .with_version_id(payload_version.version_id().to_string())
                        .with_is_draft(payload_version.is_draft())
                        .with_current_revision_id(revision.revision_id())
                        .with_workflow_state(payload_version.workflow_state().to_string())
                        .with_revisions(vec![revision])
                        .with_workflow_state(payload_version.workflow_state().to_string())
                        .build()
                        .map_err(|err| {
                            ApplyError::InvalidTransaction(format!(
                                "Cannot build purchase order version: {}",
                                err
                            ))
                        })?],
                    Some(desired_version_workflow_state),
                )
            }
            None => (vec![], None),
        };

    let perm_string = Permission::CanCreatePo.to_string();
    let perm_result = perm_checker
        .check_permission_to_enter_workflow(
            &perm_string,
            signer,
            agent.org_id(),
            po_subworkflow.start_state().ok_or_else(|| {
                ApplyError::InvalidTransaction(
                    "Purchase order subworkflow does not have start state".to_string(),
                )
            })?,
            payload.workflow_state(),
        )
        .map_err(|err| {
            ApplyError::InvalidTransaction(format!("Unable to check agent's permission: {}", err))
        })?;
    if !perm_result {
        return Err(ApplyError::InvalidTransaction(format!(
            "Agent {} does not have permission {} for organization {} to create a purchase order \
                with state {}",
            signer,
            &perm_string,
            agent.org_id(),
            payload.workflow_state()
        )));
    }

    /* ------------------ Build updated state ------------------------ */

    let purchase_order = PurchaseOrderBuilder::new()
        .with_uid(po_uid.to_string())
        .with_versions(versions.to_vec())
        .with_workflow_state(payload.workflow_state().to_string())
        .with_alternate_ids(payload.alternate_ids().to_vec())
        .with_created_at(payload.created_at())
        .with_is_closed(false)
        .with_buyer_org_id(buyer_org_id)
        .with_seller_org_id(seller_org_id)
        .with_workflow_id(workflow_id)
        .build()
        .map_err(|err| {
            ApplyError::InvalidTransaction(format!("Cannot build purchase order: {}", err))
        })?;

    /* ------------------- Validate updated state ----------------------------- */
    if desired_po_workflow_state.has_constraint(&WorkflowConstraint::Closed.to_string()) {
        return Err(ApplyError::InvalidTransaction(format!(
            "Unable to open purchase order {} in workflow state `{}` with `closed` constraint",
            po_uid,
            payload.workflow_state(),
        )));
    }
    if desired_po_workflow_state.has_constraint(&WorkflowConstraint::Complete.to_string())
        && purchase_order.versions().is_empty()
    {
        return Err(ApplyError::InvalidTransaction(format!(
            "Unable to open purchase order {} without versions defined in workflow state `{}` with \
            `complete` constraint",
            po_uid,
            payload.workflow_state(),
        )));
    }
    if desired_po_workflow_state.has_constraint(&WorkflowConstraint::Accepted.to_string()) {
        return Err(ApplyError::InvalidTransaction(format!(
            "Unable to open purchase order {} in workflow state `{}` with `accepted` constraint, \
            no version has been accepted yet",
            po_uid,
            payload.workflow_state(),
        )));
    }
    if let Some(desired_version_state) = version_desired_state {
        if desired_version_state.has_constraint(&WorkflowConstraint::Accepted.to_string()) {
            return Err(ApplyError::InvalidTransaction(format!(
                "Unable to open purchase order version {} in workflow state `{}` with `accepted` \
                constraint, no version has been accepted yet",
                po_uid,
                payload.workflow_state(),
            )));
        }
        let version = versions.last().ok_or_else(|| {
            ApplyError::InvalidTransaction(
                "Unable to retrieve purchase order's version".to_string(),
            )
        })?;
        if desired_version_state.has_constraint(&WorkflowConstraint::Draft.to_string())
            && !version.is_draft()
        {
            return Err(ApplyError::InvalidTransaction(format!(
                "Unable to open purchase order {} version {} in workflow state `{}` with \
                    `draft` constraint, version's `is_draft` field is `false`",
                po_uid,
                version.version_id(),
                payload.workflow_state(),
            )));
        }
    }

    /* ------------------- Persist updated state ----------------------------- */
    state.set_purchase_order(po_uid, purchase_order)?;
    Ok(())
}

fn update_purchase_order(
    payload: &UpdatePurchaseOrderPayload,
    signer: &str,
    state: &mut PurchaseOrderState,
    perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    /* ------------------ Access current state ------------------------ */

    // Validate agent exists
    let agent = state.get_agent(signer)?.ok_or_else(|| {
        ApplyError::InvalidTransaction(format!("The signer is not an agent: {}", signer))
    })?;
    // Validate purchase order exists
    let po_uid = payload.uid();
    let purchase_order = match state.get_purchase_order(po_uid)? {
        Some(purchase_order) => Ok(purchase_order),
        None => Err(ApplyError::InvalidTransaction(format!(
            "No purchase order exists: {}",
            po_uid
        ))),
    }?;
    // Collect the versions from this purchase order
    let mut existing_versions = purchase_order.versions().to_vec().into_iter();
    // Lists existing versions if there is also an update payload for the version
    let mut version_updates = payload
        .version_updates()
        .iter()
        .map(|update| {
            let corresponding_version = existing_versions
                .find(|vers| vers.version_id() == update.version_id())
                .ok_or_else(|| {
                    ApplyError::InvalidTransaction(format!(
                        "Cannot update version {} that does not exist",
                        update.version_id()
                    ))
                })?;
            Ok((update.version_id().to_string(), corresponding_version))
        })
        .collect::<Result<HashMap<String, PurchaseOrderVersion>, ApplyError>>()?;
    let workflow = get_workflow(purchase_order.workflow_id()).ok_or_else(|| {
        ApplyError::InvalidTransaction(format!(
            "Cannot build workflow type {}",
            purchase_order.workflow_id()
        ))
    })?;
    let existing_po_workflow_state = workflow
        .subworkflow("po")
        .ok_or_else(|| {
            ApplyError::InvalidTransaction("Subworkflow `po` does not exist".to_string())
        })?
        .state(purchase_order.workflow_state())
        .ok_or_else(|| {
            ApplyError::InvalidTransaction(format!(
                "Workflow state `{}` does not exist in `po` subworkflow",
                purchase_order.workflow_state()
            ))
        })?;
    let desired_po_workflow_state = workflow
        .subworkflow("po")
        .ok_or_else(|| {
            ApplyError::InvalidTransaction("Subworkflow `po` does not exist".to_string())
        })?
        .state(payload.workflow_state())
        .ok_or_else(|| {
            ApplyError::InvalidTransaction(format!(
                "Workflow state `{}` does not exist in `po` subworkflow",
                purchase_order.workflow_state()
            ))
        })?;

    let version_subworkflow = workflow.subworkflow("version").ok_or_else(|| {
        ApplyError::InvalidTransaction("Subworkflow `version` does not exist".to_string())
    })?;

    /* ------------------ Verify submitter's permissions ------------------------ */

    // Check if the agent has permission to update the purchase order
    let perm_string = if payload.workflow_state() == purchase_order.workflow_state() {
        // Updates within the same state require CanUpdatePo
        Permission::CanUpdatePo
    } else {
        // Updates from one state to another require that specific transition permission
        Permission::can_transition(payload.workflow_state()).ok_or_else(|| {
            ApplyError::InvalidTransaction(format!(
                "No permission exists to allow transitioning to a state of `{}`",
                payload.workflow_state()
            ))
        })?
    };
    let perm_result = perm_checker
        .check_permission_within_workflow(
            &perm_string.to_string(),
            signer,
            agent.org_id(),
            &existing_po_workflow_state,
            payload.workflow_state(),
        )
        .map_err(|err| {
            ApplyError::InvalidTransaction(format!("Unable to check agent's permission: {}", err))
        })?;
    if !perm_result {
        return Err(ApplyError::InvalidTransaction(format!(
            "Agent {} does not have the correct permissions to update \
                     purchase order {} from a state of {} to {}",
            signer,
            po_uid,
            purchase_order.workflow_state(),
            payload.workflow_state()
        )));
    }
    // Validate permissions for version updates and convert to state object
    let mut updated_versions = payload
        .version_updates()
        .iter()
        .map(|update_payload| {
            let existing_version = version_updates
                .remove(update_payload.version_id())
                .ok_or_else(|| {
                    ApplyError::InvalidTransaction(format!(
                        "Cannot update version {} that does not exist",
                        update_payload.version_id()
                    ))
                })?;
            // check the permission of the individual version update
            validate_version_update_permissions(
                &existing_version,
                update_payload,
                signer,
                agent.org_id(),
                &workflow,
                perm_checker,
            )?;
            convert_update_to_version(existing_version, update_payload)
        })
        .collect::<Result<HashMap<String, PurchaseOrderVersion>, ApplyError>>()?;

    /* ------------------- Create new state ----------------------------- */

    // Meld version updates to those into this purchase order's existing versions
    let versions = purchase_order
        .versions()
        .to_vec()
        .into_iter()
        .map(|vers| {
            if let Some(updated_vers) = updated_versions.remove(vers.version_id()) {
                Ok(updated_vers)
            } else {
                Ok(vers)
            }
        })
        .collect::<Result<Vec<PurchaseOrderVersion>, ApplyError>>()?;

    // Create the updated purchase_order
    let mut builder = PurchaseOrderBuilder::new()
        .with_uid(po_uid.to_string())
        .with_workflow_state(payload.workflow_state().to_string())
        .with_alternate_ids(payload.alternate_ids().to_vec())
        .with_is_closed(payload.is_closed())
        .with_versions(versions)
        .with_created_at(purchase_order.created_at())
        .with_buyer_org_id(purchase_order.buyer_org_id().to_string())
        .with_seller_org_id(purchase_order.seller_org_id().to_string())
        .with_workflow_id(purchase_order.workflow_id().to_string());

    if let Some(vers) = payload.accepted_version_number() {
        builder = builder.with_accepted_version_number(vers.to_string());
    }

    let updated_po = builder.build().map_err(|err| {
        ApplyError::InvalidTransaction(format!("Cannot build purchase order: {}", err))
    })?;

    /* ------------------- Validate updated state ----------------------------- */

    if updated_po.is_closed() {
        // Validate the accepted version number
        if let Some(accepted_version_number) = updated_po.accepted_version_number() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Accepted version number {} set for closed purchase order {}. Expected accepted \
                version number to be empty",
                accepted_version_number, po_uid,
            )));
        }

        // Validate the workflow is not set to complete
        if desired_po_workflow_state.has_constraint(&WorkflowConstraint::Complete.to_string()) {
            return Err(ApplyError::InvalidTransaction(format!(
                "Workflow state `{}` set for closed purchase order {}. Expected workflow \
                state not to be closed for a complete purchase order",
                updated_po.workflow_state(),
                po_uid,
            )));
        }

        // Validate the workflow is closed
        if !desired_po_workflow_state.has_constraint(&WorkflowConstraint::Closed.to_string()) {
            return Err(ApplyError::InvalidTransaction(format!(
                "Workflow state `{}` does not have `closed` constraint, but purchase order {} \
                is closed",
                updated_po.workflow_state(),
                po_uid,
            )));
        }
    } else {
        // Validate the workflow is not closed
        if desired_po_workflow_state.has_constraint(&WorkflowConstraint::Closed.to_string()) {
            return Err(ApplyError::InvalidTransaction(format!(
                "The desired workflow state {} for purchase order {} has the `closed` constraint, \
                but property `is_closed` was set to false.",
                updated_po.workflow_state(),
                po_uid,
            )));
        }
    }

    if updated_po.accepted_version_number().is_none()
        && desired_po_workflow_state.has_constraint(&WorkflowConstraint::Accepted.to_string())
    {
        return Err(ApplyError::InvalidTransaction(format!(
            "Workflow state `{}` set for purchase order {} has `accepted` constraint, but no \
            version is accepted",
            updated_po.workflow_state(),
            po_uid,
        )));
    }

    let mut updated_version_states = updated_po
        .versions()
        .iter()
        .map(|vers| {
            let vers_state = version_subworkflow
                .state(vers.workflow_state())
                .ok_or_else(|| {
                    ApplyError::InvalidTransaction(format!(
                        "Workflow state `{}` does not exist in `version` subworkflow",
                        vers.workflow_state()
                    ))
                })?;
            Ok((vers.version_id(), vers_state))
        })
        .collect::<Result<HashMap<&str, WorkflowState>, ApplyError>>()?;

    if let Some(accepted_version_number) =
        updated_po.accepted_version_number().map(ToOwned::to_owned)
    {
        // Remove the accepted version from the list of versions
        let accepted_vers_state = updated_version_states
            .remove(accepted_version_number.as_str())
            .ok_or_else(|| {
                ApplyError::InvalidTransaction(format!(
                    "Cannot accept purchase order {} version {} that does not exist",
                    updated_po.uid(),
                    accepted_version_number,
                ))
            })?;

        // Validate this version is not a draft
        if accepted_vers_state.has_constraint(&WorkflowConstraint::Draft.to_string()) {
            return Err(ApplyError::InvalidTransaction(format!(
                "Workflow state `{}` set for purchase order {} version {}, \
                cannot accept draft version",
                accepted_vers_state.name(),
                updated_po.uid(),
                accepted_version_number
            )));
        }

        // Check that the purchase order is in an "accepted"-constrained state
        if !desired_po_workflow_state.has_constraint(&WorkflowConstraint::Accepted.to_string()) {
            return Err(ApplyError::InvalidTransaction(format!(
                "Purchase order {} has accepted version {}, but po workflow state `{}` does not \
                have an `accepted` costraint",
                updated_po.uid(),
                accepted_version_number,
                updated_po.workflow_state(),
            )));
        }

        // Check that the accepted version is in an "accepted"-constrained state
        if !accepted_vers_state.has_constraint(&WorkflowConstraint::Accepted.to_string()) {
            return Err(ApplyError::InvalidTransaction(format!(
                "Purchase order {} has accepted version {}, but version workflow state `{}` does \
                not have an `accepted` costraint",
                updated_po.uid(),
                accepted_version_number,
                accepted_vers_state.name(),
            )));
        }

        // Retain a list of versions that were not the "accepted_version_number" that are still in
        // an invalid "accepted"-constraint state
        if updated_version_states
            .into_iter()
            .filter_map(|(vers_id, workflow_state)| {
                if workflow_state.has_constraint(&WorkflowConstraint::Accepted.to_string()) {
                    Some(vers_id)
                } else {
                    None
                }
            })
            .next()
            .is_some()
        {
            return Err(ApplyError::InvalidTransaction(format!(
                "Attempting to accept purchase order {} version {}, but other versions are already \
                accepted",
                updated_po.uid(),
                accepted_version_number,
            )));
        }
    } else {
        // Check if we have removed a version as "accepted"
        if let Some((vers_id, workflow_state)) =
            updated_version_states
                .into_iter()
                .find(|(_vers_id, workflow_state)| {
                    workflow_state.has_constraint(&WorkflowConstraint::Accepted.to_string())
                })
        {
            return Err(ApplyError::InvalidTransaction(format!(
                "Purchase order {} does not specify an accepted version, but version \
                    {} workflow state {} has `accepted` constraint",
                updated_po.uid(),
                vers_id,
                workflow_state.name(),
            )));
        }
    }

    /* ------------------- Persist updated state ----------------------------- */

    state.set_purchase_order(po_uid, updated_po)?;

    Ok(())
}

fn validate_version_update_permissions(
    existing_version: &PurchaseOrderVersion,
    payload: &UpdateVersionPayload,
    signer: &str,
    agent_org_id: &str,
    workflow: &Workflow,
    perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    let existing_version_workflow_state = workflow
        .subworkflow("version")
        .ok_or_else(|| {
            ApplyError::InvalidTransaction(format!(
                "Workflow `{}` does not contain a `po` subworkflow",
                payload.workflow_state(),
            ))
        })?
        .state(existing_version.workflow_state())
        .ok_or_else(|| {
            ApplyError::InvalidTransaction(format!(
                "Workflow state `{}` does not exist in version subworkflow",
                payload.workflow_state()
            ))
        })?;
    let perm_string = if existing_version.workflow_state() == payload.workflow_state() {
        Permission::CanUpdatePoVersion.to_string()
    } else {
        Permission::can_transition(payload.workflow_state())
            .ok_or_else(|| {
                ApplyError::InvalidTransaction(format!(
                    "Transition permission does not exist for `{}` workflow state",
                    payload.workflow_state()
                ))
            })?
            .to_string()
    };
    let perm_result = perm_checker
        .check_permission_within_workflow(
            &perm_string,
            signer,
            agent_org_id,
            &existing_version_workflow_state,
            payload.workflow_state(),
        )
        .map_err(|err| {
            ApplyError::InvalidTransaction(format!("Unable to check agent's permission: {}", err))
        })?;

    if !perm_result {
        return Err(ApplyError::InvalidTransaction(format!(
            "Agent {} does not have permission to update version {} to workflow state `{}`",
            signer,
            payload.version_id(),
            payload.workflow_state(),
        )));
    }

    Ok(())
}

fn convert_update_to_version(
    existing_version: PurchaseOrderVersion,
    payload: &UpdateVersionPayload,
) -> Result<(String, PurchaseOrderVersion), ApplyError> {
    let version_id = existing_version.version_id().to_string();
    let new_revision = PurchaseOrderRevisionBuilder::new()
        .with_revision_id(payload.revision().revision_id())
        .with_submitter(payload.revision().submitter().to_string())
        .with_created_at(payload.revision().created_at())
        .with_order_xml_v3_4(payload.revision().order_xml_v3_4().to_string())
        .build()
        .map_err(|err| {
            ApplyError::InvalidTransaction(format!(
                "Unable to build revision {} of purchase order version {}: {}",
                payload.revision().revision_id(),
                payload.version_id(),
                err
            ))
        })?;
    // Check if a new revision is included in the update
    let (current_revision_id, rev_addition): (u64, Option<PurchaseOrderRevision>) =
        match existing_version.revisions().to_vec().into_iter().last() {
            Some(last_rev) => {
                if last_rev == new_revision {
                    (last_rev.revision_id(), None)
                } else {
                    (last_rev.revision_id() + 1, Some(new_revision))
                }
            }
            None => (1, Some(new_revision)),
        };
    let mut existing_revisions = existing_version.revisions().to_vec();
    if let Some(revision) = rev_addition {
        // Update the revision ID of the payload's revision
        let new_revision = revision
            .into_builder()
            .with_revision_id(current_revision_id)
            .build()
            .map_err(|err| {
                ApplyError::InvalidTransaction(format!(
                    "Cannot build purchase order version: {}",
                    err
                ))
            })?;
        existing_revisions.push(new_revision);
    }
    // Update the corresponding version
    let updated_version = existing_version
        .into_builder()
        .with_workflow_state(payload.workflow_state().to_string())
        .with_is_draft(payload.is_draft())
        .with_current_revision_id(current_revision_id)
        .with_revisions(existing_revisions)
        .build()
        .map_err(|err| {
            ApplyError::InvalidTransaction(format!("Cannot build purchase order version: {}", err))
        })?;
    Ok((version_id, updated_version))
}

fn create_version(
    payload: &CreateVersionPayload,
    signer: &str,
    state: &mut PurchaseOrderState,
    perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    /* ------------------ Access current state ------------------------ */
    // Validate the signer exists as an agent and retrieve the agent's organization ID
    let agent_org_id = state
        .get_agent(signer)?
        .ok_or_else(|| {
            ApplyError::InvalidTransaction(format!("Signer {} does not exist as an agent", signer))
        })?
        .org_id()
        .to_string();
    // Retrieve the workflow this version belongs to, by obtaining the specified purchase order
    let workflow_id = match state.get_purchase_order(payload.po_uid())? {
        Some(po) => Ok(po.workflow_id().to_string()),
        None => Err(ApplyError::InvalidTransaction(format!(
            "Purchase order {} does not exist",
            payload.po_uid(),
        ))),
    }?;
    // Validate this version does not already exist
    if state
        .get_purchase_order_version(payload.po_uid(), payload.version_id())?
        .is_some()
    {
        return Err(ApplyError::InvalidTransaction(format!(
            "Version {} already exists for Purchase Order {}",
            payload.version_id(),
            payload.po_uid(),
        )));
    }

    // Retrieve the workflow state we will put the version in
    let desired_workflow_state = get_workflow(&workflow_id)
        .ok_or_else(|| {
            ApplyError::InvalidTransaction(format!("Workflow `{}` does not exist", &workflow_id))
        })?
        .subworkflow("version")
        .ok_or_else(|| {
            ApplyError::InvalidTransaction("Subworkflow `version` does not exist".to_string())
        })?
        .state(payload.workflow_state())
        .ok_or_else(|| {
            ApplyError::InvalidTransaction(format!(
                "Workflow state `{}` does not exist in `version` subworkflow",
                payload.workflow_state()
            ))
        })?;
    // Get the start state from the version subworkflow, to validate if we are able to
    // create this version
    let version_subworkflow = get_workflow(&workflow_id)
        .ok_or_else(|| {
            ApplyError::InvalidTransaction(format!("Workflow `{}` does not exist", &workflow_id))
        })?
        .subworkflow("version")
        .ok_or_else(|| {
            ApplyError::InvalidTransaction("Subworkflow `version` does not exist".to_string())
        })?;
    let start_workflow_state = version_subworkflow.start_state().ok_or_else(|| {
        ApplyError::InvalidTransaction(
            "Workflow start state does not exist in `version` subworkflow".to_string(),
        )
    })?;

    /* ------------------ Verify submitter's permissions ------------------------ */

    // Validate the agent is able to create the purchase order version
    let perm_string = Permission::CanCreatePoVersion.to_string();
    let perm_result = perm_checker
        .check_permission_to_enter_workflow(
            &perm_string,
            signer,
            &agent_org_id,
            start_workflow_state,
            payload.workflow_state(),
        )
        .map_err(|err| {
            ApplyError::InvalidTransaction(format!("Unable to check agent's permission: {}", err))
        })?;

    if !perm_result {
        return Err(ApplyError::InvalidTransaction(format!(
            "Agent {} does not have the correct permissions for organization {} to create purchase \
             order version {} in the {} workflow state",
            signer,
            &agent_org_id,
            payload.version_id(),
            payload.workflow_state(),
        )));
    }

    /* ------------------ Build updated state ------------------------ */

    // Create the PurchaseOrderRevision to be added to the version
    let payload_revision = payload.revision();
    let revision = PurchaseOrderRevisionBuilder::new()
        .with_revision_id(payload_revision.revision_id())
        .with_submitter(payload_revision.submitter().to_string())
        .with_created_at(payload_revision.created_at())
        .with_order_xml_v3_4(payload_revision.order_xml_v3_4().to_string())
        .build()
        .map_err(|err| {
            ApplyError::InvalidTransaction(format!("Cannot build purchase order revision: {}", err))
        })?;
    // Create the PurchaseOrderVersion to be added to state
    let new_version = PurchaseOrderVersionBuilder::new()
        .with_version_id(payload.version_id().to_string())
        .with_workflow_state(payload.workflow_state().to_string())
        .with_is_draft(payload.is_draft())
        .with_current_revision_id(revision.revision_id())
        .with_revisions(vec![revision])
        .build()
        .map_err(|err| {
            ApplyError::InvalidTransaction(format!("Cannot build purchase order version: {}", err))
        })?;

    /* ------------------- Validate updated state ----------------------------- */

    // Check the desired workflow state for any constraints it may have related to the version
    if desired_workflow_state.has_constraint(&WorkflowConstraint::Accepted.to_string())
        && new_version.is_draft()
    {
        return Err(ApplyError::InvalidTransaction(
            "Desired workflow state has `accepted` constraint, version is a draft".to_string(),
        ));
    }
    if desired_workflow_state.has_constraint(&WorkflowConstraint::Draft.to_string())
        && !new_version.is_draft()
    {
        return Err(ApplyError::InvalidTransaction(
            "Desired workflow state has `draft` constraint, version is not a draft".to_string(),
        ));
    }

    /* ------------------- Persist updated state ----------------------------- */
    state.set_purchase_order_version(payload.po_uid(), new_version)?;
    Ok(())
}

fn update_version(
    payload: &UpdateVersionPayload,
    signer: &str,
    state: &mut PurchaseOrderState,
    perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    /* ------------------ Access current state ------------------------ */
    // Validate the signer exists as an agent
    let agent = state.get_agent(signer)?.ok_or_else(|| {
        ApplyError::InvalidTransaction(format!("The signer is not an agent: {}", signer))
    })?;
    // Validate this version exists to be updated
    let original_version =
        match state.get_purchase_order_version(payload.po_uid(), payload.version_id()) {
            Ok(Some(po_version)) => Ok(po_version),
            Ok(None) => Err(ApplyError::InvalidTransaction(format!(
                "No version {} exists for purchase order {}",
                payload.version_id(),
                payload.po_uid()
            ))),
            Err(err) => Err(err),
        }?;
    // Retrieving the type of workflow used for this purchase order
    let original_po = state.get_purchase_order(payload.po_uid())?.ok_or_else(|| {
        ApplyError::InvalidTransaction(format!(
            "Purchase order {} does not exist",
            payload.po_uid()
        ))
    })?;
    let original_po_workflow_state = get_workflow(original_po.workflow_id())
        .ok_or_else(|| {
            ApplyError::InvalidTransaction(format!(
                "Cannot build workflow {}",
                original_po.workflow_id(),
            ))
        })?
        .subworkflow("po")
        .ok_or_else(|| {
            ApplyError::InvalidTransaction("Subworkflow `po` does not exist".to_string())
        })?
        .state(original_po.workflow_state())
        .ok_or_else(|| {
            ApplyError::InvalidTransaction(format!(
                "Workflow state `{}` does not exist in `po` subworkflow",
                original_po.workflow_state()
            ))
        })?;
    let version_subworkflow = get_workflow(original_po.workflow_id())
        .ok_or_else(|| {
            ApplyError::InvalidTransaction(format!(
                "Cannot build workflow {}",
                original_po.workflow_id(),
            ))
        })?
        .subworkflow("version")
        .ok_or_else(|| {
            ApplyError::InvalidTransaction("Subworkflow `version` does not exist".to_string())
        })?;
    let original_version_workflow_state = version_subworkflow
        .state(original_version.workflow_state())
        .ok_or_else(|| {
            ApplyError::InvalidTransaction(format!(
                "Workflow state `{}` does not exist in `version` subworkflow",
                original_version.workflow_state()
            ))
        })?;
    /* ------------------ Verify submitter's permissions ------------------------ */
    // Check if the agent has permission to update the version
    let perm_string = if payload.workflow_state() == original_version.workflow_state() {
        Permission::CanUpdatePoVersion
    } else {
        Permission::can_transition(payload.workflow_state()).ok_or_else(|| {
            ApplyError::InvalidTransaction(format!(
                "No permission exists to allow transitioning to a state of `{}`",
                payload.workflow_state()
            ))
        })?
    };
    // Validate the submitter is allowed to perform the action
    let perm_result = perm_checker
        .check_permission_within_workflow(
            &perm_string.to_string(),
            signer,
            agent.org_id(),
            &original_version_workflow_state,
            payload.workflow_state(),
        )
        .map_err(|err| {
            ApplyError::InvalidTransaction(format!("Unable to check agent's permission: {}", err))
        })?;
    if !perm_result {
        return Err(ApplyError::InvalidTransaction(format!(
            "Agent {} does not have the correct permissions to update \
                     purchase order version {} from a state of {} to {}",
            signer,
            original_version.version_id(),
            original_version.workflow_state(),
            payload.workflow_state(),
        )));
    }

    /* ------------------- Create new state ----------------------------- */

    // Create the PurchaseOrderRevision to be added to the version
    let mut new_revision = PurchaseOrderRevisionBuilder::new()
        .with_revision_id(payload.revision().revision_id())
        .with_submitter(payload.revision().submitter().to_string())
        .with_created_at(payload.revision().created_at())
        .with_order_xml_v3_4(payload.revision().order_xml_v3_4().to_string())
        .build()
        .map_err(|err| {
            ApplyError::InvalidTransaction(format!("Cannot build purchase order revision: {}", err))
        })?;
    // Check if we are adding a new revision within this update
    let (current_revision_id, current_revisions) = if original_version
        .revisions()
        .to_vec()
        .contains(&new_revision)
    {
        (
            new_revision.revision_id(),
            original_version.revisions().to_vec(),
        )
    } else {
        // Make sure the revision ID is incremented from the previous `current_revision_id`
        let new_rev_id = original_version
            .revisions()
            .iter()
            .last()
            .map(|rev| rev.revision_id() + 1)
            .unwrap_or(1);
        new_revision = new_revision
            .into_builder()
            .with_revision_id(new_rev_id)
            .build()
            .map_err(|err| {
                ApplyError::InvalidTransaction(format!("Cannot build revision: {}", err))
            })?;
        // Updating the `current_revision_id` requires adding the revision to the version's
        // `revisions`
        let mut revisions = original_version.revisions().to_vec();
        revisions.push(new_revision);
        (new_rev_id, revisions)
    };

    let updated_version = original_version
        .into_builder()
        .with_workflow_state(payload.workflow_state().to_string())
        .with_is_draft(payload.is_draft())
        .with_current_revision_id(current_revision_id)
        .with_revisions(current_revisions)
        .build()
        .map_err(|err| {
            ApplyError::InvalidTransaction(format!("Cannot build purchase order version: {}", err))
        })?;

    /* ------------------- Validate updated state ----------------------------- */

    let update_version_workflow_state = version_subworkflow
        .state(updated_version.workflow_state())
        .ok_or_else(|| {
            ApplyError::InvalidTransaction(format!(
                "Workflow state `{}` does not exist in `version` subworkflow",
                updated_version.workflow_state()
            ))
        })?;

    if update_version_workflow_state.has_constraint(&WorkflowConstraint::Draft.to_string())
        && !updated_version.is_draft()
    {
        // Validate this purchase order version is a draft version
        return Err(ApplyError::InvalidTransaction(format!(
            "Workflow state {} has `draft` constraint, updated version is not a draft",
            updated_version.workflow_state()
        )));
    }
    // The purchase order may need to be updated if this version is accepted
    if update_version_workflow_state.has_constraint(&WorkflowConstraint::Accepted.to_string()) {
        let updated_accepted_version = updated_version.version_id();

        if !original_po_workflow_state.has_constraint(&WorkflowConstraint::Accepted.to_string()) {
            // Validate the purchase order is in an `accepted` workflow state
            return Err(ApplyError::InvalidTransaction(format!(
                "Purchase order {} workflow state `{}` does not have `accepted` constraint, \
                cannot accept version {}",
                original_po.uid(),
                original_po.workflow_state(),
                updated_accepted_version,
            )));
        }

        if original_po.accepted_version_number().is_none()
            || original_po.accepted_version_number() != Some(updated_accepted_version)
        {
            return Err(ApplyError::InvalidTransaction(format!(
                "Purchase order {} refers to accepted version '{:?}', must reference newly \
                accepted version {}",
                original_po.uid(),
                original_po.accepted_version_number(),
                updated_accepted_version,
            )));
        }

        if original_po
            .versions()
            .iter()
            .filter_map(|version| {
                let version_workflow_state = version_subworkflow
                    .state(version.workflow_state())
                    .filter(|v_wfs| {
                        v_wfs.has_constraint(&WorkflowConstraint::Accepted.to_string())
                    });
                if version_workflow_state.is_some()
                    && version.version_id() != updated_accepted_version
                {
                    Some(version.version_id())
                } else {
                    None
                }
            })
            .next()
            .is_some()
        {
            return Err(ApplyError::InvalidTransaction(format!(
                "Attempting to accept purchase order {} version {}, but other versions are \
                    already accepted",
                original_po.uid(),
                updated_accepted_version,
            )));
        }

        // Validate this purchase order version is able to be accepted
        if updated_version.is_draft() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Desired workflow state `{}` has `accepted` constraint, version is a draft",
                updated_version.workflow_state()
            )));
        }
    } else {
        // Check if we are moving this version out of an `accepted`-constrained state
        if original_po.accepted_version_number() == Some(updated_version.version_id()) {
            return Err(ApplyError::InvalidTransaction(format!(
                "Attempting to transition purchase order {} version {} to `{}` workflow state, \
                but purchase order still specifies this version as accepted",
                original_po.uid(),
                updated_version.version_id(),
                updated_version.workflow_state(),
            )));
        }
    }

    /* ------------------- Persist updated state ----------------------------- */

    state.set_purchase_order_version(payload.po_uid(), updated_version)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;

    use grid_sdk::{
        pike::addressing::{
            compute_agent_address, compute_organization_address, compute_role_address,
        },
        protocol::pike::state::{
            AgentBuilder, AgentListBuilder, OrganizationBuilder, OrganizationListBuilder,
            RoleBuilder, RoleListBuilder,
        },
        protocol::purchase_order::{
            payload::{
                CreatePurchaseOrderPayloadBuilder, CreateVersionPayloadBuilder,
                PayloadRevisionBuilder, UpdatePurchaseOrderPayloadBuilder,
                UpdateVersionPayloadBuilder,
            },
            state::{
                PurchaseOrder, PurchaseOrderBuilder, PurchaseOrderListBuilder,
                PurchaseOrderRevision, PurchaseOrderRevisionBuilder, PurchaseOrderVersion,
                PurchaseOrderVersionBuilder,
            },
        },
        protos::IntoBytes,
        purchase_order::addressing::compute_purchase_order_address,
    };
    use sawtooth_sdk::processor::handler::{ContextError, TransactionContext};

    use crate::workflow::POWorkflow;

    const BUYER_PUB_KEY: &str = "buyer_agent_pub_key";
    const SELLER_PUB_KEY: &str = "seller_agent_pub_key";
    const PARTNER_PUB_KEY: &str = "partner_agent_pub_key";
    const DRAFT_PUB_KEY: &str = "draft_agent_pub_key";
    const EDITOR_PUB_KEY: &str = "editor_agent_pub_key";

    const PO_UID: &str = "test_po_1";
    const PO_VERSION_ID_1: &str = "01";
    const PO_VERSION_ID_2: &str = "02";

    const ROLE_BUYER: &str = "buyer";
    const PERM_ALIAS_BUYER: &str = "po::buyer";

    const ROLE_SELLER: &str = "seller";
    const PERM_ALIAS_SELLER: &str = "po::seller";

    const ROLE_DRAFT: &str = "draft";
    const PERM_ALIAS_DRAFT: &str = "po::draft";

    const ROLE_PARTNER: &str = "partner";
    const PERM_ALIAS_PARTNER: &str = "po::partner";

    const ROLE_EDITOR: &str = "editor";
    const PERM_ALIAS_EDITOR: &str = "po::editor";

    const ORG_ID_1: &str = "test_org_1";
    const ORG_ID_2: &str = "test_org_2";

    #[derive(Default, Debug)]
    /// A MockTransactionContext that can be used to test ProductState
    struct MockTransactionContext {
        state: RefCell<HashMap<String, Vec<u8>>>,
    }

    impl TransactionContext for MockTransactionContext {
        fn get_state_entries(
            &self,
            addresses: &[String],
        ) -> Result<Vec<(String, Vec<u8>)>, ContextError> {
            let mut results = Vec::new();
            for addr in addresses {
                let data = match self.state.borrow().get(addr) {
                    Some(data) => data.clone(),
                    None => Vec::new(),
                };
                results.push((addr.to_string(), data));
            }
            Ok(results)
        }

        fn set_state_entries(&self, entries: Vec<(String, Vec<u8>)>) -> Result<(), ContextError> {
            for (addr, data) in entries {
                self.state.borrow_mut().insert(addr, data);
            }
            Ok(())
        }

        /// this is not needed for these tests
        fn delete_state_entries(&self, _addresses: &[String]) -> Result<Vec<String>, ContextError> {
            unimplemented!()
        }

        /// this is not needed for these tests
        fn add_receipt_data(&self, _data: &[u8]) -> Result<(), ContextError> {
            unimplemented!()
        }

        /// this is not needed for these tests
        fn add_event(
            &self,
            _event_type: String,
            _attributes: Vec<(String, String)>,
            _data: &[u8],
        ) -> Result<(), ContextError> {
            unimplemented!()
        }
    }

    impl MockTransactionContext {
        fn add_buyer_role(&self) {
            let buyer_role = RoleBuilder::new()
                .with_org_id(ORG_ID_1.to_string())
                .with_name(ROLE_BUYER.to_string())
                .with_description("Purchase Order buyer role".to_string())
                .with_permissions(vec![PERM_ALIAS_BUYER.to_string()])
                .build()
                .expect("Unable to build role");
            let role_list = RoleListBuilder::new()
                .with_roles(vec![buyer_role])
                .build()
                .expect("Unable to build role list");
            let role_bytes = role_list
                .into_bytes()
                .expect("Unable to convert role list to bytes");
            let role_address = compute_role_address(ROLE_BUYER, ORG_ID_1);
            self.set_state_entry(role_address, role_bytes)
                .expect("Unable to set role in state");
        }

        fn add_buyer_agent(&self) {
            let agent = AgentBuilder::new()
                .with_org_id(ORG_ID_1.to_string())
                .with_public_key(BUYER_PUB_KEY.to_string())
                .with_active(true)
                .with_roles(vec![ROLE_BUYER.to_string()])
                .build()
                .expect("Unable to build agent");

            let agent_list = AgentListBuilder::new()
                .with_agents(vec![agent])
                .build()
                .expect("Unable to build agent list");
            let agent_bytes = agent_list
                .into_bytes()
                .expect("Unable to convert agent list to bytes");
            let agent_address = compute_agent_address(BUYER_PUB_KEY);
            self.set_state_entry(agent_address, agent_bytes)
                .expect("Unable to add agent to state");
        }

        fn add_seller_role(&self) {
            let role = RoleBuilder::new()
                .with_org_id(ORG_ID_2.to_string())
                .with_name(ROLE_SELLER.to_string())
                .with_description("Purchase Order seller role".to_string())
                .with_permissions(vec![PERM_ALIAS_SELLER.to_string()])
                .build()
                .expect("Unable to build role");

            let role_list = RoleListBuilder::new()
                .with_roles(vec![role])
                .build()
                .expect("Unable to build role list");
            let role_bytes = role_list.into_bytes().unwrap();
            let role_address = compute_role_address(ROLE_SELLER, ORG_ID_2);
            self.set_state_entry(role_address, role_bytes)
                .expect("Unable to add role to state");
        }

        fn add_seller_agent(&self) {
            let agent = AgentBuilder::new()
                .with_org_id(ORG_ID_2.to_string())
                .with_public_key(SELLER_PUB_KEY.to_string())
                .with_active(true)
                .with_roles(vec![ROLE_SELLER.to_string()])
                .build()
                .expect("Unable to build agent");
            let agent_list = AgentListBuilder::new()
                .with_agents(vec![agent])
                .build()
                .expect("Unable to build agent list");
            let agent_bytes = agent_list
                .into_bytes()
                .expect("Unable to convert agent list to bytes");
            let agent_address = compute_agent_address(SELLER_PUB_KEY);
            self.set_state_entry(agent_address, agent_bytes)
                .expect("Unable to set agent in state");
        }

        fn add_draft_role(&self) {
            let draft_role = RoleBuilder::new()
                .with_org_id(ORG_ID_1.to_string())
                .with_name(ROLE_DRAFT.to_string())
                .with_description("Purchase Order drafting role".to_string())
                .with_permissions(vec![PERM_ALIAS_DRAFT.to_string()])
                .build()
                .expect("Unable to build role");
            let role_list = RoleListBuilder::new()
                .with_roles(vec![draft_role])
                .build()
                .expect("Unable to build role list");
            let role_bytes = role_list
                .into_bytes()
                .expect("Unable to convert role list to bytes");
            let role_address = compute_role_address(ROLE_DRAFT, ORG_ID_1);
            self.set_state_entry(role_address, role_bytes)
                .expect("Unable to set role in state");
        }

        fn add_drafting_agent(&self) {
            let agent = AgentBuilder::new()
                .with_org_id(ORG_ID_1.to_string())
                .with_public_key(DRAFT_PUB_KEY.to_string())
                .with_active(true)
                .with_roles(vec![ROLE_DRAFT.to_string()])
                .build()
                .expect("Unable to build agent");
            let agent_list = AgentListBuilder::new()
                .with_agents(vec![agent])
                .build()
                .expect("Unable to build agent list");
            let agent_bytes = agent_list
                .into_bytes()
                .expect("Unable to convert agent list to bytes");
            let agent_address = compute_agent_address(DRAFT_PUB_KEY);
            self.set_state_entry(agent_address, agent_bytes)
                .expect("Unable to set agent in state");
        }

        fn add_editor_role(&self) {
            let editor_role = RoleBuilder::new()
                .with_org_id(ORG_ID_1.to_string())
                .with_name(ROLE_EDITOR.to_string())
                .with_description("Purchase Order editor role".to_string())
                .with_permissions(vec![PERM_ALIAS_EDITOR.to_string()])
                .build()
                .expect("Unable to build role");
            let role_list = RoleListBuilder::new()
                .with_roles(vec![editor_role])
                .build()
                .expect("Unable to build role list");
            let role_bytes = role_list
                .into_bytes()
                .expect("Unable to convert role list to bytes");
            let role_address = compute_role_address(ROLE_EDITOR, ORG_ID_1);
            self.set_state_entry(role_address, role_bytes)
                .expect("Unable to set role in state");
        }

        fn add_editor_agent(&self) {
            let agent = AgentBuilder::new()
                .with_org_id(ORG_ID_1.to_string())
                .with_public_key(EDITOR_PUB_KEY.to_string())
                .with_active(true)
                .with_roles(vec![ROLE_EDITOR.to_string()])
                .build()
                .expect("Unable to build agent");

            let agent_list = AgentListBuilder::new()
                .with_agents(vec![agent])
                .build()
                .expect("Unable to build agent list");
            let agent_bytes = agent_list
                .into_bytes()
                .expect("Unable to convert agent list to bytes");
            let agent_address = compute_agent_address(EDITOR_PUB_KEY);
            self.set_state_entry(agent_address, agent_bytes)
                .expect("Unable to add agent to state");
        }

        fn add_partner_role(&self) {
            let role = RoleBuilder::new()
                .with_org_id(ORG_ID_1.to_string())
                .with_name(ROLE_PARTNER.to_string())
                .with_description("Purchase Order seller role".to_string())
                .with_permissions(vec![PERM_ALIAS_PARTNER.to_string()])
                .build()
                .expect("Unable to build role");
            let role_list = RoleListBuilder::new()
                .with_roles(vec![role])
                .build()
                .expect("Unable to build role list");
            let role_bytes = role_list.into_bytes().unwrap();
            let role_address = compute_role_address(ROLE_PARTNER, ORG_ID_1);
            self.set_state_entry(role_address, role_bytes)
                .expect("Unable to add role to state");
        }

        fn add_partner_agent(&self) {
            let agent = AgentBuilder::new()
                .with_org_id(ORG_ID_1.to_string())
                .with_public_key(PARTNER_PUB_KEY.to_string())
                .with_active(true)
                .with_roles(vec![ROLE_PARTNER.to_string()])
                .build()
                .expect("Unable to build agent");
            let agent_list = AgentListBuilder::new()
                .with_agents(vec![agent])
                .build()
                .expect("Unable to build agent list");
            let agent_bytes = agent_list
                .into_bytes()
                .expect("Unable to convert agent list to bytes");
            let agent_address = compute_agent_address(PARTNER_PUB_KEY);
            self.set_state_entry(agent_address, agent_bytes)
                .expect("Unable to set agent in state");
        }

        fn add_org(&self, org_id: &str) {
            let org = OrganizationBuilder::new()
                .with_org_id(org_id.to_string())
                .with_name(format!("Organization {}", org_id))
                .build()
                .expect("Unable to build organization");
            let list = OrganizationListBuilder::new()
                .with_organizations(vec![org])
                .build()
                .expect("Unable to build organization list");
            let org_bytes = list
                .into_bytes()
                .expect("Unable to convert organization list to bytes");
            let org_address = compute_organization_address(org_id);
            self.set_state_entry(org_address, org_bytes)
                .expect("Unable to add organization to state");
        }

        fn add_purchase_order(&self, po: PurchaseOrder) {
            let po_uid = po.uid().to_string();
            let list = PurchaseOrderListBuilder::new()
                .with_purchase_orders(vec![po])
                .build()
                .expect("Unable to build purchase order list");
            let po_bytes = list
                .into_bytes()
                .expect("Unable to convert purchase order list to bytes");
            let po_address = compute_purchase_order_address(&po_uid);
            self.set_state_entry(po_address, po_bytes)
                .expect("Unable to add purchase order to state");
        }
    }

    #[test]
    fn test_create_po_already_exists() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order());
        let create_po_payload = CreatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_created_at(1)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_state("proposed".to_string())
            .with_workflow_id(POWorkflow::SystemOfRecord.to_string())
            .build()
            .expect("Unable to build CreatePurchaseOrderPayload");

        if let Ok(()) =
            create_purchase_order(&create_po_payload, BUYER_PUB_KEY, &mut state, &perm_checker)
        {
            panic!(
                "New purchase order should be invalid because one with the same ID already exists"
            )
        }
    }

    #[test]
    fn test_create_po_org_does_not_exist() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order());
        ctx.add_org(ORG_ID_1);
        ctx.add_buyer_agent();
        let create_po_payload = CreatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_created_at(1)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_state("proposed".to_string())
            .with_workflow_id(POWorkflow::SystemOfRecord.to_string())
            .build()
            .expect("Unable to build CreatePurchaseOrderPayload");

        if let Ok(()) =
            create_purchase_order(&create_po_payload, BUYER_PUB_KEY, &mut state, &perm_checker)
        {
            panic!("New purchase order should be invalid because an organization does not exist")
        }
    }

    #[test]
    fn test_create_po_agent_does_not_exist() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order());
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        let create_po_payload = CreatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_created_at(1)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_state("proposed".to_string())
            .with_workflow_id(POWorkflow::SystemOfRecord.to_string())
            .build()
            .expect("Unable to build CreatePurchaseOrderPayload");

        if let Ok(()) =
            create_purchase_order(&create_po_payload, BUYER_PUB_KEY, &mut state, &perm_checker)
        {
            panic!("New purchase order should be invalid because submitter agent does not exist")
        }
    }

    #[test]
    fn test_create_po_invalid_agent_role() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order());
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_seller_role();
        ctx.add_seller_agent();
        let create_po_payload = CreatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_created_at(1)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_state("proposed".to_string())
            .with_workflow_id(POWorkflow::SystemOfRecord.to_string())
            .build()
            .expect("Unable to build CreatePurchaseOrderPayload");

        if let Ok(()) = create_purchase_order(
            &create_po_payload,
            SELLER_PUB_KEY,
            &mut state,
            &perm_checker,
        ) {
            panic!("Should be invalid because agent does not have permission to create po")
        }
    }

    #[test]
    fn test_create_po_valid() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_buyer_role();
        ctx.add_buyer_agent();
        let create_po_payload = CreatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_created_at(1)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_state("issued".to_string())
            .with_workflow_id(POWorkflow::SystemOfRecord.to_string())
            .build()
            .expect("Unable to build CreatePurchaseOrderPayload");
        if let Err(err) =
            create_purchase_order(&create_po_payload, BUYER_PUB_KEY, &mut state, &perm_checker)
        {
            panic!("Should be valid: {}", err)
        }
    }

    #[test]
    /// Validates a purchase order may not be created and put into a workflow state with the
    /// `accepted` constraint on creation. A purchase order may not be opened in a workflow state
    /// with the `accepted` constraint as no versions have been accepted yet.
    ///
    /// 1. Create the necessary organizations and create an agent with the "buyer" role
    /// 2. Build a `CreatePurchaseOrderPayload` with a `workflow_state` of `confirmed`
    /// 3. Assert the `create_version` function returns an error
    fn test_create_po_invalid_workflow_state() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_buyer_role();
        ctx.add_buyer_agent();
        let create_po_payload = CreatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_created_at(1)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_state("confirmed".to_string())
            .with_workflow_id(POWorkflow::SystemOfRecord.to_string())
            .build()
            .expect("Unable to build CreatePurchaseOrderPayload");
        let expected = format!(
            "Agent {} does not have permission can-create-po for \
        organization {} to create a purchase order with state confirmed",
            BUYER_PUB_KEY, ORG_ID_1
        );
        match create_purchase_order(&create_po_payload, BUYER_PUB_KEY, &mut state, &perm_checker) {
            Err(ApplyError::InvalidTransaction(ref value)) if value == &expected => (),
            value => {
                panic!(
                    "Got {:?} expected ApplyError::InvalidTransaction({:?})",
                    value, expected
                )
            }
        }
    }

    #[test]
    // Test that the update po action fails if the uid does not exist
    fn test_update_po_does_not_exist() {
        let ctx = MockTransactionContext::default();
        ctx.add_buyer_agent();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        let update = UpdatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_is_closed(false)
            .with_workflow_state("proposed".to_string())
            .build()
            .expect("Unable to build UpdatePurchaseOrderPayload");

        let expected = format!("No purchase order exists: {}", PO_UID);
        match update_purchase_order(&update, BUYER_PUB_KEY, &mut state, &perm_checker) {
            Err(ApplyError::InvalidTransaction(ref value)) if value == &expected => (),
            value => {
                panic!(
                    "Got {:?} expected ApplyError::InvalidTransaction({:?})",
                    value, expected
                )
            }
        }
    }

    #[test]
    // Test that the update po action fails if the agent does not exist
    fn test_update_po_agent_does_not_exist() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order());
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        let update = UpdatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_is_closed(false)
            .with_workflow_state("proposed".to_string())
            .build()
            .expect("Unable to build UpdatePurchaseOrderPayload");

        let expected = format!("The signer is not an agent: {}", BUYER_PUB_KEY);
        match update_purchase_order(&update, BUYER_PUB_KEY, &mut state, &perm_checker) {
            Err(ApplyError::InvalidTransaction(ref value)) if value == &expected => (),
            value => {
                panic!(
                    "Got {:?} expected ApplyError::InvalidTransaction({:?})",
                    value, expected
                )
            }
        }
    }

    #[test]
    // Test that the update po action fails if the agent does not have the correct role
    fn test_update_po_invalid_agent_role() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order());
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        // Do not add the role
        //ctx.add_seller_role();
        ctx.add_seller_agent();
        let update = UpdatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_is_closed(false)
            .with_workflow_state("closed".to_string())
            .build()
            .expect("Unable to build UpdatePurchaseOrderPayload");

        let expected = "Agent seller_agent_pub_key does not have the \
            correct permissions to update purchase order test_po_1 from a state of issued \
            to closed";
        match update_purchase_order(&update, SELLER_PUB_KEY, &mut state, &perm_checker) {
            Err(ApplyError::InvalidTransaction(ref value)) if value == expected => (),
            value => {
                panic!(
                    "Got {:?} expected ApplyError::InvalidTransaction({:?})",
                    value, expected
                )
            }
        }
    }

    #[test]
    // Test that the update po closed state will fail when there is an incorrect workflow state
    fn test_update_po_closed_state_fails_incorrect_state() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_buyer_agent();
        ctx.add_buyer_role();
        ctx.add_purchase_order(purchase_order_confirmed(
            vec![purchase_order_version_accepted(PO_VERSION_ID_1)],
            Some(PO_VERSION_ID_1.to_string()),
        ));

        let to_workflow = "issued";
        let update = UpdatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_is_closed(true)
            .with_workflow_state(to_workflow.to_string())
            .build()
            .expect("Unable to build UpdatePurchaseOrderPayload");

        let expected = format!(
            "Workflow state `{}` does not have `closed` constraint, but purchase order {} \
            is closed",
            to_workflow, PO_UID,
        );
        match update_purchase_order(&update, BUYER_PUB_KEY, &mut state, &perm_checker) {
            Err(ApplyError::InvalidTransaction(ref value)) if value == &expected => (),
            value => {
                panic!(
                    "Got {:?} expected ApplyError::InvalidTransaction({:?})",
                    value, expected
                )
            }
        }
    }

    #[test]
    // Test that the update po closed workflow will fail when there is an incorrect po state
    fn test_update_po_closed_workflow_fails_incorrect_state() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_buyer_agent();
        ctx.add_buyer_role();
        ctx.add_purchase_order(purchase_order());

        let to_workflow = "closed";
        let update = UpdatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_is_closed(false)
            .with_workflow_state(to_workflow.to_string())
            .build()
            .expect("Unable to build UpdatePurchaseOrderPayload");

        let expected = format!(
            "The desired workflow state {} for purchase order {} has the `closed` constraint, \
            but property `is_closed` was set to false.",
            to_workflow, PO_UID
        );
        match update_purchase_order(&update, BUYER_PUB_KEY, &mut state, &perm_checker) {
            Err(ApplyError::InvalidTransaction(ref value)) if value == &expected => (),
            value => {
                panic!(
                    "Got {:?} expected ApplyError::InvalidTransaction({:?})",
                    value, expected
                )
            }
        }
    }

    #[test]
    // Test that the update po closed state will fail when there's an accepted version number
    fn test_update_po_closed_state_fails_with_accepted_version_number() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_seller_agent();
        ctx.add_seller_role();
        ctx.add_purchase_order(purchase_order());

        let update = UpdatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_is_closed(true)
            .with_accepted_version_number(Some(PO_VERSION_ID_1.to_string()))
            .with_workflow_state("confirmed".to_string())
            .build()
            .expect("Unable to build UpdatePurchaseOrderPayload");

        let expected = format!(
            "Accepted version number {} set for closed purchase order {}. \
                Expected accepted version number to be empty",
            PO_VERSION_ID_1, PO_UID
        );
        match update_purchase_order(&update, SELLER_PUB_KEY, &mut state, &perm_checker) {
            Err(ApplyError::InvalidTransaction(ref value)) if value == &expected => (),
            value => {
                panic!(
                    "Got {:?} expected ApplyError::InvalidTransaction({:?})",
                    value, expected
                )
            }
        }
    }

    #[test]
    // Test that the update po closed state succeeds
    fn test_update_po_closed_state_succeeds() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_buyer_role();
        ctx.add_buyer_agent();
        ctx.add_purchase_order(purchase_order());

        let update = UpdatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_is_closed(true)
            .with_workflow_state("closed".to_string())
            .build()
            .expect("Unable to build UpdatePurchaseOrderPayload");

        if let Err(err) = update_purchase_order(&update, BUYER_PUB_KEY, &mut state, &perm_checker) {
            panic!("Should be valid: {}", err)
        }
    }

    #[test]
    // Test that the update po checks the existence of accepted versions
    fn test_update_po_accepted_version_number_checks_existence() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_seller_agent();
        ctx.add_seller_role();
        ctx.add_purchase_order(purchase_order());

        let update = UpdatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_is_closed(false)
            .with_accepted_version_number(Some(PO_VERSION_ID_2.to_string()))
            .with_workflow_state("confirmed".to_string())
            .build()
            .expect("Unable to build UpdatePurchaseOrderPayload");
        let expected = format!(
            "Cannot accept purchase order {} version {} that does not exist",
            PO_UID, PO_VERSION_ID_2,
        );
        match update_purchase_order(&update, SELLER_PUB_KEY, &mut state, &perm_checker) {
            Err(ApplyError::InvalidTransaction(ref value)) if value == &expected => (),
            value => {
                panic!(
                    "Got {:?} expected ApplyError::InvalidTransaction({:?})",
                    value, expected
                )
            }
        }
    }

    #[test]
    // Test that the update po checks the state of accepted versions
    fn test_update_po_accepted_version_number_validates_version_workflow_accepted() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_seller_agent();
        ctx.add_seller_role();
        let expected_version = purchase_order_version_draft(PO_VERSION_ID_1);
        ctx.add_purchase_order(purchase_order_with_versions(vec![expected_version]));

        let update = UpdatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_is_closed(false)
            .with_accepted_version_number(Some(PO_VERSION_ID_1.to_string()))
            .with_workflow_state("confirmed".to_string())
            .build()
            .expect("Unable to build UpdatePurchaseOrderPayload");
        let expected_version = purchase_order_version_draft(PO_VERSION_ID_1);
        let expected = format!(
            "Workflow state `{}` set for purchase order {} version {}, \
            cannot accept draft version",
            expected_version.workflow_state(),
            PO_UID,
            PO_VERSION_ID_1,
        );
        match update_purchase_order(&update, SELLER_PUB_KEY, &mut state, &perm_checker) {
            Err(ApplyError::InvalidTransaction(ref value)) if value == &expected => (),
            value => {
                panic!(
                    "Got {:?} expected ApplyError::InvalidTransaction({:?})",
                    value, expected
                )
            }
        }
    }

    #[test]
    // Test that the update po checks the state of the po when accepted versions are set
    fn test_update_po_accepted_version_number_validates_po_workflow_accepted() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_buyer_agent();
        ctx.add_buyer_role();
        ctx.add_purchase_order(purchase_order_confirmed(
            vec![purchase_order_version_accepted(PO_VERSION_ID_1)],
            Some(PO_VERSION_ID_1.to_string()),
        ));

        let to_workflow = "issued";
        let update = UpdatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_is_closed(false)
            .with_workflow_state(to_workflow.to_string())
            .with_accepted_version_number(Some(PO_VERSION_ID_1.to_string()))
            .build()
            .expect("Unable to build UpdatePurchaseOrderPayload");

        let expected = format!(
            "Purchase order {} has accepted version {}, but po workflow state `{}` does not have \
            an `accepted` costraint",
            PO_UID, PO_VERSION_ID_1, to_workflow,
        );
        match update_purchase_order(&update, BUYER_PUB_KEY, &mut state, &perm_checker) {
            Err(ApplyError::InvalidTransaction(ref value)) if value == &expected => (),
            value => {
                panic!(
                    "Got {:?} expected ApplyError::InvalidTransaction({:?})",
                    value, expected
                )
            }
        }
    }

    #[test]
    // Test that the update po complete that is closed fails
    fn test_update_po_complete_closed_fails() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_seller_agent();
        ctx.add_seller_role();
        ctx.add_purchase_order(purchase_order());

        let to_workflow = "confirmed";
        let update = UpdatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_is_closed(true)
            .with_workflow_state(to_workflow.to_string())
            .build()
            .expect("Unable to build UpdatePurchaseOrderPayload");

        let expected = format!(
            "Workflow state `{}` set for closed purchase order {}. Expected workflow \
                state not to be closed for a complete purchase order",
            to_workflow, PO_UID
        );
        match update_purchase_order(&update, SELLER_PUB_KEY, &mut state, &perm_checker) {
            Err(ApplyError::InvalidTransaction(ref value)) if value == &expected => (),
            value => {
                panic!(
                    "Got {:?} expected ApplyError::InvalidTransaction({:?})",
                    value, expected
                )
            }
        }
    }

    #[test]
    // Test that the update po accepted without accepted version number fails
    fn test_update_po_accepted_without_accepted_version_number_fails() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_seller_role();
        ctx.add_seller_agent();
        ctx.add_purchase_order(purchase_order());
        let to_workflow = "confirmed";
        let update = UpdatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_is_closed(false)
            .with_workflow_state(to_workflow.to_string())
            .build()
            .expect("Unable to build UpdatePurchaseOrderPayload");

        let expected = format!(
            "Workflow state `{}` set for purchase order {} has `accepted` constraint, \
            but no version is accepted",
            to_workflow, PO_UID
        );
        match update_purchase_order(&update, SELLER_PUB_KEY, &mut state, &perm_checker) {
            Err(ApplyError::InvalidTransaction(ref value)) if value == &expected => (),
            value => {
                panic!(
                    "Got {:?} expected ApplyError::InvalidTransaction({:?})",
                    value, expected
                )
            }
        }
    }

    #[test]
    // Test that the update po works when everything is set
    fn test_update_po_valid() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_seller_agent();
        ctx.add_seller_role();
        ctx.add_purchase_order(purchase_order_with_versions(vec![
            purchase_order_version_accepted(PO_VERSION_ID_1),
        ]));

        let update = UpdatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_is_closed(false)
            .with_accepted_version_number(Some(PO_VERSION_ID_1.to_string()))
            .with_workflow_state("confirmed".to_string())
            .build()
            .expect("Unable to build UpdatePurchaseOrderPayload");

        if let Err(err) = update_purchase_order(&update, SELLER_PUB_KEY, &mut state, &perm_checker)
        {
            panic!("Should be valid: {}", err)
        }
    }

    #[test]
    /// Validates the `create_version` function returns an error in the case that the submitting
    /// agent does not have the correct permissions to create a purchase order version.
    /// The test follows these steps:
    ///
    /// 1. Create the necessary organizations and create an agent with the "draft" role
    /// 2. Add a Purchase Order to state with versions (This will issue the purchase order
    ///    within the System of Record version subworkflow)
    /// 3. Build a `CreateVersionPayload` with an `is_draft` field of `false` and a
    ///    `workflow_state` of `proposed`
    /// 4. Assert the `create_version` function returns an error
    ///
    /// This test validates an agent is unable to create a "proposed" purchase order version with
    /// the "draft" workflow permission.
    fn test_create_po_vers_invalid_agent_perms() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order());
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_draft_role();
        ctx.add_drafting_agent();
        let payload_revision = PayloadRevisionBuilder::new()
            .with_revision_id(1)
            .with_submitter(BUYER_PUB_KEY.to_string())
            .with_created_at(1)
            .with_order_xml_v3_4("xml_v3_4_string".to_string())
            .build()
            .expect("Unable to build payload revision");
        let create_po_vers_payload = CreateVersionPayloadBuilder::new()
            .with_version_id(PO_VERSION_ID_1.to_string())
            .with_po_uid(PO_UID.to_string())
            .with_is_draft(true)
            .with_workflow_state("editable".to_string())
            .with_revision(payload_revision)
            .build()
            .expect("Unable to build CreateVersionPayload");

        if let Ok(()) = create_version(
            &create_po_vers_payload,
            BUYER_PUB_KEY,
            &mut state,
            &perm_checker,
        ) {
            panic!(
                "New purchase order version should be invalid because one with the same ID \
                already exists"
            )
        }
    }

    #[test]
    /// Validates the `create_version` function returns an error in the case that a purchase order
    /// version with the same version ID already exists in state. The test follows these steps:
    ///
    /// 1. Create the necessary organizations and create an agent with the "draft" role
    /// 2. Add a Purchase Order to state with versions (This will issue the purchase order
    ///    within the System of Record version subworkflow)
    /// 3. Build a `CreateVersionPayload` with an `is_draft` field of `true` and a `workflow_state`
    ///    of `editable`
    /// 4. Assert the `create_version` function returns an error
    ///
    /// This test validates a purchase order version is unable to be created if a version already
    /// exists in state with the same version ID.
    fn test_create_po_vers_already_exists() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order());
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_draft_role();
        ctx.add_drafting_agent();
        let payload_revision = PayloadRevisionBuilder::new()
            .with_revision_id(1)
            .with_submitter(BUYER_PUB_KEY.to_string())
            .with_created_at(1)
            .with_order_xml_v3_4("xml_v3_4_string".to_string())
            .build()
            .expect("Unable to build payload revision");
        let create_po_vers_payload = CreateVersionPayloadBuilder::new()
            .with_version_id(PO_VERSION_ID_1.to_string())
            .with_po_uid(PO_UID.to_string())
            .with_is_draft(true)
            .with_workflow_state("editable".to_string())
            .with_revision(payload_revision)
            .build()
            .expect("Unable to build CreateVersionPayload");

        if let Ok(()) = create_version(
            &create_po_vers_payload,
            BUYER_PUB_KEY,
            &mut state,
            &perm_checker,
        ) {
            panic!(
                "New purchase order version should be invalid because one with the same ID \
                already exists"
            )
        }
    }

    #[test]
    /// Validates the `create_version` function returns an error in the case that the
    /// payload's `workflow_state` does not match the purchase order version's state within a
    /// `Collaborative` version subworkflow. The test follows these steps:
    ///
    /// 1. Create the necessary organizations and create an agent with the "partner" role
    /// 2. Add a Purchase Order to state without versions (This will issue the purchase order
    ///    within the `Collaborative` version subworkflow)
    /// 3. Build a `CreateVersionPayload` with an `is_draft` field of `true` and a `workflow_state`
    ///    of `modified`
    /// 4. Assert the `create_version` function returns an error
    ///
    /// This test validates a purchase order version is unable to be created with a
    /// `workflow_state` of `modified` within the `Collaborative` version subworkflow
    /// although the agent had the correct workflow permission to create a version, "partner".
    /// Purchase order versions must first be `proposed` before they are able to be `modified`
    /// within both the Collaborative and System of Record subworkflows.
    fn test_col_create_po_vers_invalid_transition() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_partner_role();
        ctx.add_partner_agent();
        ctx.add_purchase_order(purchase_order_wo_versions());
        let payload_revision = PayloadRevisionBuilder::new()
            .with_revision_id(1)
            .with_submitter(PARTNER_PUB_KEY.to_string())
            .with_created_at(1)
            .with_order_xml_v3_4("xml_v3_4_string".to_string())
            .build()
            .expect("Unable to build payload revision");
        let create_po_vers_payload = CreateVersionPayloadBuilder::new()
            .with_version_id(PO_VERSION_ID_1.to_string())
            .with_po_uid(PO_UID.to_string())
            .with_is_draft(true)
            .with_workflow_state("modified".to_string())
            .with_revision(payload_revision)
            .build()
            .expect("Unable to build CreateVersionPayload");

        if let Ok(()) = create_version(
            &create_po_vers_payload,
            PARTNER_PUB_KEY,
            &mut state,
            &perm_checker,
        ) {
            panic!(
                "New purchase order version should be invalid because \
                the desired workflow state is invalid"
            )
        }
    }

    #[test]
    /// Validates the `create_version` function returns successfully when given a valid payload.
    /// Specifically, this test creates a scenario where the purchase order version is created
    /// within the Collaborative subworkflow and is successfully transitioned to the "proposed"
    /// workflow state. The test follows these steps:
    ///
    /// 1. Create the necessary organizations and create an agent with the "partner" role
    /// 2. Add a Purchase Order to state without versions (This will issue the purchase order
    ///    within the Collaborative version subworkflow)
    /// 3. Build a `CreateVersionPayload` with an `is_draft` field of `false` and a
    ///    `workflow_state` of `proposed`
    /// 4. Assert the `create_version` function returns succesfully
    ///
    /// This test validates a purchase order version is able to be created with a
    /// `workflow_state` of `proposed` within the `Collaborative` version subworkflow
    /// as the agent had the correct workflow permission to create a version, "partner".
    fn test_col_create_po_vers_valid() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_partner_role();
        ctx.add_partner_agent();
        ctx.add_purchase_order(purchase_order_wo_versions());
        let payload_revision = PayloadRevisionBuilder::new()
            .with_revision_id(1)
            .with_submitter(PARTNER_PUB_KEY.to_string())
            .with_created_at(1)
            .with_order_xml_v3_4("xml_v3_4_string".to_string())
            .build()
            .expect("Unable to build payload revision");
        let create_po_vers_payload = CreateVersionPayloadBuilder::new()
            .with_version_id(PO_VERSION_ID_1.to_string())
            .with_po_uid(PO_UID.to_string())
            .with_is_draft(false)
            .with_workflow_state("proposed".to_string())
            .with_revision(payload_revision)
            .build()
            .expect("Unable to build CreateVersionPayload");

        if let Err(err) = create_version(
            &create_po_vers_payload,
            PARTNER_PUB_KEY,
            &mut state,
            &perm_checker,
        ) {
            panic!("New purchase order version should be valid: {}", err,)
        }
    }

    #[test]
    /// Validates the `create_version` function returns succesfully with a valid payload. This test
    /// specifically tests the scenario where a purchase order version is created not as a draft
    /// and moved into the "proposed" workflow state within the System of Record version
    /// subworkflow. The test follows these steps:
    ///
    /// 1. Add the buyer and seller organizations to state
    /// 2. Create an agent with the "buyer" role
    /// 3. Add a Purchase Order to state with versions (This will issue the purchase order
    ///    within the System of Record version subworkflow)
    /// 4. Build a `CreateVersionPayload` with an `is_draft` field of `false` and a `workflow_state`
    ///    of `proposed`
    /// 5. Assert the `create_version` function returns successfully
    ///
    /// This test validates a purchase order version is able to be created with a
    /// `workflow_state` of `proposed` within the System of Record version subworkflow as the
    /// agent has the "buyer" role, which enables the agent to create the version and transition
    /// it to the "proposed" workflow state.
    fn test_sys_create_po_vers_valid_transition_proposed() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order());
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_buyer_role();
        ctx.add_buyer_agent();
        let payload_revision = PayloadRevisionBuilder::new()
            .with_revision_id(1)
            .with_submitter(BUYER_PUB_KEY.to_string())
            .with_created_at(1)
            .with_order_xml_v3_4("xml_v3_4_string".to_string())
            .build()
            .expect("Unable to build payload revision");
        let create_po_vers_payload = CreateVersionPayloadBuilder::new()
            .with_version_id(PO_VERSION_ID_2.to_string())
            .with_po_uid(PO_UID.to_string())
            .with_is_draft(false)
            .with_workflow_state("proposed".to_string())
            .with_revision(payload_revision)
            .build()
            .expect("Unable to build CreateVersionPayload");

        if let Err(err) = create_version(
            &create_po_vers_payload,
            BUYER_PUB_KEY,
            &mut state,
            &perm_checker,
        ) {
            panic!("New purchase order version should be valid: {}", err)
        }
    }

    #[test]
    /// Validates the `create_version` function returns succesfully with a valid payload. This test
    /// specifically tests the scenario where a purchase order version is created as a draft and
    /// moved into the "editable" workflow state within the System of Record version subworkflow.
    /// The test follows these steps:
    ///
    /// 1. Add the buyer and seller organizations to state
    /// 2. Create an agent with the "draft" role
    /// 3. Add a Purchase Order to state with versions (This will issue the purchase order
    ///    within the System of Record version subworkflow)
    /// 4. Build a `CreateVersionPayload` with an `is_draft` field of `true` and a `workflow_state`
    ///    of `editable`
    /// 5. Assert the `create_version` function returns successfully
    ///
    /// This test validates a purchase order version is able to be created with a
    /// `workflow_state` of `editable` within the System of Record version subworkflow as the
    /// agent has the "draft" role, which enables the agent to create the version and transition
    /// it to the "editable" workflow state.
    fn test_sys_create_po_vers_valid_transition_editable() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order());
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_draft_role();
        ctx.add_drafting_agent();
        let payload_revision = PayloadRevisionBuilder::new()
            .with_revision_id(1)
            .with_submitter(DRAFT_PUB_KEY.to_string())
            .with_created_at(1)
            .with_order_xml_v3_4("xml_v3_4_string".to_string())
            .build()
            .expect("Unable to build payload revision");
        let create_po_vers_payload = CreateVersionPayloadBuilder::new()
            .with_version_id(PO_VERSION_ID_2.to_string())
            .with_po_uid(PO_UID.to_string())
            .with_is_draft(true)
            .with_workflow_state("editable".to_string())
            .with_revision(payload_revision)
            .build()
            .expect("Unable to build CreateVersionPayload");

        if let Err(err) = create_version(
            &create_po_vers_payload,
            DRAFT_PUB_KEY,
            &mut state,
            &perm_checker,
        ) {
            panic!("New purchase order version should be valid: {}", err)
        }
    }

    #[test]
    /// This test validates a draft purchase order version is not able to be created with a
    /// `workflow_state` of `accepted` within the System of Record version subworkflow.
    /// An accepted purchase order version may not be a draft. The test follows these steps:
    ///
    /// 1. Add the buyer and seller organizations to state
    /// 2. Create an agent with the "buyer" role
    /// 3. Add a Purchase Order to state with versions (This will issue the purchase order
    ///    within the System of Record version subworkflow)
    /// 4. Build a `CreateVersionPayload` with an `is_draft` field of `true` and a `workflow_state`
    ///    of `accepted`
    /// 5. Assert the `create_version` function returns an error
    fn test_sys_create_po_vers_invalid_transition_accepted() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order());
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_buyer_role();
        ctx.add_buyer_agent();
        let payload_revision = PayloadRevisionBuilder::new()
            .with_revision_id(1)
            .with_submitter(BUYER_PUB_KEY.to_string())
            .with_created_at(1)
            .with_order_xml_v3_4("xml_v3_4_string".to_string())
            .build()
            .expect("Unable to build payload revision");
        let create_po_vers_payload = CreateVersionPayloadBuilder::new()
            .with_version_id(PO_VERSION_ID_2.to_string())
            .with_po_uid(PO_UID.to_string())
            .with_is_draft(false)
            .with_workflow_state("accepted".to_string())
            .with_revision(payload_revision)
            .build()
            .expect("Unable to build CreateVersionPayload");
        let expected = format!(
            "Agent {} does not have the correct permissions for organization {} \
            to create purchase order version {} in the {} workflow state",
            BUYER_PUB_KEY, ORG_ID_1, PO_VERSION_ID_2, "accepted"
        );

        match create_version(
            &create_po_vers_payload,
            BUYER_PUB_KEY,
            &mut state,
            &perm_checker,
        ) {
            Err(ApplyError::InvalidTransaction(ref value)) if value == &expected => (),
            value => {
                panic!(
                    "Got `{:?}`, expected ApplyError with message `{:?}`",
                    value, expected
                )
            }
        }
    }

    #[test]
    /// This test validates a draft purchase order version is able to be updated with a new
    /// revision.
    ///
    /// 1. Add the buyer and seller organizations to state
    /// 2. Create an agent with the "draft" role
    /// 3. Add a Purchase Order to state with a draft version (This will issue the purchase order
    ///    within the System of Record version subworkflow)
    /// 4. Build an `UpdateVersionPayload` with a new revision for the existing version, v01
    /// 5. Assert the `update_version` function returns successfully
    fn test_sys_update_po_vers_draft_valid() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order_with_versions(vec![
            purchase_order_version_draft(PO_VERSION_ID_1),
        ]));
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_draft_role();
        ctx.add_drafting_agent();
        let payload_revision = PayloadRevisionBuilder::new()
            .with_revision_id(2)
            .with_submitter(DRAFT_PUB_KEY.to_string())
            .with_created_at(3)
            .with_order_xml_v3_4("xml_v3_4_string".to_string())
            .build()
            .expect("Unable to build payload revision");
        let update_vers_payload = UpdateVersionPayloadBuilder::new()
            .with_version_id(PO_VERSION_ID_1.to_string())
            .with_po_uid(PO_UID.to_string())
            .with_workflow_state("editable".to_string())
            .with_is_draft(true)
            .with_revision(payload_revision)
            .build()
            .expect("Unable to build UpdateVersionPayload");
        if let Err(err) = update_version(
            &update_vers_payload,
            DRAFT_PUB_KEY,
            &mut state,
            &perm_checker,
        ) {
            panic!(
                "Update to Purchase Order Version should be valid: {:?}",
                err
            )
        }
    }

    #[test]
    /// This test validates a draft purchase order version is able to be updated with a new
    /// revision.
    ///
    /// 1. Add the buyer and seller organizations to state
    /// 2. Create an agent with the "buyer" role
    /// 3. Add a Purchase Order to state with a version (This will issue the purchase order
    ///    within the System of Record version subworkflow)
    /// 4. Build an `UpdateVersionPayload` with a new revision for the existing version, v01
    /// 5. Assert the `update_version` function returns successfully
    fn test_sys_update_po_vers_valid() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order());
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_buyer_role();
        ctx.add_buyer_agent();
        let payload_revision = PayloadRevisionBuilder::new()
            .with_revision_id(2)
            .with_submitter(BUYER_PUB_KEY.to_string())
            .with_created_at(3)
            .with_order_xml_v3_4("xml_v3_4_string".to_string())
            .build()
            .expect("Unable to build payload revision");
        let update_vers_payload = UpdateVersionPayloadBuilder::new()
            .with_version_id(PO_VERSION_ID_1.to_string())
            .with_po_uid(PO_UID.to_string())
            .with_workflow_state("proposed".to_string())
            .with_is_draft(false)
            .with_revision(payload_revision)
            .build()
            .expect("Unable to build UpdateVersionPayload");
        if let Err(err) = update_version(
            &update_vers_payload,
            BUYER_PUB_KEY,
            &mut state,
            &perm_checker,
        ) {
            panic!(
                "Update to Purchase Order Version should be valid: {:?}",
                err
            )
        }
    }

    #[test]
    /// This test validates a purchase order version is able to be updated with a new revision
    /// and then the version is able to be transitioned to a `draft` version.
    ///
    /// 1. Add the buyer and seller organizations to state
    /// 2. Create an agent with the "seller" role
    /// 3. Add a Purchase Order to state with a version (This will issue the purchase order
    ///    within the System of Record version subworkflow)
    /// 4. Build an `UpdateVersionPayload` with a new revision for the existing version, v01
    /// 5. Assert the `update_version` function returns successfully
    /// 6. Create an agent with the "editor" role
    /// 7. Build an `UpdateVersionPayload` for the v01, with an `is_draft` field of true and the
    ///    proposed workflow state, "editable"
    /// 8. Assert the `update_version` function returns successfully
    fn test_sys_update_po_vers_transition_modified() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order());
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_seller_role();
        ctx.add_seller_agent();
        let payload_revision = PayloadRevisionBuilder::new()
            .with_revision_id(2)
            .with_submitter(SELLER_PUB_KEY.to_string())
            .with_created_at(3)
            .with_order_xml_v3_4("xml_v3_4_string".to_string())
            .build()
            .expect("Unable to build payload revision");
        let update_vers_payload = UpdateVersionPayloadBuilder::new()
            .with_version_id(PO_VERSION_ID_1.to_string())
            .with_po_uid(PO_UID.to_string())
            .with_workflow_state("modified".to_string())
            .with_is_draft(false)
            .with_revision(payload_revision)
            .build()
            .expect("Unable to build UpdateVersionPayload");
        if let Err(err) = update_version(
            &update_vers_payload,
            SELLER_PUB_KEY,
            &mut state,
            &perm_checker,
        ) {
            panic!(
                "Update to Purchase Order Version should be valid: {:?}",
                err
            )
        }
        // Add the editing agent who will transition our version to a draft version
        ctx.add_editor_role();
        ctx.add_editor_agent();
        let payload_revision = PayloadRevisionBuilder::new()
            .with_revision_id(2)
            .with_submitter(SELLER_PUB_KEY.to_string())
            .with_created_at(3)
            .with_order_xml_v3_4("xml_v3_4_string".to_string())
            .build()
            .expect("Unable to build payload revision");
        let update_vers_payload = UpdateVersionPayloadBuilder::new()
            .with_version_id(PO_VERSION_ID_1.to_string())
            .with_po_uid(PO_UID.to_string())
            .with_workflow_state("editable".to_string())
            .with_is_draft(true)
            .with_revision(payload_revision)
            .build()
            .expect("Unable to build UpdateVersionPayload");
        if let Err(err) = update_version(
            &update_vers_payload,
            EDITOR_PUB_KEY,
            &mut state,
            &perm_checker,
        ) {
            panic!(
                "Update to Purchase Order Version should be valid: {:?}",
                err
            )
        }
    }

    #[test]
    /// This test validates a purchase order version is not able to be updated with the incorrect
    /// permissions for the System of Record version workflow. An agent with the `partner` role
    /// carries permissions within the Collaborative version workflow
    ///
    /// 1. Add the buyer and seller organizations to state
    /// 2. Create an agent with the "partner" role
    /// 3. Add a Purchase Order to state with a version (This will issue the purchase order
    ///    within the System of Record version subworkflow)
    /// 4. Build an `UpdateVersionPayload` with a new revision for the existing version, v01
    /// 5. Assert the `update_version` function returns an error
    fn test_sys_update_po_vers_invalid() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order());
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_partner_role();
        ctx.add_partner_agent();
        let payload_revision = PayloadRevisionBuilder::new()
            .with_revision_id(2)
            .with_submitter(PARTNER_PUB_KEY.to_string())
            .with_created_at(3)
            .with_order_xml_v3_4("xml_v3_4_string".to_string())
            .build()
            .expect("Unable to build payload revision");
        let update_vers_payload = UpdateVersionPayloadBuilder::new()
            .with_version_id(PO_VERSION_ID_1.to_string())
            .with_po_uid(PO_UID.to_string())
            .with_workflow_state("proposed".to_string())
            .with_is_draft(false)
            .with_revision(payload_revision)
            .build()
            .expect("Unable to build UpdateVersionPayload");
        let expected = format!(
            "Agent {} does not have the correct permissions to update purchase order version 01 \
            from a state of proposed to proposed",
            PARTNER_PUB_KEY,
        );
        match update_version(
            &update_vers_payload,
            PARTNER_PUB_KEY,
            &mut state,
            &perm_checker,
        ) {
            Err(ApplyError::InvalidTransaction(ref value)) if value == &expected => (),
            value => {
                panic!(
                    "Got {:?} expected ApplyError with message {:?}",
                    value, expected
                )
            }
        }
    }

    #[test]
    /// This test validates a purchase order version may not be updated if the version does not
    /// already exist in state.
    ///
    /// 1. Add the buyer and seller organizations to state
    /// 2. Create an agent with the "buyer" role
    /// 3. Add a Purchase Order to state with no versions (This will issue a Collaborative PO)
    /// 4. Build an `UpdateVersionPayload` with a new revision for the version '01'
    /// 5. Assert the `update_version` function returns an error
    fn test_col_update_po_vers_does_not_exist() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order_wo_versions());
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_buyer_role();
        ctx.add_buyer_agent();
        let payload_revision = PayloadRevisionBuilder::new()
            .with_revision_id(1)
            .with_submitter(BUYER_PUB_KEY.to_string())
            .with_created_at(3)
            .with_order_xml_v3_4("xml_v3_4_string".to_string())
            .build()
            .expect("Unable to build payload revision");
        let update_vers_payload = UpdateVersionPayloadBuilder::new()
            .with_version_id(PO_VERSION_ID_1.to_string())
            .with_po_uid(PO_UID.to_string())
            .with_workflow_state("editable".to_string())
            .with_is_draft(true)
            .with_revision(payload_revision)
            .build()
            .expect("Unable to build UpdateVersionPayload");
        let expected = format!(
            "No version {} exists for purchase order {}",
            PO_VERSION_ID_1, PO_UID
        );
        match update_version(
            &update_vers_payload,
            BUYER_PUB_KEY,
            &mut state,
            &perm_checker,
        ) {
            Err(ApplyError::InvalidTransaction(ref value)) if value == &expected => (),
            value => {
                panic!(
                    "Got {:?} expected ApplyError with message {:?}",
                    value, expected
                )
            }
        }
    }

    #[test]
    /// This test validates a purchase order version is only able to be updated to the desired
    /// workflow state if all constraints are met, specifically if the `draft` constraint exists
    /// on the version's workflow state, the version must be a draft.
    ///
    /// 1. Add the buyer and seller organizations to state
    /// 2. Create an agent with the "draft" role
    /// 3. Add a Purchase Order to state with a draft version (This will issue the purchase order
    ///    within the System of Record version subworkflow)
    /// 4. Build an `UpdateVersionPayload` with a new revision for the existing version, v01 with
    ///    an `is_draft` field of false
    /// 5. Assert the `update_version` function returns an error
    fn test_sys_update_po_vers_draft_invalid() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order_with_versions(vec![
            purchase_order_version_draft(PO_VERSION_ID_1),
        ]));
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_draft_role();
        ctx.add_drafting_agent();
        let payload_revision = PayloadRevisionBuilder::new()
            .with_revision_id(2)
            .with_submitter(DRAFT_PUB_KEY.to_string())
            .with_created_at(3)
            .with_order_xml_v3_4("xml_v3_4_string".to_string())
            .build()
            .expect("Unable to build payload revision");
        let update_vers_payload = UpdateVersionPayloadBuilder::new()
            .with_version_id(PO_VERSION_ID_1.to_string())
            .with_po_uid(PO_UID.to_string())
            .with_workflow_state("editable".to_string())
            .with_is_draft(false)
            .with_revision(payload_revision)
            .build()
            .expect("Unable to build UpdateVersionPayload");
        let expected = format!(
            "Workflow state editable has `draft` constraint, updated version is not a draft"
        );
        match update_version(
            &update_vers_payload,
            DRAFT_PUB_KEY,
            &mut state,
            &perm_checker,
        ) {
            Err(ApplyError::InvalidTransaction(ref value)) if value == &expected => (),
            value => {
                panic!(
                    "Got {:?} expected ApplyError with message {:?}",
                    value, expected
                )
            }
        }
    }

    #[test]
    /// This test validates a purchase order version is only able to be updated to the desired
    /// workflow state if all constraints are met, specifically if the `accepted` constraint exists
    /// on the version's workflow state, the version must not be a draft.
    ///
    /// 1. Add the buyer and seller organizations to state
    /// 2. Create an agent with the "seller" role
    /// 3. Add a Purchase Order to state with a proposed version (This will issue the purchase
    ///    order within the System of Record version subworkflow)
    /// 4. Build an `UpdateVersionPayload` with a no new revision and an `is_draft` field of true.
    /// 5. Assert the `update_version` function returns an error
    fn test_sys_update_po_vers_accepted_invalid() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order_confirmed(
            vec![purchase_order_version(PO_VERSION_ID_1)],
            Some(PO_VERSION_ID_1.to_string()),
        ));
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_seller_role();
        ctx.add_seller_agent();
        let payload_revision = PayloadRevisionBuilder::new()
            .with_revision_id(1)
            .with_submitter(BUYER_PUB_KEY.to_string())
            .with_created_at(1)
            .with_order_xml_v3_4("xml_v3_4_string".to_string())
            .build()
            .expect("Unable to build payload revision");
        let update_vers_payload = UpdateVersionPayloadBuilder::new()
            .with_version_id(PO_VERSION_ID_1.to_string())
            .with_po_uid(PO_UID.to_string())
            .with_workflow_state("accepted".to_string())
            .with_is_draft(true)
            .with_revision(payload_revision)
            .build()
            .expect("Unable to build UpdateVersionPayload");
        let expected = format!(
            "Desired workflow state `{}` has `accepted` constraint, version is a draft",
            update_vers_payload.workflow_state(),
        );
        match update_version(
            &update_vers_payload,
            SELLER_PUB_KEY,
            &mut state,
            &perm_checker,
        ) {
            Err(ApplyError::InvalidTransaction(ref value)) if value == &expected => (),
            value => {
                panic!(
                    "Got {:?} expected ApplyError with message {:?}",
                    value, expected
                )
            }
        }
    }

    fn purchase_order() -> PurchaseOrder {
        PurchaseOrderBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_workflow_state("issued".to_string())
            .with_created_at(1)
            .with_versions(vec![purchase_order_version(PO_VERSION_ID_1)])
            .with_is_closed(false)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_id(POWorkflow::SystemOfRecord.to_string())
            .build()
            .expect("Unable to build purchase order")
    }

    fn purchase_order_with_versions(versions: Vec<PurchaseOrderVersion>) -> PurchaseOrder {
        PurchaseOrderBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_workflow_state("issued".to_string())
            .with_created_at(1)
            .with_versions(versions)
            .with_is_closed(false)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_id(POWorkflow::SystemOfRecord.to_string())
            .build()
            .expect("Unable to build purchase order")
    }

    fn purchase_order_confirmed(
        versions: Vec<PurchaseOrderVersion>,
        accepted_version: Option<String>,
    ) -> PurchaseOrder {
        let mut po_builder = PurchaseOrderBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_workflow_state("confirmed".to_string())
            .with_created_at(1)
            .with_versions(versions)
            .with_is_closed(false)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_id(POWorkflow::SystemOfRecord.to_string());
        if let Some(accepted_vers) = accepted_version {
            po_builder = po_builder.with_accepted_version_number(accepted_vers);
        };
        po_builder.build().expect("Unable to build purchase order")
    }

    fn purchase_order_wo_versions() -> PurchaseOrder {
        PurchaseOrderBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_workflow_state("issued".to_string())
            .with_created_at(1)
            .with_is_closed(false)
            .with_versions(vec![])
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_id(POWorkflow::Collaborative.to_string())
            .build()
            .expect("Unable to build purchase order")
    }

    fn purchase_order_version_draft(version_id: &str) -> PurchaseOrderVersion {
        PurchaseOrderVersionBuilder::new()
            .with_version_id(version_id.to_string())
            .with_workflow_state("editable".to_string())
            .with_is_draft(true)
            .with_current_revision_id(1)
            .with_revisions(purchase_order_revisions())
            .build()
            .expect("Unable to build first purchase order version")
    }

    fn purchase_order_version(version_id: &str) -> PurchaseOrderVersion {
        PurchaseOrderVersionBuilder::new()
            .with_version_id(version_id.to_string())
            .with_workflow_state("proposed".to_string())
            .with_is_draft(false)
            .with_current_revision_id(1)
            .with_revisions(purchase_order_revisions())
            .build()
            .expect("Unable to build first purchase order version")
    }

    fn purchase_order_version_accepted(version_id: &str) -> PurchaseOrderVersion {
        PurchaseOrderVersionBuilder::new()
            .with_version_id(version_id.to_string())
            .with_workflow_state("accepted".to_string())
            .with_is_draft(false)
            .with_current_revision_id(1)
            .with_revisions(purchase_order_revisions())
            .build()
            .expect("Unable to build first purchase order version")
    }

    fn purchase_order_revisions() -> Vec<PurchaseOrderRevision> {
        vec![PurchaseOrderRevisionBuilder::new()
            .with_revision_id(1)
            .with_submitter(BUYER_PUB_KEY.to_string())
            .with_created_at(1)
            .with_order_xml_v3_4("xml_purchase_order".to_string())
            .build()
            .expect("Unable to build purchase order revision")]
    }
}
