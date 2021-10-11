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

use crate::permissions::Permission;
use crate::state::PurchaseOrderState;
use crate::workflow::{get_workflow, POWorkflow};

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
    let buyer_org_id = payload.buyer_org_id().to_string();
    let seller_org_id = payload.seller_org_id().to_string();
    // Check that the organizations owning the purchase order exist
    state.get_organization(&buyer_org_id)?.ok_or_else(|| {
        ApplyError::InvalidTransaction(format!("Organization {} does not exist", &buyer_org_id))
    })?;
    state.get_organization(&seller_org_id)?.ok_or_else(|| {
        ApplyError::InvalidTransaction(format!("Organization {} does not exist", &seller_org_id))
    })?;
    // Validate the signer exists
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
    // Check if the payload contains a `PurchaseOrderVersion`
    let mut workflow = POWorkflow::SystemOfRecord;
    let versions = match payload.create_version_payload() {
        Some(payload) => {
            let perm_string = Permission::CanCreatePoVersion.to_string();
            let vers_desired_state = payload.workflow_status().to_string();
            // Validate the intended state for the new version
            if payload.is_draft() && &vers_desired_state != "editable"
                || !payload.is_draft() && &vers_desired_state != "proposed"
            {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Version draft status {} does not match intended workflow state {}",
                    payload.is_draft(),
                    &vers_desired_state,
                )));
            }
            let beginning_workflow = get_workflow(&workflow.to_string())
                .ok_or_else(|| ApplyError::InternalError("Cannot build PO Workflow".to_string()))?;
            let version_subworkflow =
                beginning_workflow.subworkflow("version").ok_or_else(|| {
                    ApplyError::InternalError("Unable to get `version` subworkflow".to_string())
                })?;
            let start_state = version_subworkflow.state("create").ok_or_else(|| {
                ApplyError::InternalError("Unable to get create state from subworkflow".to_string())
            })?;
            let perm_result = perm_checker
                .check_permission_with_workflow(
                    &perm_string,
                    signer,
                    agent.org_id(),
                    start_state,
                    &vers_desired_state,
                )
                .map_err(|err| {
                    ApplyError::InternalError(format!(
                        "Unable to check agent's permission: {}",
                        err
                    ))
                })?;

            if !perm_result {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Agent {} does not have the correct permissions for organization {} to create \
                     purchase order version {} with a status of {}",
                    signer,
                    agent.org_id(),
                    payload.version_id(),
                    vers_desired_state,
                )));
            }
            // Create the version to be added to this Purchase Order
            let payload_revision = payload.revision();
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
            let version = PurchaseOrderVersionBuilder::new()
                .with_version_id(payload.version_id().to_string())
                .with_workflow_status(vers_desired_state)
                .with_is_draft(payload.is_draft())
                .with_current_revision_id(revision.revision_id())
                .with_revisions(vec![revision])
                .build()
                .map_err(|err| {
                    ApplyError::InvalidTransaction(format!(
                        "Cannot build purchase order version: {}",
                        err
                    ))
                })?;

            // Add version to the list
            vec![version]
        }
        None => {
            workflow = POWorkflow::Collaborative;
            vec![]
        }
    };
    let beginning_workflow = get_workflow(&workflow.to_string()).ok_or_else(|| {
        ApplyError::InternalError("Cannot build System Of Record PO workflow".to_string())
    })?;
    let po_subworkflow = beginning_workflow
        .subworkflow("po")
        .ok_or_else(|| ApplyError::InternalError("Unable to get po subworkflow".to_string()))?;
    let start_state = po_subworkflow.state("create").ok_or_else(|| {
        ApplyError::InternalError("Unable to get create state from subworkflow".to_string())
    })?;
    let perm_string = Permission::CanCreatePo.to_string();
    let desired_state = payload.workflow_status();
    let perm_result = perm_checker
        .check_permission_with_workflow(
            &perm_string,
            signer,
            agent.org_id(),
            start_state,
            desired_state,
        )
        .map_err(|err| {
            ApplyError::InternalError(format!("Unable to check agent's permission: {}", err))
        })?;
    if !perm_result {
        return Err(ApplyError::InvalidTransaction(format!(
            "Agent {} does not have the correct permissions for organization {} to create a \
             purchase order {} with a status of {}",
            signer,
            agent.org_id(),
            payload.uid(),
            desired_state,
        )));
    }

    let purchase_order = PurchaseOrderBuilder::new()
        .with_uid(po_uid.to_string())
        .with_versions(versions)
        .with_workflow_status(desired_state.to_string())
        .with_created_at(payload.created_at())
        .with_is_closed(false)
        .with_buyer_org_id(buyer_org_id)
        .with_seller_org_id(seller_org_id)
        .with_workflow_type(workflow.to_string())
        .build()
        .map_err(|err| {
            ApplyError::InvalidTransaction(format!("Cannot build purchase order: {}", err))
        })?;

    state.set_purchase_order(po_uid, purchase_order)?;

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
    payload: &CreateVersionPayload,
    signer: &str,
    state: &mut PurchaseOrderState,
    perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    // Validate the signer exists as an agent
    let agent_org_id = state
        .get_agent(signer)?
        .ok_or_else(|| {
            ApplyError::InvalidTransaction(format!("Signer {} does not exist as an agent", signer))
        })?
        .org_id()
        .to_string();
    let po_uid = payload.po_uid();
    let version_id = payload.version_id();
    let existing_po_workflow_type = state
        .get_purchase_order(po_uid)
        .map_err(|err| {
            ApplyError::InternalError(format!(
                "Unable to retrieve purchase order {}: {}",
                po_uid, err
            ))
        })?
        .ok_or_else(|| {
            ApplyError::InvalidTransaction(format!("Purchase order {} does not exist", po_uid))
        })?
        .workflow_type()
        .to_string();
    // Validate this version does not already exist
    if state
        .get_purchase_order_version(po_uid, version_id)?
        .is_some()
    {
        return Err(ApplyError::InvalidTransaction(format!(
            "Version {} already exists for Purchase Order {}",
            version_id, po_uid,
        )));
    }

    let desired_state = payload.workflow_status().to_string();
    // Validate the intended state for the new version
    if payload.is_draft() && desired_state != "editable"
        || !payload.is_draft() && desired_state != "proposed"
    {
        return Err(ApplyError::InvalidTransaction(format!(
            "Version draft status {} does not match intended workflow state {}",
            payload.is_draft(),
            &desired_state,
        )));
    }
    // Get the workflow specific to the purchase order the version is to be added to
    let workflow = get_workflow(&existing_po_workflow_type).ok_or_else(|| {
        ApplyError::InternalError(format!(
            "Unable to get `{}` workflow",
            &existing_po_workflow_type
        ))
    })?;
    let version_subworkflow = workflow.subworkflow("version").ok_or_else(|| {
        ApplyError::InternalError("Unable to get `version` subworkflow".to_string())
    })?;
    let workflow_state = version_subworkflow.state("create").ok_or_else(|| {
        ApplyError::InternalError("Unable to get state from `version` subworkflow".to_string())
    })?;
    // Validate the agent is able to create the purchase order version
    let perm_string = Permission::CanCreatePoVersion.to_string();
    let perm_result = perm_checker
        .check_permission_with_workflow(
            &perm_string,
            signer,
            &agent_org_id,
            workflow_state,
            &desired_state,
        )
        .map_err(|err| {
            ApplyError::InternalError(format!("Unable to check agent's permission: {}", err))
        })?;

    if !perm_result {
        return Err(ApplyError::InvalidTransaction(format!(
            "Agent {} does not have the correct permissions for organization {} to create purchase \
             order version {} with a status of {}",
            signer,
            &agent_org_id,
            payload.version_id(),
            desired_state,
        )));
    }

    // Create the PurchaseOrderRevision to be added to the version
    let payload_revision = payload.revision();
    let revision = PurchaseOrderRevisionBuilder::new()
        .with_revision_id(payload_revision.revision_id())
        .with_submitter(signer.to_string())
        .with_created_at(payload_revision.created_at())
        .with_order_xml_v3_4(payload_revision.order_xml_v3_4().to_string())
        .build()
        .map_err(|err| {
            ApplyError::InvalidTransaction(format!("Cannot build purchase order revision: {}", err))
        })?;
    // Create the PurchaseOrderVersion to be added to state
    let new_version = PurchaseOrderVersionBuilder::new()
        .with_version_id(payload.version_id().to_string())
        .with_workflow_status(desired_state)
        .with_is_draft(payload.is_draft())
        .with_current_revision_id(revision.revision_id())
        .with_revisions(vec![revision])
        .build()
        .map_err(|err| {
            ApplyError::InvalidTransaction(format!("Cannot build purchase order version: {}", err))
        })?;
    // Set the purchase order version in state
    state.set_purchase_order_version(po_uid, new_version)?;

    Ok(())
}

