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

#[cfg(test)]
mod tests {
    use crate::grid_sdk::protos::IntoBytes;
    use crate::workflow::{get_workflow, POWorkflow};
    use grid_sdk::protocol::pike::state::{
        Agent, AgentBuilder, AgentListBuilder, Role, RoleBuilder, RoleListBuilder,
    };
    use grid_sdk::protocol::purchase_order::state::{
        PurchaseOrderRevision, PurchaseOrderRevisionBuilder, PurchaseOrderVersion,
        PurchaseOrderVersionBuilder,
    };

    use super::*;
    use grid_sdk::pike::addressing::{compute_agent_address, compute_role_address};

    use std::cell::RefCell;
    use std::collections::HashMap;

    use sawtooth_sdk::processor::handler::ContextError;

    const PUBLIC_KEY_ALPHA: &str = "alpha_agent_public_key";
    const PUBLIC_KEY_BETA: &str = "beta_agent_public_key";
    const ORG_ID_ALPHA: &str = "alpha";
    const ORG_ID_BETA: &str = "beta";
    const ROLE_BUYER: &str = "buyer";
    const ROLE_BUYER_BETA: &str = "buyer";
    const PERM_BUYER: &str = "po::buyer";
    const ROLE_SELLER: &str = "seller";
    const PERM_SELLER: &str = "po::seller";

    #[derive(Default)]
    /// A MockTransactionContext that can be used to test PermissionChecker
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

    #[test]
    fn test_can_create_po() {
        let context = MockTransactionContext::default();
        let perm_checker = PermissionChecker::new(&context);

        let role_builder = RoleBuilder::new();
        let role = role_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_name(ROLE_BUYER.to_string())
            .with_permissions(vec![PERM_BUYER.to_string()])
            .build()
            .unwrap();

        let role_address = compute_role_address(ROLE_BUYER, ORG_ID_ALPHA);
        context
            .set_state_entry(role_address, role_to_bytes(role))
            .unwrap();

        let agent_builder = AgentBuilder::new();
        let agent = agent_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_public_key(PUBLIC_KEY_ALPHA.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_BUYER.to_string()])
            .build()
            .unwrap();

        let agent_address = compute_agent_address(PUBLIC_KEY_ALPHA);
        context
            .set_state_entry(agent_address, agent_to_bytes(agent))
            .unwrap();

        let workflow = get_workflow(POWorkflow::Collaborative).unwrap();
        let subworkflow = workflow.subworkflow("po").unwrap();
        let state = subworkflow.state("create").unwrap();

        let result = check_permission_with_workflow(
            &perm_checker,
            "can-create-po",
            PUBLIC_KEY_ALPHA,
            ORG_ID_ALPHA,
            state,
            "issued",
        );

