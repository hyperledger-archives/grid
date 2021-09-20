// Copyright 2019-2021 Cargill Incorporated
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

pub mod error;
use crate::pike::addressing::{compute_agent_address, compute_role_address};
use crate::pike::permissions::error::PermissionCheckerError;
use crate::protocol::pike::state::{Agent, AgentList, Role, RoleList};
use crate::protos::FromBytes;
use crate::workflow::WorkflowState;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::TransactionContext;
    } else {
        use sawtooth_sdk::processor::handler::TransactionContext;
    }
}
/// Helper struct for Pike functionality.
pub struct PermissionChecker<'a> {
    /// A PermissionChecker is tied to a version of state, so it has a
    /// reference to a TransactionContext.
    context: &'a dyn TransactionContext,
}

impl<'a> PermissionChecker<'a> {
    /// Returns a PermissionChecker for a certain context.
    ///
    /// # Arguments
    ///
    /// * `context` - A reference to the transaction context.
    ///
    pub fn new(context: &'a dyn TransactionContext) -> PermissionChecker {
        PermissionChecker { context }
    }

    /// Checks whether an agent with a given public key has a certain role and
    /// belongs to the organization that owns the record.
    ///
    /// # Arguments
    ///
    /// * `public_key` - Public key of a Pike agent.
    /// * `permission` - Permission string to be checked.
    /// * `record_owner` - Pike organization ID of the record owner.
    ///
    pub fn has_permission(
        &self,
        public_key: &str,
        permission: &str,
        record_owner: &str,
    ) -> Result<bool, PermissionCheckerError> {
        let agent = self.get_agent(public_key)?;

        match agent {
            Some(agent) => {
                if !agent.active() {
                    return Ok(false);
                }

                let agent_roles: Vec<Role> = agent
                    .roles()
                    .iter()
                    .filter_map(|r| {
                        if r.contains('.') {
                            self.get_role(r, None).ok()
                        } else {
                            self.get_role(r, Some(agent.org_id())).ok()
                        }
                    })
                    .flatten()
                    .collect();

                Ok(self.check_roles_for_permission(
                    permission,
                    &agent_roles,
                    record_owner,
                    agent.org_id(),
                ))
            }
            None => Err(PermissionCheckerError::InvalidPublicKey(format!(
                "The signer is not an Agent: {}",
                public_key
            ))),
        }
    }

    fn check_roles_for_permission(
        &self,
        permission: &str,
        roles: &[Role],
        record_owner: &str,
        agent_org_id: &str,
    ) -> bool {
        roles.iter().any(|r| {
            if r.permissions().iter().any(|p| p == permission) {
                record_owner == r.org_id()
                    || r.allowed_organizations()
                        .iter()
                        .any(|org| org == record_owner)
            } else {
                if r.inherit_from().is_empty() {
                    return false;
                }
                let inheriting_roles: Vec<Role> = r
                    .inherit_from()
                    .iter()
                    .filter_map(|role| {
                        if role.contains('.') {
                            self.get_role(role, None).ok()
                        } else {
                            self.get_role(role, Some(agent_org_id)).ok()
                        }
                    })
                    .flatten()
                    .filter(|role| {
                        role.org_id() == agent_org_id
                            || role
                                .allowed_organizations()
                                .contains(&agent_org_id.to_string())
                    })
                    .collect();

                self.check_roles_for_permission(
                    permission,
                    &inheriting_roles,
                    record_owner,
                    agent_org_id,
                )
            }
        })
    }

