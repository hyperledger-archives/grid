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
            let mut version_builder = PurchaseOrderVersionBuilder::new()
                .with_version_id(payload.version_id().to_string())
                .with_is_draft(payload.is_draft())
                .with_current_revision_id(revision.revision_id())
                .with_workflow_status(payload.workflow_status().to_string())
                .with_revisions(vec![revision]);
            let perm_string = Permission::CanCreatePoVersion.to_string();
            if payload.is_draft() {
                let beginning_workflow = get_workflow(&workflow.to_string()).ok_or_else(|| {
                    ApplyError::InternalError("Cannot build PO Workflow".to_string())
                })?;
                let version_subworkflow =
                    beginning_workflow.subworkflow("version").ok_or_else(|| {
                        ApplyError::InternalError("Unable to get `version` subworkflow".to_string())
                    })?;
                let start_state = version_subworkflow.state("editable").ok_or_else(|| {
                    ApplyError::InternalError(
                        "Unable to get create state from subworkflow".to_string(),
                    )
                })?;
                let perm_result = perm_checker
                    .check_permission_with_workflow(
                        &perm_string,
                        signer,
                        agent.org_id(),
                        start_state,
                        "editable",
                    )
                    .map_err(|err| {
                        ApplyError::InternalError(format!(
                            "Unable to check agent's permission: {}",
                            err
                        ))
                    })?;
                if !perm_result {
                    return Err(ApplyError::InvalidTransaction(format!(
                        "Agent {} does not have permission {} for organization {}",
                        signer,
                        &perm_string,
                        agent.org_id(),
                    )));
                }
                version_builder = version_builder.with_workflow_status("editable".to_string());
            } else {
                let beginning_workflow = get_workflow(&workflow.to_string()).ok_or_else(|| {
                    ApplyError::InternalError("Cannot build PO Workflow".to_string())
                })?;
                let version_subworkflow =
                    beginning_workflow.subworkflow("version").ok_or_else(|| {
                        ApplyError::InternalError("Unable to get `version` subworkflow".to_string())
                    })?;
                let start_state = version_subworkflow.state("create").ok_or_else(|| {
                    ApplyError::InternalError(
                        "Unable to get create state from subworkflow".to_string(),
                    )
                })?;
                let perm_result = perm_checker
                    .check_permission_with_workflow(
                        &perm_string,
                        signer,
                        agent.org_id(),
                        start_state,
                        "proposed",
                    )
                    .map_err(|err| {
                        ApplyError::InternalError(format!(
                            "Unable to check agent's permission: {}",
                            err
                        ))
                    })?;
                if !perm_result {
                    return Err(ApplyError::InvalidTransaction(format!(
                        "Agent {} does not have permission {} for organization {}",
                        signer,
                        &perm_string,
                        agent.org_id(),
                    )));
                }
                version_builder = version_builder.with_workflow_status("proposed".to_string());
            }

            vec![version_builder.build().map_err(|err| {
                ApplyError::InvalidTransaction(format!(
                    "Cannot build purchase order version: {}",
                    err
                ))
            })?]
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
    let perm_result = perm_checker
        .check_permission_with_workflow(&perm_string, signer, agent.org_id(), start_state, "issued")
        .map_err(|err| {
            ApplyError::InternalError(format!("Unable to check agent's permission: {}", err))
        })?;
    if !perm_result {
        return Err(ApplyError::InvalidTransaction(format!(
            "Agent {} does not have permission {} for organization {}",
            signer,
            &perm_string,
            agent.org_id()
        )));
    }

    let purchase_order = PurchaseOrderBuilder::new()
        .with_uid(po_uid.to_string())
        .with_versions(versions)
        .with_workflow_status("issued".to_string())
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
            payload::CreatePurchaseOrderPayloadBuilder,
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

    const PO_UID: &str = "test_po_1";
    const PO_VERSION_ID_1: &str = "01";

    const ROLE_BUYER: &str = "buyer";
    const PERM_ALIAS_BUYER: &str = "po::buyer";

    const ROLE_SELLER: &str = "seller";
    const PERM_ALIAS_SELLER: &str = "po::seller";

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
                .expect("Unable to convert agent list toy bytes");
            let agent_address = compute_agent_address(SELLER_PUB_KEY);
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

        fn add_purchase_order(&self) {
            let po = purchase_order();
            let list = PurchaseOrderListBuilder::new()
                .with_purchase_orders(vec![po])
                .build()
                .expect("Unable to build purchase order list");
            let po_bytes = list
                .into_bytes()
                .expect("Unable to convert purchase order list to bytes");
            let po_address = compute_purchase_order_address(PO_UID);
            self.set_state_entry(po_address, po_bytes)
                .expect("Unable to add purchase order to state");
        }
    }

    #[test]
    fn test_create_po_already_exists() {
        let ctx = MockTransactionContext::default();
        let mut state = PurchaseOrderState::new(&ctx);
        let perm_checker = PermissionChecker::new(&ctx);
        ctx.add_purchase_order();
        let create_po_payload = CreatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_created_at(1)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_status("proposed".to_string())
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
        ctx.add_org(ORG_ID_1);
        ctx.add_buyer_agent();
        let create_po_payload = CreatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_created_at(1)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_status("proposed".to_string())
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
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        let create_po_payload = CreatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_created_at(1)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_status("proposed".to_string())
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
        ctx.add_org(ORG_ID_1);
        ctx.add_org(ORG_ID_2);
        ctx.add_seller_role();
        ctx.add_seller_agent();
        let create_po_payload = CreatePurchaseOrderPayloadBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_created_at(1)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_status("proposed".to_string())
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
            .with_workflow_status("proposed".to_string())
            .build()
            .expect("Unable to build CreatePurchaseOrderPayload");
        if let Err(err) =
            create_purchase_order(&create_po_payload, BUYER_PUB_KEY, &mut state, &perm_checker)
        {
            panic!("Should be valid: {}", err)
        }
    }

    fn purchase_order() -> PurchaseOrder {
        PurchaseOrderBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_workflow_status("Issued".to_string())
            .with_created_at(1)
            .with_versions(vec![purchase_order_version(PO_VERSION_ID_1)])
            .with_is_closed(false)
            .with_buyer_org_id(ORG_ID_1.to_string())
            .with_seller_org_id(ORG_ID_2.to_string())
            .with_workflow_type(POWorkflow::SystemOfRecord.to_string())
            .build()
            .expect("Unable to build purchase order")
    }

    fn purchase_order_version(version_id: &str) -> PurchaseOrderVersion {
        PurchaseOrderVersionBuilder::new()
            .with_version_id(version_id.to_string())
            .with_workflow_status("Editable".to_string())
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