        assert_eq!((), result.unwrap())
    }

    #[test]
    fn test_wrong_role() {
        let context = MockTransactionContext::default();
        let perm_checker = PermissionChecker::new(&context);

        let buyer_role_builder = RoleBuilder::new();
        let buyer_role = buyer_role_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_name(ROLE_BUYER.to_string())
            .with_permissions(vec![PERM_BUYER.to_string()])
            .build()
            .unwrap();

        let buyer_role_address = compute_role_address(ROLE_BUYER, ORG_ID_ALPHA);
        context
            .set_state_entry(buyer_role_address, role_to_bytes(buyer_role))
            .unwrap();

        let seller_role_builder = RoleBuilder::new();
        let seller_role = seller_role_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_name(ROLE_SELLER.to_string())
            .with_permissions(vec![PERM_SELLER.to_string()])
            .build()
            .unwrap();

        let seller_role_address = compute_role_address(ROLE_SELLER, ORG_ID_ALPHA);
        context
            .set_state_entry(seller_role_address, role_to_bytes(seller_role))
            .unwrap();

        let agent_builder = AgentBuilder::new();
        let agent = agent_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_public_key(PUBLIC_KEY_ALPHA.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_SELLER.to_string()])
            .build()
            .unwrap();

        let agent_address = compute_agent_address(PUBLIC_KEY_ALPHA);
        context
            .set_state_entry(agent_address, agent_to_bytes(agent))
            .unwrap();

        let workflow = get_workflow(POWorkflow::Collaborative).unwrap();
        let subworkflow = workflow.subworkflow("po").unwrap();
        let state = subworkflow.state("create").unwrap();

        let result = check_permission_with_workflow(
            &perm_checker,
            "can-create-po",
            PUBLIC_KEY_ALPHA,
            ORG_ID_ALPHA,
            state,
            "issued",
        );

        match result {
            Ok(()) => panic!("Agent should not have permission"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert_eq!(
                    "Signer alpha_agent_public_key does not have permission can-create-po",
                    err
                );
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    fn test_cannot_transition() {
        let context = MockTransactionContext::default();
        let perm_checker = PermissionChecker::new(&context);

        let buyer_role_builder = RoleBuilder::new();
        let buyer_role = buyer_role_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_name(ROLE_BUYER.to_string())
            .with_permissions(vec![PERM_BUYER.to_string()])
            .build()
            .unwrap();

        let buyer_role_address = compute_role_address(ROLE_BUYER, ORG_ID_ALPHA);
        context
            .set_state_entry(buyer_role_address, role_to_bytes(buyer_role))
            .unwrap();

        let agent_builder = AgentBuilder::new();
        let agent = agent_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_public_key(PUBLIC_KEY_ALPHA.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_BUYER.to_string()])
            .build()
            .unwrap();

        let agent_address = compute_agent_address(PUBLIC_KEY_ALPHA);
        context
            .set_state_entry(agent_address, agent_to_bytes(agent))
            .unwrap();

        let workflow = get_workflow(POWorkflow::Collaborative).unwrap();
        let subworkflow = workflow.subworkflow("po").unwrap();
        let state = subworkflow.state("issued").unwrap();

        let result = check_permission_with_workflow(
            &perm_checker,
            "can-update-po",
            PUBLIC_KEY_ALPHA,
            ORG_ID_ALPHA,
            state,
            "confirmed",
        );

        match result {
            Ok(()) => panic!("Agent should not have permission to transition"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert_eq!(
                    "Signer alpha_agent_public_key does not have permission can-update-po",
                    err
                );
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    fn test_allowed_org_create() {
        let context = MockTransactionContext::default();
        let perm_checker = PermissionChecker::new(&context);

        let buyer_role_alpha_builder = RoleBuilder::new();
        let buyer_role_alpha = buyer_role_alpha_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_name(ROLE_BUYER.to_string())
            .with_permissions(vec![PERM_BUYER.to_string()])
            .with_allowed_organizations(vec![ORG_ID_BETA.to_string()])
            .build()
            .unwrap();

        let buyer_role_alpha_address = compute_role_address(ROLE_BUYER, ORG_ID_ALPHA);
        context
            .set_state_entry(buyer_role_alpha_address, role_to_bytes(buyer_role_alpha))
            .unwrap();

        let buyer_role_beta_builder = RoleBuilder::new();
        let buyer_role_beta = buyer_role_beta_builder
            .with_org_id(ORG_ID_BETA.to_string())
            .with_name(ROLE_BUYER_BETA.to_string())
            .with_permissions(vec![])
            .with_inherit_from(vec![format!("{}.{}", ORG_ID_ALPHA, ROLE_BUYER)])
            .build()
            .unwrap();

        let buyer_role_beta_address = compute_role_address(ROLE_BUYER, ORG_ID_BETA);
        context
            .set_state_entry(buyer_role_beta_address, role_to_bytes(buyer_role_beta))
            .unwrap();

        let agent_builder = AgentBuilder::new();
        let agent = agent_builder
            .with_org_id(ORG_ID_BETA.to_string())
            .with_public_key(PUBLIC_KEY_BETA.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_BUYER_BETA.to_string()])
            .build()
            .unwrap();

        let agent_address = compute_agent_address(PUBLIC_KEY_BETA);
        context
            .set_state_entry(agent_address, agent_to_bytes(agent))
            .unwrap();

        let workflow = get_workflow(POWorkflow::Collaborative).unwrap();
        let subworkflow = workflow.subworkflow("po").unwrap();
        let state = subworkflow.state("create").unwrap();

        let result = check_permission_with_workflow(
            &perm_checker,
            "can-create-po",
            PUBLIC_KEY_BETA,
            ORG_ID_ALPHA,
            state,
            "issued",
        );

        assert_eq!((), result.unwrap())
    }

    #[test]
    fn test_disallowed_org() {
        let context = MockTransactionContext::default();
        let perm_checker = PermissionChecker::new(&context);

        let buyer_role_alpha_builder = RoleBuilder::new();
        let buyer_role_alpha = buyer_role_alpha_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_name(ROLE_BUYER.to_string())
            .with_permissions(vec![PERM_BUYER.to_string()])
            .build()
            .unwrap();

        let buyer_role_alpha_address = compute_role_address(ROLE_BUYER, ORG_ID_ALPHA);
        context
            .set_state_entry(buyer_role_alpha_address, role_to_bytes(buyer_role_alpha))
            .unwrap();

        let buyer_role_beta_builder = RoleBuilder::new();
        let buyer_role_beta = buyer_role_beta_builder
            .with_org_id(ORG_ID_BETA.to_string())
            .with_name(ROLE_BUYER_BETA.to_string())
            .with_permissions(vec![])
            .with_inherit_from(vec![format!("{}.{}", ORG_ID_ALPHA, ROLE_BUYER)])
            .build()
            .unwrap();

        let buyer_role_beta_address = compute_role_address(ROLE_BUYER, ORG_ID_BETA);
        context
            .set_state_entry(buyer_role_beta_address, role_to_bytes(buyer_role_beta))
            .unwrap();

        let agent_builder = AgentBuilder::new();
        let agent = agent_builder
            .with_org_id(ORG_ID_BETA.to_string())
            .with_public_key(PUBLIC_KEY_BETA.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_BUYER_BETA.to_string()])
            .build()
            .unwrap();

        let agent_address = compute_agent_address(PUBLIC_KEY_BETA);
        context
            .set_state_entry(agent_address, agent_to_bytes(agent))
            .unwrap();

        let workflow = get_workflow(POWorkflow::Collaborative).unwrap();
        let subworkflow = workflow.subworkflow("po").unwrap();
        let state = subworkflow.state("create").unwrap();

        let result = check_permission_with_workflow(
            &perm_checker,
            "can-create-po",
            PUBLIC_KEY_BETA,
            ORG_ID_ALPHA,
            state,
            "issued",
        );

        match result {
            Ok(()) => {
                panic!("Agent should not have permission because org is not allowed to inherit")
            }
            Err(ApplyError::InvalidTransaction(err)) => {
                assert_eq!(
                    "Signer beta_agent_public_key does not have permission can-create-po",
                    err
                );
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    fn agent_to_bytes(agent: Agent) -> Vec<u8> {
        let builder = AgentListBuilder::new();
        let agent_list = builder.with_agents(vec![agent.clone()]).build().unwrap();
        return agent_list.into_bytes().unwrap();
    }

    fn role_to_bytes(role: Role) -> Vec<u8> {
        let builder = RoleListBuilder::new();
        let role_list = builder.with_roles(vec![role.clone()]).build().unwrap();
        return role_list.into_bytes().unwrap();
    }
}