    pub fn get_agent(&self, public_key: &str) -> Result<Option<Agent>, PermissionCheckerError> {
        let address = compute_agent_address(public_key);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let agent_list = AgentList::from_bytes(packed.as_slice())?;
                for agent in agent_list.agents() {
                    if agent.public_key() == public_key {
                        return Ok(Some(agent.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn get_role(
        &self,
        name: &str,
        org_id: Option<&str>,
    ) -> Result<Option<Role>, PermissionCheckerError> {
        let (name, org_id) = match org_id {
            Some(org_id) => (name, org_id),
            None => {
                if name.contains('.') {
                    let t: Vec<&str> = name.split('.').collect();
                    let org_id = t[0];
                    let name = t[1];
                    (name, org_id)
                } else {
                    return Err(PermissionCheckerError::InvalidRole(
                        "External roles need to be prefixed with their org ID. Format: <org_id>.<role_name>"
                        .to_string(),
                    ));
                }
            }
        };

        let address = compute_role_address(name, org_id);

        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let role_list = RoleList::from_bytes(packed.as_slice())?;
                for role in role_list.roles() {
                    if role.name() == name && role.org_id() == org_id {
                        return Ok(Some(role.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn check_permission_with_workflow(
        &self,
        permission: &str,
        signer: &str,
        record_owner: &str,
        workflow_state: WorkflowState,
        desired_state: &str,
    ) -> Result<bool, PermissionCheckerError> {
        let agent = self.get_agent(signer)?.ok_or_else(|| {
            PermissionCheckerError::InvalidPublicKey(format!(
                "Agent with public key {} does not exist",
                signer
            ))
        })?;
        // Collect the agent's permission aliases
        let mut agent_perms = Vec::new();
        agent.roles().iter().for_each(|r| {
            let mut org_id = Some(record_owner);
            if r.contains('.') {
                org_id = None;
            }

            let role = self.get_role(r, org_id).ok().flatten();

            if let Some(role) = role {
                agent_perms.extend_from_slice(role.permissions());
            }
        });
        let mut has_perm_alias = false;
        // Retrieve the aliases assigned the permission being validating
        let perm_aliases = workflow_state.get_aliases_by_permission(permission);
        for alias in perm_aliases {
            // If the agent has a permission alias within this list, the agent presumably has
            // the permission being validated (as this list was collected using the
            // `WorkflowState`'s `get_aliases_by_permission` method).
            if self.has_permission(signer, &alias, record_owner)? {
                has_perm_alias = true;
            }
        }
        // Retrieve the agent's permissions, as determined by their assigned permission aliases
        let agent_workflow_permissions = workflow_state.expand_permissions(&agent_perms);
        // Validate the aliases used by the agent has the correct permission assigned to it
        let has_permission = agent_workflow_permissions.contains(&permission.to_string());

        // Validate the agent is able to make the desired transition, based on the agent's
        // permission aliases
        let can_transition = workflow_state.can_transition(desired_state.to_string(), agent_perms);

        Ok(has_perm_alias && has_permission && can_transition)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::protocol::pike::state::{
        AgentBuilder, AgentListBuilder, RoleBuilder, RoleListBuilder,
    };
    use crate::protos::IntoBytes;
    use crate::workflow::{
        PermissionAlias, SubWorkflow, SubWorkflowBuilder, Workflow, WorkflowStateBuilder,
    };

    use sawtooth_sdk::processor::handler::ContextError;

    const PUBLIC_KEY_ALPHA: &str = "alpha_agent_public_key";
    const PUBLIC_KEY_BETA: &str = "beta_agent_public_key";
    const PUBLIC_KEY_GAMMA_1: &str = "gamma_agent_public_key_1";

    const ORG_ID_ALPHA: &str = "alpha";
    const ORG_ID_BETA: &str = "beta";
    const ORG_ID_GAMMA: &str = "gamma";

    const PUBLIC_KEY: &str = "test_public_key";
    const ORG_ID: &str = "test_org";
    const WRONG_ORG_ID: &str = "test_wrong_org";

    const PERM_CAN_DRIVE: &str = "tankops::can-drive";
    const PERM_CAN_TURN_TURRET: &str = "tankops::can-turn-turret";
    const PERM_CAN_FIRE: &str = "tankops::can-fire";

    const ROLE_ALPHA_INSPECTOR: &str = "Inspector";
    const ROLE_ALPHA_DRIVER: &str = "Driver";
    const ROLE_BETA_DRIVER: &str = "Driver";
    const ROLE_GAMMA_NAVIGATOR: &str = "Navigator";

    const ROLE_ALPHA_BUYER: &str = "buyer";
    const ROLE_BETA_BUYER: &str = "buyer";
    const ROLE_ALPHA_SELLER: &str = "seller";
    const ROLE_BETA_SELLER: &str = "seller";

    const PERM_ALIAS_BUYER: &str = "po::buyer";
    const PERM_ALIAS_SELLER: &str = "po::seller";
    const PERM_CAN_CREATE_PO: &str = "can-create-po";
    const PERM_CAN_UPDATE_PO_VERSION: &str = "can-update-po-version";

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

    /// These tests are based on the example in the Grid Identity RFC.

    /// has_permission() returns false if the agent doesn't have any roles.
    #[test]
    fn test_alpha_can_decommission_no_roles() {
        let context = MockTransactionContext::default();
        let pc = PermissionChecker::new(&context);

        let agent_builder = AgentBuilder::new();
        let agent = agent_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_public_key(PUBLIC_KEY_ALPHA.to_string())
            .with_active(true)
            .build()
            .unwrap();

        let agent_address = compute_agent_address(PUBLIC_KEY_ALPHA);
        context
            .set_state_entry(agent_address, agent_to_bytes(agent))
            .unwrap();

        let result = pc
            .has_permission(PUBLIC_KEY_ALPHA, "tankops::can-decommission", ORG_ID)
            .unwrap();
        assert!(!result);
    }

    /// has_permission() returns false if the agent doesn't have any roles with
    /// the given permission.
    #[test]
    fn test_alpha_can_decommission_wrong_role() {
        let context = MockTransactionContext::default();
        let pc = PermissionChecker::new(&context);

        let role_builder = RoleBuilder::new();
        let role = role_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_name(ROLE_ALPHA_INSPECTOR.to_string())
            .with_permissions(vec![PERM_CAN_DRIVE.to_string()])
            .build()
            .unwrap();

        let role_address = compute_role_address(ROLE_ALPHA_INSPECTOR, ORG_ID_ALPHA);
        context
            .set_state_entry(role_address, role_to_bytes(role))
            .unwrap();

        let agent_builder = AgentBuilder::new();
        let agent = agent_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_public_key(PUBLIC_KEY_ALPHA.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_ALPHA_DRIVER.to_string()])
            .build()
            .unwrap();

        let agent_address = compute_agent_address(PUBLIC_KEY_ALPHA);
        context
            .set_state_entry(agent_address, agent_to_bytes(agent))
            .unwrap();

        let result = pc
            .has_permission(PUBLIC_KEY_ALPHA, "tankops::can-decommission", ORG_ID)
            .unwrap();
        assert!(!result);
    }

    /// has_permission() returns true if the agent has a role with multiple
    /// given permissions.
    #[test]
    fn test_alpha_multiple_perms() {
        let context = MockTransactionContext::default();
        let pc = PermissionChecker::new(&context);

        let role_builder = RoleBuilder::new();
        let role = role_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_name(ROLE_ALPHA_DRIVER.to_string())
            .with_permissions(vec![
                PERM_CAN_DRIVE.to_string(),
                PERM_CAN_FIRE.to_string(),
                PERM_CAN_TURN_TURRET.to_string(),
            ])
            .build()
            .unwrap();

        let role_address = compute_role_address(ROLE_ALPHA_DRIVER, ORG_ID_ALPHA);
        context
            .set_state_entry(role_address, role_to_bytes(role))
            .unwrap();

        let agent_builder = AgentBuilder::new();
        let agent = agent_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_public_key(PUBLIC_KEY_ALPHA.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_ALPHA_DRIVER.to_string()])
            .build()
            .unwrap();

        let agent_address = compute_agent_address(PUBLIC_KEY_ALPHA);
        context
            .set_state_entry(agent_address, agent_to_bytes(agent))
            .unwrap();

        let result = pc
            .has_permission(PUBLIC_KEY_ALPHA, PERM_CAN_DRIVE, ORG_ID)
            .unwrap();
        assert!(!result);
    }

    /// has_permission() returns true if the agent has a role with multiple
    /// given permissions from an inherited role.
    #[test]
    fn test_beta_multiple_perms() {
        let context = MockTransactionContext::default();
        let pc = PermissionChecker::new(&context);

        let alpha_role_builder = RoleBuilder::new();
        let alpha_role = alpha_role_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_name(ROLE_ALPHA_DRIVER.to_string())
            .with_permissions(vec![
                PERM_CAN_DRIVE.to_string(),
                PERM_CAN_FIRE.to_string(),
                PERM_CAN_TURN_TURRET.to_string(),
            ])
            .with_allowed_organizations(vec![ORG_ID_BETA.to_string()])
            .build()
            .unwrap();

        let alpha_role_address = compute_role_address(ROLE_ALPHA_DRIVER, ORG_ID_ALPHA);
        context
            .set_state_entry(alpha_role_address, role_to_bytes(alpha_role))
            .unwrap();

        let beta_role_builder = RoleBuilder::new();
        let beta_role = beta_role_builder
            .with_org_id(ORG_ID_BETA.to_string())
            .with_name(ROLE_BETA_DRIVER.to_string())
            .with_permissions(vec![])
            .with_inherit_from(vec![format!("{}.{}", ORG_ID_ALPHA, ROLE_ALPHA_DRIVER)])
            .build()
            .unwrap();

        let beta_role_address = compute_role_address(ROLE_BETA_DRIVER, ORG_ID_BETA);
        context
            .set_state_entry(beta_role_address, role_to_bytes(beta_role))
            .unwrap();

        let agent_builder = AgentBuilder::new();
        let agent = agent_builder
            .with_org_id(ORG_ID_BETA.to_string())
            .with_public_key(PUBLIC_KEY_BETA.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_BETA_DRIVER.to_string()])
            .build()
            .unwrap();

        let agent_address = compute_agent_address(PUBLIC_KEY_BETA);
        context
            .set_state_entry(agent_address, agent_to_bytes(agent))
            .unwrap();

        let result = pc
            .has_permission(PUBLIC_KEY_BETA, PERM_CAN_DRIVE, ORG_ID_BETA)
            .unwrap();
        assert!(result);
    }

    /// has_permission() returns true for agents that have a role with a given
    /// permission and false for agents that have a role without a given
    /// permission. This is to test that inherited roles can be properly
    /// decomposed by the inheriting org.
    #[test]
    fn test_gamma_multiple_perms_multiple_agents() {
        let context = MockTransactionContext::default();
        let pc = PermissionChecker::new(&context);

        let alpha_role_builder = RoleBuilder::new();
        let alpha_role = alpha_role_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_name(ROLE_ALPHA_DRIVER.to_string())
            .with_permissions(vec![
                PERM_CAN_DRIVE.to_string(),
                PERM_CAN_FIRE.to_string(),
                PERM_CAN_TURN_TURRET.to_string(),
            ])
            .with_allowed_organizations(vec![ORG_ID_GAMMA.to_string()])
            .build()
            .unwrap();

        let alpha_role_address = compute_role_address(ROLE_ALPHA_DRIVER, ORG_ID_ALPHA);
        context
            .set_state_entry(alpha_role_address, role_to_bytes(alpha_role))
            .unwrap();

        let gamma_role_navigator_builder = RoleBuilder::new();
        let gamma_role_navigator = gamma_role_navigator_builder
            .with_org_id(ORG_ID_GAMMA.to_string())
            .with_name(ROLE_GAMMA_NAVIGATOR.to_string())
            .with_permissions(vec![PERM_CAN_DRIVE.to_string()])
            .with_inherit_from(vec![format!("{}.{}", ORG_ID_ALPHA, ROLE_ALPHA_DRIVER)])
            .build()
            .unwrap();

        let gamma_role_navigator_address = compute_role_address(ROLE_GAMMA_NAVIGATOR, ORG_ID_GAMMA);
        context
            .set_state_entry(
                gamma_role_navigator_address,
                role_to_bytes(gamma_role_navigator),
            )
            .unwrap();

        let agent_builder = AgentBuilder::new();
        let agent = agent_builder
            .with_org_id(ORG_ID_GAMMA.to_string())
            .with_public_key(PUBLIC_KEY_GAMMA_1.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_GAMMA_NAVIGATOR.to_string()])
            .build()
            .unwrap();

        let agent_address = compute_agent_address(PUBLIC_KEY_GAMMA_1);
        context
            .set_state_entry(agent_address, agent_to_bytes(agent))
            .unwrap();

        let result = pc
            .has_permission(PUBLIC_KEY_GAMMA_1, PERM_CAN_TURN_TURRET, ORG_ID_GAMMA)
            .unwrap();
        assert!(result);
    }

    #[test]
    // Test that if an agent has the correct roles but the record doesn't belong their org, false is returned
    fn test_has_wrong_org() {
        let context = MockTransactionContext::default();
        let pc = PermissionChecker::new(&context);

        let builder = AgentBuilder::new();
        let agent = builder
            .with_org_id(ORG_ID.to_string())
            .with_public_key(PUBLIC_KEY.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_ALPHA_DRIVER.to_string()])
            .build()
            .unwrap();
        let builder = AgentListBuilder::new();
        let agent_list = builder.with_agents(vec![agent.clone()]).build().unwrap();
        let agent_bytes = agent_list.into_bytes().unwrap();
        let agent_address = compute_agent_address(PUBLIC_KEY);
        context.set_state_entry(agent_address, agent_bytes).unwrap();

        let result = pc
            .has_permission(PUBLIC_KEY, PERM_CAN_DRIVE, WRONG_ORG_ID)
            .unwrap();
        assert!(!result);
    }

    /// The following tests are inspired by the Workflow RFC.

    #[test]
    /// Test an agent with the correct workflow permissions is validated successfully.
    ///
    /// 1. Create the TransactionContext, heretofore referred to as state, and PermissionChecker
    /// 2. Create a "buyer" role with the permissions set to the workflow permission alias,
    ///    "po::buyer" and set this role in state.
    /// 3. Create an agent with the "buyer" role and set this agent in state.
    /// 4. Create a Workflow that contains a permission alias "po::buyer", corresponding to the
    ///    permission assigned to the "buyer" role in Pike.
    /// 5. Check that `check_permission_with_workflow`, when given the permission "can-create-po"
    ///    and a desired state of "issued", is able to successfully validate the agent, added to
    ///    state in the previous step, has the correct permission, "can-create-po", to transition
    ///    the workflow state to "issued" in order to create a purchase order in state.
    fn test_permission_with_workflow() {
        let context = MockTransactionContext::default();
        let perm_checker = PermissionChecker::new(&context);
        // Add the buyer role to state
        let role = RoleBuilder::new()
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_name(ROLE_ALPHA_BUYER.to_string())
            .with_permissions(vec![PERM_ALIAS_BUYER.to_string()])
            .build()
            .expect("Unable to build Role");
        let role_address = compute_role_address(ROLE_ALPHA_BUYER, ORG_ID_ALPHA);
        context
            .set_state_entry(role_address, role_to_bytes(role))
            .expect("Unable to set Role in state");
        // Add the Agent to state, with the buyer role
        let agent_builder = AgentBuilder::new();
        let agent = agent_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_public_key(PUBLIC_KEY_ALPHA.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_ALPHA_BUYER.to_string()])
            .build()
            .expect("Unable to build Agent");
        let agent_address = compute_agent_address(PUBLIC_KEY_ALPHA);
        context
            .set_state_entry(agent_address, agent_to_bytes(agent))
            .expect("Unable to set Agent in state");
        // Create the Workflow
        let workflow = get_workflow();
        let subworkflow = workflow
            .subworkflow("po")
            .expect("Unable to get po subworkflow");
        let state = subworkflow
            .state("create")
            .expect("Unable to get create state from subworkflow");
        // Validate that the Agent has the correct permission
        let result = perm_checker
            .check_permission_with_workflow(
                PERM_CAN_CREATE_PO,
                PUBLIC_KEY_ALPHA,
                ORG_ID_ALPHA,
                state,
                "issued",
            )
            .expect("Unable to check permission with workflow");
        assert!(result);
    }

    #[test]
    /// Test an agent with the incorrect workflow permissions is validated successfully.
    ///
    /// 1. Create the TransactionContext, heretofore referred to as state, and PermissionChecker
    /// 2. Create a "buyer" role with the permissions set to the workflow permission alias,
    ///    "po::buyer" and set this role in state.
    /// 3. Create a "seller" role with the permissions set to the workflow permission alias,
    ///    "po::seller" and set this role in state.
    /// 4. Create an agent with the "seller" role and set this agent in state.
    /// 5. Create a Workflow that contains a permission aliases "po::seller", corresponding to the
    ///    permission assigned to the "seller" role in Pike, and "po::buyer" which was not assigned
    ///    to the agent.
    /// 6. Check that `check_permission_with_workflow`, when given the permission "can-create-po"
    ///    and a desired state of "issued", is able to successfully validate the agent does not
    ///    have the correct permission, "can-create-po" to transition the workflow state to "issued"
    ///    in order to create a purchase order.
    fn test_permission_with_workflow_invalid_role() {
        let context = MockTransactionContext::default();
        let perm_checker = PermissionChecker::new(&context);
        // Add the buyer role to state
        let role = RoleBuilder::new()
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_name(ROLE_ALPHA_BUYER.to_string())
            .with_permissions(vec![PERM_ALIAS_BUYER.to_string()])
            .build()
            .expect("Unable to build Role");
        let role_address = compute_role_address(ROLE_ALPHA_BUYER, ORG_ID_ALPHA);
        context
            .set_state_entry(role_address, role_to_bytes(role))
            .expect("Unable to set Role in state");
        // Add the seller role to state
        let role = RoleBuilder::new()
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_name(ROLE_ALPHA_SELLER.to_string())
            .with_permissions(vec![PERM_ALIAS_SELLER.to_string()])
            .build()
            .expect("Unable to build Role");
        let role_address = compute_role_address(ROLE_ALPHA_SELLER, ORG_ID_ALPHA);
        context
            .set_state_entry(role_address, role_to_bytes(role))
            .expect("Unable to set Role in state");
        // Add the Agent to state, with the seller role
        let agent_builder = AgentBuilder::new();
        let agent = agent_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_public_key(PUBLIC_KEY_ALPHA.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_ALPHA_SELLER.to_string()])
            .build()
            .expect("Unable to build Agent");
        let agent_address = compute_agent_address(PUBLIC_KEY_ALPHA);
        context
            .set_state_entry(agent_address, agent_to_bytes(agent))
            .expect("Unable to set Agent in state");
        // Create the Workflow
        let workflow = get_workflow();
        let subworkflow = workflow
            .subworkflow("po")
            .expect("Unable to get po subworkflow");
        let state = subworkflow
            .state("create")
            .expect("Unable to get create state from subworkflow");
        // Validate the Agent does not have the correct permission.
        let result = perm_checker
            .check_permission_with_workflow(
                PERM_CAN_CREATE_PO,
                PUBLIC_KEY_ALPHA,
                ORG_ID_ALPHA,
                state,
                "issued",
            )
            .expect("Unable to check permission with workflow");
        assert!(!result)
    }

    #[test]
    /// Test an agent with the incorrect workflow permissions to transition the workflow state
    /// to "confirmed" is validated successfully.
    ///
    /// 1. Create the TransactionContext, heretofore referred to as state, and PermissionChecker
    /// 2. Create a "buyer" role with the permissions set to the workflow permission alias,
    ///    "po::buyer" and set this role in state.
    /// 3. Create an agent with the "buyer" role and set this agent in state.
    /// 4. Create a Workflow that contains a permission alias "po::buyer", corresponding to the
    ///    permission assigned to the "buyer" role in Pike. This permission alias does not have
    ///    the ability to transition the "issued" state to "confirmed".
    /// 5. Check that `check_permission_with_workflow`, when given the permission
    ///    "can-update-po-version" and a desired state of "confirmed", is able to successfully
    ///    validate the agent has a permission that does not allow them to transition the
    ///    "issued" state to "confirmed", though the "po::buyer" alias does have the
    ///    "can-update-po-version" permission.
    fn test_permission_with_workflow_cannot_transition() {
        let context = MockTransactionContext::default();
        let perm_checker = PermissionChecker::new(&context);
        // Add the buyer role to state
        let role = RoleBuilder::new()
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_name(ROLE_ALPHA_BUYER.to_string())
            .with_permissions(vec![PERM_ALIAS_BUYER.to_string()])
            .build()
            .expect("Unable to build Role");
        let role_address = compute_role_address(ROLE_ALPHA_BUYER, ORG_ID_ALPHA);
        context
            .set_state_entry(role_address, role_to_bytes(role))
            .expect("Unable to set Role in state");
        // Add the Agent to state, with the buyer role
        let agent_builder = AgentBuilder::new();
        let agent = agent_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_public_key(PUBLIC_KEY_ALPHA.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_ALPHA_BUYER.to_string()])
            .build()
            .expect("Unable to build Agent");
        let agent_address = compute_agent_address(PUBLIC_KEY_ALPHA);
        context
            .set_state_entry(agent_address, agent_to_bytes(agent))
            .expect("Unable to set Agent in state");
        // Create the Workflow
        let workflow = get_workflow();
        let subworkflow = workflow
            .subworkflow("po")
            .expect("Unable to get po subworkflow");
        let state = subworkflow
            .state("issued")
            .expect("Unable to get issued state from subworkflow");

        let result = perm_checker
            .check_permission_with_workflow(
                PERM_CAN_UPDATE_PO_VERSION,
                PUBLIC_KEY_ALPHA,
                ORG_ID_ALPHA,
                state,
                "confirmed",
            )
            .expect("Unable to check permission with workflow");

        assert!(!result);
    }

    #[test]
    /// Test an agent with the correct workflow permissions, inherited via the Pike role's
    /// `allowed_organizations` and not directly assigned to the agent, is validated successfully.
    /// This ensures agents, with the valid permissions, are able to make updates to the workflow
    /// across organizations.
    ///
    /// 1. Create an instance of state and a `PermissionChecker`
    /// 2. Create a "buyer" role with the permissions set to the workflow permission alias,
    ///    "po::buyer" and set this role in state for the Alpha organization. This role has
    ///    the Beta organization as an `allowed_organization`.
    /// 3. Create a "buyer" role, set to inherit permissions from the Alpha organization's "buyer"
    ///    role, and set this role in state for the Beta organization.
    /// 4. Create an agent with the Beta organization's "buyer" role and set this agent in state.
    /// 5. Create a Workflow that contains a permission alias "po::buyer", corresponding to the
    ///    permission assigned to the "buyer" role in Pike.
    /// 6. Check that `check_permission_with_workflow`, when given the permission "can-create-po"
    ///    and a desired state of "issued", is able to successfully validate the agent has the
    ///    correct permission, "can-create-po", for the Alpha organization in order to transition
    ///    the workflow state to "issued" and create a purchase order in state.
    fn test_permission_with_workflow_allowed_org() {
        let context = MockTransactionContext::default();
        let perm_checker = PermissionChecker::new(&context);
        // Add the alpha buyer role to state
        let role = RoleBuilder::new()
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_name(ROLE_ALPHA_BUYER.to_string())
            .with_permissions(vec![PERM_ALIAS_BUYER.to_string()])
            .with_allowed_organizations(vec![ORG_ID_BETA.to_string()])
            .build()
            .expect("Unable to build Role");
        let role_address = compute_role_address(ROLE_ALPHA_BUYER, ORG_ID_ALPHA);
        context
            .set_state_entry(role_address, role_to_bytes(role))
            .expect("Unable to set Role in state");
        // Add the beta buyer role to state
        let role = RoleBuilder::new()
            .with_org_id(ORG_ID_BETA.to_string())
            .with_name(ROLE_BETA_BUYER.to_string())
            .with_permissions(vec![])
            .with_inherit_from(vec![format!("{}.{}", ORG_ID_ALPHA, ROLE_ALPHA_BUYER)])
            .build()
            .expect("Unable to build Role");
        let role_address = compute_role_address(ROLE_BETA_BUYER, ORG_ID_BETA);
        context
            .set_state_entry(role_address, role_to_bytes(role))
            .expect("Unable to set Role in state");
        // Add the Agent to state, with the beta buyer role
        let agent = AgentBuilder::new()
            .with_org_id(ORG_ID_BETA.to_string())
            .with_public_key(PUBLIC_KEY_BETA.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_BETA_BUYER.to_string()])
            .build()
            .expect("Unable to build Agent");
        let agent_address = compute_agent_address(PUBLIC_KEY_BETA);
        context
            .set_state_entry(agent_address, agent_to_bytes(agent))
            .expect("Unable to set Agent in state");
        // Create the Workflow
        let workflow = get_workflow();
        let subworkflow = workflow
            .subworkflow("po")
            .expect("Unable to get po subworkflow");
        let state = subworkflow
            .state("create")
            .expect("Unable to get create state from subworkflow");
        // Check that the agent is able to transition the state to "issued" and has the
        // "can-create-po" permission for the Alpha organization
        let result = perm_checker
            .check_permission_with_workflow(
                PERM_CAN_CREATE_PO,
                PUBLIC_KEY_BETA,
                ORG_ID_ALPHA,
                state,
                "issued",
            )
            .expect("Unable to check permission with workflow");
        assert!(result);
    }

    #[test]
    /// Test an agent with valid workflow permissions is unable to inherit a role's permissions
    /// if the agent's organization is not in the role's `allowed_organization` field.
    ///
    /// 1. Create an instance of state and a `PermissionChecker`
    /// 2. Create a "buyer" role with the permissions set to the workflow permission alias,
    ///    "po::buyer" and set this role in state for the Alpha organization. This role does not
    ///    have the Beta organization as an `allowed_organization`.
    /// 3. Create a "buyer" role, set to inherit permissions from the Alpha organization's "buyer"
    ///    role, and set this role in state for the Beta organization.
    /// 4. Create an agent with the Beta organization's "buyer" role and set this agent in state.
    /// 5. Create a Workflow that contains a permission alias "po::buyer", corresponding to the
    ///    permission assigned to the "buyer" role in Pike.
    /// 6. Check that `check_permission_with_workflow`, when given the permission "can-create-po"
    ///    and a desired state of "issued" and the Alpha organization as the `record_owner`, is
    ///    able to succesfully evaluate the Beta org's agent does not have the correct permission
    ///    for the Alpha organization to create a purchase order and transition state to "issued".
    fn test_permission_with_workflow_disallowed_org() {
        let context = MockTransactionContext::default();
        let perm_checker = PermissionChecker::new(&context);
        // Add the alpha buyer role to state
        let role = RoleBuilder::new()
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_name(ROLE_ALPHA_BUYER.to_string())
            .with_permissions(vec![PERM_ALIAS_BUYER.to_string()])
            .build()
            .expect("Unable to build Role");
        let role_address = compute_role_address(ROLE_ALPHA_BUYER, ORG_ID_ALPHA);
        context
            .set_state_entry(role_address, role_to_bytes(role))
            .expect("Unable to set Role in state");
        // Add the beta buyer role to state
        let role = RoleBuilder::new()
            .with_org_id(ORG_ID_BETA.to_string())
            .with_name(ROLE_BETA_BUYER.to_string())
            .with_permissions(vec![])
            .with_inherit_from(vec![format!("{}.{}", ORG_ID_ALPHA, ROLE_ALPHA_BUYER)])
            .build()
            .expect("Unable to build Role");
        let role_address = compute_role_address(ROLE_BETA_BUYER, ORG_ID_BETA);
        context
            .set_state_entry(role_address, role_to_bytes(role))
            .expect("Unable to set Role in state");
        // Add the Agent to state, with the beta buyer role
        let agent = AgentBuilder::new()
            .with_org_id(ORG_ID_BETA.to_string())
            .with_public_key(PUBLIC_KEY_BETA.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_BETA_BUYER.to_string()])
            .build()
            .expect("Unable to build Agent");
        let agent_address = compute_agent_address(PUBLIC_KEY_BETA);
        context
            .set_state_entry(agent_address, agent_to_bytes(agent))
            .expect("Unable to set Agent in state");
        // Create the Workflow
        let workflow = get_workflow();
        let subworkflow = workflow
            .subworkflow("po")
            .expect("Unable to get po subworkflow");
        let state = subworkflow
            .state("create")
            .expect("Unable to get create state from subworkflow");

        let result = perm_checker
            .check_permission_with_workflow(
                PERM_CAN_CREATE_PO,
                PUBLIC_KEY_BETA,
                ORG_ID_ALPHA,
                state,
                "issued",
            )
            .expect("Unable to check permission with workflow");
        assert!(!result);
    }

    fn get_workflow() -> Workflow {
        Workflow::new(vec![default_sub_workflow()])
    }

    fn default_sub_workflow() -> SubWorkflow {
        let create = {
            let mut buyer = PermissionAlias::new("po::buyer");
            buyer.add_permission("can-create-po");
            buyer.add_permission("can-create-po-version");
            buyer.add_permission("can-transition-issued");
            buyer.add_transition("issued");

            let mut seller = PermissionAlias::new("po::seller");
            seller.add_permission("can-create-po-version");
            buyer.add_permission("can-transition-issued");
            buyer.add_transition("issued");

            WorkflowStateBuilder::new("create")
                .add_transition("issued")
                .add_permission_alias(buyer)
                .add_permission_alias(seller)
                .build()
        };

        let issued = {
            let mut buyer = PermissionAlias::new("po::buyer");
            buyer.add_permission("can-create-po-version");
            buyer.add_permission("can-update-po-version");
            buyer.add_permission("can-transition-closed");
            buyer.add_transition("closed");

            let mut seller = PermissionAlias::new("po::seller");
            seller.add_permission("can-create-po-version");
            seller.add_permission("can-update-po-version");
            seller.add_permission("can-transition-confirmed");
            seller.add_transition("confirmed");

            WorkflowStateBuilder::new("issued")
                .add_transition("confirmed")
                .add_transition("closed")
                .add_permission_alias(buyer)
                .add_permission_alias(seller)
                .build()
        };

        let confirmed = {
            let mut buyer = PermissionAlias::new("po::buyer");
            buyer.add_permission("can-create-po-version");
            buyer.add_permission("can-transition-issued");
            buyer.add_transition("issued");

            let mut seller = PermissionAlias::new("po::seller");
            seller.add_permission("can-create-po-version");
            seller.add_permission("can-transition-closed");
            seller.add_transition("confirmed");

            WorkflowStateBuilder::new("confirmed")
                .add_transition("issued")
                .add_transition("closed")
                .add_permission_alias(buyer)
                .add_permission_alias(seller)
                .build()
        };

        let closed = {
            let buyer = PermissionAlias::new("po::buyer");
            let seller = PermissionAlias::new("po::seller");

            WorkflowStateBuilder::new("closed")
                .add_permission_alias(buyer)
                .add_permission_alias(seller)
                .build()
        };

        SubWorkflowBuilder::new("po")
            .add_state(create)
            .add_state(issued)
            .add_state(confirmed)
            .add_state(closed)
            .add_starting_state("create")
            .build()
    }
}