fn update_version(
    _payload: &UpdateVersionPayload,
    _signer: &str,
    _state: &mut PurchaseOrderState,
    _perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    unimplemented!();
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::collections::HashMap;

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
                PayloadRevisionBuilder,
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

    const BUYER_PUB_KEY: &str = "buyer_agent_pub_key";
    const SELLER_PUB_KEY: &str = "seller_agent_pub_key";
    const PARTNER_PUB_KEY: &str = "partner_agent_pub_key";

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
                .with_public_key(BUYER_PUB_KEY.to_string())
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
            let agent_address = compute_agent_address(BUYER_PUB_KEY);
            self.set_state_entry(agent_address, agent_bytes)
                .expect("Unable to set agent in state");
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
    /// Validate a Purchase Order is unable to be created if one with the same UID already exists
    /// in state. The test follows these steps:
    ///
    /// 1. Create the necessary organizations and buyer agent with the correct permissions to
    ///    create a purchase order
    /// 2. Add a Purchase Order to state, with UID `test_po_1`
    /// 3. Build a `CreatePurchaseOrderPayload` with a `uid` field of `test_po_1`
    /// 4. Validate an error is returned when attempting to submit the payload
    ///
    /// This test validates a purchase order must have a unique ID to be added to state.
    fn test_create_po_already_exists() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order(purchase_order());
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_buyer_role();
        ctx.add_buyer_agent();
        let create_po_payload = CreatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_created_at(1)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_status("issued".to_string())
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
    /// Validate a Purchase Order is unable to be created if one of the specified organizations
    /// does not exist in state. The test follows these steps:
    ///
    /// 1. Add an organization to state
    /// 2. Add the buyer agent with the correct permissions to create a purchase order to state
    /// 3. Build a `CreatePurchaseOrderPayload`, which references two different organizations
    /// 4. Validate an error is returned when attempting to submit the payload
    ///
    /// This test validates a purchase order is unable to be created if one of the organizations
    /// referenced by the payload does not exist in state.
    fn test_create_po_org_does_not_exist() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_org(ORG_ID_1);
        ctx.add_buyer_role();
        ctx.add_buyer_agent();
        let create_po_payload = CreatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_created_at(1)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_status("issued".to_string())
            .build()
            .expect("Unable to build CreatePurchaseOrderPayload");

        if let Ok(()) =
            create_purchase_order(&create_po_payload, BUYER_PUB_KEY, &mut state, &perm_checker)
        {
            panic!("New purchase order should be invalid because an organization does not exist")
        }
    }

    #[test]
    /// Validate a Purchase Order is unable to be created if the signer does not exist as an Agent.
    /// The test follows these steps:
    ///
    /// 1. Add the necessary organizations and the roles to create a purchase order to state
    /// 2. Build a `CreatePurchaseOrderPayload`
    /// 3. Validate an error is returned when attempting to submit the payload
    ///
    /// This test validates a purchase order is unable to be created if the signer submitting the
    /// payload does not already exist as an agent in state.
    fn test_create_po_agent_does_not_exist() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_buyer_role();
        let create_po_payload = CreatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_created_at(1)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_status("issued".to_string())
            .build()
            .expect("Unable to build CreatePurchaseOrderPayload");

        if let Ok(()) =
            create_purchase_order(&create_po_payload, BUYER_PUB_KEY, &mut state, &perm_checker)
        {
            panic!("New purchase order should be invalid because submitter agent does not exist")
        }
    }

    #[test]
    /// Validate a Purchase Order is unable to be created if the signing agent does not have the
    /// correct workflow permissions. The test follows these steps:
    ///
    /// 1. Add the necessary organizations to state
    /// 2. Add a seller agent, with the `po::seller` workflow permissions
    /// 3. Build a `CreatePurchaseOrderPayload`
    /// 4. Validate an error is returned when attempting to submit the payload
    ///
    /// This test validates a purchase order is unable to be created if the signer submitting the
    /// payload does not have the correct workflow permissions to create a purchase order.
    /// A seller is not able to create a purchase order, but is able to update with the correct
    /// permissions.
    fn test_create_po_invalid_agent_role() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_seller_role();
        ctx.add_seller_agent();
        let create_po_payload = CreatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_created_at(1)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_status("issued".to_string())
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
    /// Validate a Purchase Order is able to be issued. The test follows these steps:
    ///
    /// 1. Add the necessary organizations and buyer agent to state
    /// 2. Build a `CreatePurchaseOrderPayload`
    /// 3. Validate the payload is submitted successfully
    /// 4. Validate the purchase order in state is returned as expected
    ///
    /// This test validates a purchase order is able to be created within the Purchase Order
    /// workflow.
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
            .with_workflow_status("issued".to_string())
            .build()
            .expect("Unable to build CreatePurchaseOrderPayload");
        if let Err(err) =
            create_purchase_order(&create_po_payload, BUYER_PUB_KEY, &mut state, &perm_checker)
        {
            panic!("Should be valid: {}", err)
        };

        let state_po = state
            .get_purchase_order(PO_UID)
            .expect("Unable to get purchase order from state");
        assert_eq!(state_po, Some(purchase_order_wo_versions()));
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
    ///    `workflow_status` of `proposed`
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
            .with_workflow_status("editable".to_string())
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
    /// 3. Build a `CreateVersionPayload` with an `is_draft` field of `true` and a `workflow_status`
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
            .with_workflow_status("editable".to_string())
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
    /// payload's `workflow_status` does not match the purchase order version's state within a
    /// `Collaborative` version subworkflow. The test follows these steps:
    ///
    /// 1. Create the necessary organizations and create an agent with the "partner" role
    /// 2. Add a Purchase Order to state without versions (This will issue the purchase order
    ///    within the `Collaborative` version subworkflow)
    /// 3. Build a `CreateVersionPayload` with an `is_draft` field of `true` and a `workflow_status`
    ///    of `modified`
    /// 4. Assert the `create_version` function returns an error
    ///
    /// This test validates a purchase order version is unable to be created with a
    /// `workflow_status` of `modified` within the `Collaborative` version subworkflow
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
            .with_workflow_status("modified".to_string())
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
                the desired workflow status is invalid"
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
    ///    `workflow_status` of `proposed`
    /// 4. Assert the `create_version` function returns succesfully
    ///
    /// This test validates a purchase order version is able to be created with a
    /// `workflow_status` of `proposed` within the `Collaborative` version subworkflow
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
            .with_workflow_status("proposed".to_string())
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
    /// 4. Build a `CreateVersionPayload` with an `is_draft` field of `false` and a `workflow_status`
    ///    of `proposed`
    /// 5. Assert the `create_version` function returns successfully
    ///
    /// This test validates a purchase order version is able to be created with a
    /// `workflow_status` of `proposed` within the System of Record version subworkflow as the
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
            .with_workflow_status("proposed".to_string())
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
    /// 4. Build a `CreateVersionPayload` with an `is_draft` field of `true` and a `workflow_status`
    ///    of `editable`
    /// 5. Assert the `create_version` function returns successfully
    ///
    /// This test validates a purchase order version is able to be created with a
    /// `workflow_status` of `editable` within the System of Record version subworkflow as the
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
            .with_submitter(BUYER_PUB_KEY.to_string())
            .with_created_at(1)
            .with_order_xml_v3_4("xml_v3_4_string".to_string())
            .build()
            .expect("Unable to build payload revision");
        let create_po_vers_payload = CreateVersionPayloadBuilder::new()
            .with_version_id(PO_VERSION_ID_2.to_string())
            .with_po_uid(PO_UID.to_string())
            .with_is_draft(true)
            .with_workflow_status("editable".to_string())
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
    /// Validates the `create_version` function returns an error if the version to be created
    /// contains state that invalidates the workflow state it is to be transitioned to.
    /// This test specifically tests the scenario where a purchase order version is created as a
    /// draft and moved into the "proposed" workflow state within the System of Record version
    /// subworkflow. The test follows these steps:
    ///
    /// 1. Add the buyer and seller organizations to state
    /// 2. Create an agent with the "buyer" role
    /// 3. Add a Purchase Order to state with versions (This will issue the purchase order
    ///    within the System of Record version subworkflow)
    /// 4. Build a `CreateVersionPayload` with an `is_draft` field of `true` and a `workflow_status`
    ///    of `proposed`
    /// 5. Assert the `create_version` function returns an error
    ///
    /// This test validates a draft purchase order version is not able to be created with a
    /// `workflow_status` of `proposed` within the System of Record version subworkflow. Draft
    /// versions are moved into the draft workflow state, "editable", upon creation.
    fn test_sys_create_po_vers_invalid_transition_proposed() {
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
            .with_is_draft(true)
            .with_workflow_status("proposed".to_string())
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
                "New purchase order version should be invalid because draft versions are \
            unable to be transitioned to the `proposed` workflow state"
            )
        }
    }

    fn purchase_order() -> PurchaseOrder {
        PurchaseOrderBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_workflow_status("issued".to_string())
            .with_created_at(1)
            .with_versions(vec![purchase_order_version(PO_VERSION_ID_1)])
            .with_is_closed(false)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_type(POWorkflow::SystemOfRecord.to_string())
            .build()
            .expect("Unable to build purchase order")
    }

    fn purchase_order_wo_versions() -> PurchaseOrder {
        PurchaseOrderBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_workflow_status("issued".to_string())
            .with_created_at(1)
            .with_is_closed(false)
            .with_versions(vec![])
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_type(POWorkflow::Collaborative.to_string())
            .build()
            .expect("Unable to build purchase order")
    }

    fn purchase_order_version(version_id: &str) -> PurchaseOrderVersion {
        PurchaseOrderVersionBuilder::new()
            .with_version_id(version_id.to_string())
            .with_workflow_status("editable".to_string())
            .with_is_draft(true)
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
