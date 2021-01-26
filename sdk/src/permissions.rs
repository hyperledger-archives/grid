// Copyright 2019 Cargill Incorporated
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

use crypto::digest::Digest;
use crypto::sha2::Sha512;
use std::error::Error;
use std::fmt;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::WasmSdkError as ContextError;
        use sabre_sdk::TransactionContext;
    } else {
        use sawtooth_sdk::processor::handler::ContextError;
        use sawtooth_sdk::processor::handler::TransactionContext;
    }
}

use crate::protocol::pike::state::{Agent, AgentList, Role, RoleList};
use crate::protos::{FromBytes, ProtoConversionError};

const PIKE_NAMESPACE: &str = "621dee05";
const PIKE_AGENT_RESOURCE: &str = "00";
const PIKE_ROLE_RESOURCE: &str = "02";

fn compute_agent_address(public_key: &str) -> String {
    let mut sha = Sha512::new();
    sha.input(public_key.as_bytes());
    String::from(PIKE_NAMESPACE) + PIKE_AGENT_RESOURCE + &sha.result_str()[..60].to_string()
}

fn compute_role_address(public_key: &str) -> String {
    let mut sha = Sha512::new();
    sha.input(public_key.as_bytes());
    String::from(PIKE_NAMESPACE) + PIKE_ROLE_RESOURCE + &sha.result_str()[..60].to_string()
}

#[derive(Debug)]
pub enum PermissionCheckerError {
    /// Returned for an error originating at the TransactionContext.
    Context(ContextError),
    /// Returned for an invalid agent public key.
    InvalidPublicKey(String),
    /// Returned for an invalid role.
    InvalidRole(String),
    /// Returned for an error in the protobuf data.
    ProtoConversion(ProtoConversionError),
}

impl fmt::Display for PermissionCheckerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PermissionCheckerError::Context(ref e) => e.fmt(f),
            PermissionCheckerError::InvalidPublicKey(ref msg) => {
                write!(f, "InvalidPublicKey: {}", msg)
            }
            PermissionCheckerError::InvalidRole(ref msg) => {
                write!(f, "InvalidRole: {}", msg)
            }
            PermissionCheckerError::ProtoConversion(ref e) => e.fmt(f),
        }
    }
}

impl Error for PermissionCheckerError {
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            PermissionCheckerError::Context(_) => None,
            PermissionCheckerError::InvalidPublicKey(_) => None,
            PermissionCheckerError::InvalidRole(_) => None,
            PermissionCheckerError::ProtoConversion(ref e) => Some(e),
        }
    }
}

impl From<ContextError> for PermissionCheckerError {
    fn from(err: ContextError) -> PermissionCheckerError {
        PermissionCheckerError::Context(err)
    }
}

impl From<ProtoConversionError> for PermissionCheckerError {
    fn from(err: ProtoConversionError) -> PermissionCheckerError {
        PermissionCheckerError::ProtoConversion(err)
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

    /// Checks whether an agent with a given public key has a certain permission.
    ///
    /// # Arguments
    ///
    /// * `public_key` - Public key of a Pike agent.
    /// * `permission` - Permission string to be checked.
    /// * `resource_owner` - The Pike org ID of the resource to check permissions for.
    ///
    pub fn has_permission(
        &self,
        public_key: &str,
        permission: &str,
        resource_owner: &str,
    ) -> Result<bool, PermissionCheckerError> {
        let agent = self.get_agent(public_key)?;
        match agent {
            Some(agent) => {
                let agent_org_id = agent.org_id();
                if !agent.active() {
                    return Err(PermissionCheckerError::InvalidPublicKey(format!(
                        "The signer is not an active agent: {}",
                        public_key
                    )));
                }
                let agent_roles: Vec<Role> = agent
                    .roles()
                    .iter()
                    .filter_map(|r| self.get_role(r).ok())
                    .filter_map(|r| r)
                    .collect();
                Ok(self.check_roles_for_permission(
                    &permission,
                    &agent_roles,
                    &resource_owner,
                    &agent_org_id,
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
        resource_owner: &str,
        agent_org_id: &str,
    ) -> bool {
        roles.iter().any(|r| {
            if r.permissions().iter().any(|p| p == permission) {
                if resource_owner == r.org_id() {
                    agent_org_id == r.org_id()
                        || r.allowed_organizations()
                            .iter()
                            .any(|org| org == agent_org_id)
                } else {
                    if r.inherit_from().is_empty() {
                        return false;
                    }
                    let inheriting_roles: Vec<Role> = r
                        .inherit_from()
                        .iter()
                        .filter_map(|r| self.get_role(r).ok())
                        .filter_map(|r| r)
                        .collect();
                    self.check_roles_for_permission(
                        &permission,
                        &inheriting_roles,
                        &resource_owner,
                        &agent_org_id,
                    )
                }
            } else {
                false
            }
        })
    }

    fn get_agent(&self, public_key: &str) -> Result<Option<Agent>, PermissionCheckerError> {
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

    fn get_role(&self, name: &str) -> Result<Option<Role>, PermissionCheckerError> {
        let address = compute_role_address(name);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let role_list = RoleList::from_bytes(packed.as_slice())?;
                for role in role_list.roles() {
                    if role.name() == name {
                        return Ok(Some(role.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
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

    const PUBLIC_KEY_ALPHA: &str = "alpha_agent_public_key";
    const PUBLIC_KEY_BETA: &str = "beta_agent_public_key";
    const PUBLIC_KEY_GAMMA_1: &str = "gamma_agent_public_key_1";
    const PUBLIC_KEY_GAMMA_2: &str = "gamma_agent_public_key_2";

    const ORG_ID_ALPHA: &str = "alpha";
    const ORG_ID_BETA: &str = "beta";
    const ORG_ID_GAMMA: &str = "gamma";
    const ORG_ID_DELTA: &str = "delta";

    const PERM_CAN_DRIVE: &str = "tankops::can-drive";
    const PERM_CAN_TURN_TURRET: &str = "tankops::can-turn-turret";
    const PERM_CAN_FIRE: &str = "tankops::can-fire";
    const PERM_CAN_DECOMMISSION: &str = "tankops::can-decommission";

    const ROLE_ALPHA_INSPECTOR: &str = "alpha.Inspector";
    const ROLE_ALPHA_DRIVER: &str = "alpha.Driver";
    const ROLE_BETA_DRIVER: &str = "beta.Driver";
    const ROLE_GAMMA_NAVIGATOR: &str = "gamma.Navigator";
    const ROLE_GAMMA_BLASTER: &str = "gamma.Blaster";
    const ROLE_DELTA_TANK_OPERATOR: &str = "delta.TankOperator";

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
            .has_permission(PUBLIC_KEY_ALPHA, PERM_CAN_DECOMMISSION, ORG_ID_ALPHA)
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
            .build()
            .unwrap();

        let role_address = compute_role_address(ROLE_ALPHA_INSPECTOR);
        context
            .set_state_entry(role_address, role_to_bytes(role))
            .unwrap();

        let agent_builder = AgentBuilder::new();
        let agent = agent_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_public_key(PUBLIC_KEY_ALPHA.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_ALPHA_INSPECTOR.to_string()])
            .build()
            .unwrap();

        let agent_address = compute_agent_address(PUBLIC_KEY_ALPHA);
        context
            .set_state_entry(agent_address, agent_to_bytes(agent))
            .unwrap();

        let result = pc
            .has_permission(PUBLIC_KEY_ALPHA, PERM_CAN_DECOMMISSION, ORG_ID_ALPHA)
            .unwrap();
        assert!(!result);
    }

    /// has_permission() returns true if the agent has a role with the given
    /// permission.
    #[test]
    fn test_alpha_can_decommission() {
        let context = MockTransactionContext::default();
        let pc = PermissionChecker::new(&context);

        let role_builder = RoleBuilder::new();
        let role = role_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_name(ROLE_ALPHA_INSPECTOR.to_string())
            .with_permissions(vec![PERM_CAN_DECOMMISSION.to_string()])
            .build()
            .unwrap();

        let role_address = compute_role_address(ROLE_ALPHA_INSPECTOR);
        context
            .set_state_entry(role_address, role_to_bytes(role))
            .unwrap();

        let agent_builder = AgentBuilder::new();
        let agent = agent_builder
            .with_org_id(ORG_ID_ALPHA.to_string())
            .with_public_key(PUBLIC_KEY_ALPHA.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_ALPHA_INSPECTOR.to_string()])
            .build()
            .unwrap();

        let agent_address = compute_agent_address(PUBLIC_KEY_ALPHA);
        context
            .set_state_entry(agent_address, agent_to_bytes(agent))
            .unwrap();

        let result = pc
            .has_permission(PUBLIC_KEY_ALPHA, PERM_CAN_DECOMMISSION, ORG_ID_ALPHA)
            .unwrap();
        assert!(result);
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

        let role_address = compute_role_address(ROLE_ALPHA_DRIVER);
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

        let result_drive = pc
            .has_permission(PUBLIC_KEY_ALPHA, PERM_CAN_DRIVE, ORG_ID_ALPHA)
            .unwrap();
        assert!(result_drive);

        let result_fire = pc
            .has_permission(PUBLIC_KEY_ALPHA, PERM_CAN_FIRE, ORG_ID_ALPHA)
            .unwrap();
        assert!(result_fire);

        let result_turn = pc
            .has_permission(PUBLIC_KEY_ALPHA, PERM_CAN_TURN_TURRET, ORG_ID_ALPHA)
            .unwrap();
        assert!(result_turn);
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

        let alpha_role_address = compute_role_address(ROLE_ALPHA_DRIVER);
        context
            .set_state_entry(alpha_role_address, role_to_bytes(alpha_role))
            .unwrap();

        let beta_role_builder = RoleBuilder::new();
        let beta_role = beta_role_builder
            .with_org_id(ORG_ID_BETA.to_string())
            .with_name(ROLE_BETA_DRIVER.to_string())
            .with_permissions(vec![
                PERM_CAN_DRIVE.to_string(),
                PERM_CAN_FIRE.to_string(),
                PERM_CAN_TURN_TURRET.to_string(),
            ])
            .with_inherit_from(vec![ROLE_ALPHA_DRIVER.to_string()])
            .build()
            .unwrap();

        let beta_role_address = compute_role_address(ROLE_BETA_DRIVER);
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

        let result_drive = pc
            .has_permission(PUBLIC_KEY_BETA, PERM_CAN_DRIVE, ORG_ID_ALPHA)
            .unwrap();
        assert!(result_drive);

        let result_fire = pc
            .has_permission(PUBLIC_KEY_BETA, PERM_CAN_FIRE, ORG_ID_ALPHA)
            .unwrap();
        assert!(result_fire);

        let result_turn = pc
            .has_permission(PUBLIC_KEY_BETA, PERM_CAN_TURN_TURRET, ORG_ID_ALPHA)
            .unwrap();
        assert!(result_turn);
    }

    /// has_permission() returns false if the agent has a role with multiple
    /// given permissions but the agent's org has not been delegated those
    /// permissions by the record owner.
    #[test]
    fn test_beta_multiple_perms_not_allowed() {
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
            .build()
            .unwrap();

        let alpha_role_address = compute_role_address(ROLE_ALPHA_DRIVER);
        context
            .set_state_entry(alpha_role_address, role_to_bytes(alpha_role))
            .unwrap();

        let beta_role_builder = RoleBuilder::new();
        let beta_role = beta_role_builder
            .with_org_id(ORG_ID_BETA.to_string())
            .with_name(ROLE_BETA_DRIVER.to_string())
            .with_permissions(vec![
                PERM_CAN_DRIVE.to_string(),
                PERM_CAN_FIRE.to_string(),
                PERM_CAN_TURN_TURRET.to_string(),
            ])
            .with_inherit_from(vec![ROLE_ALPHA_DRIVER.to_string()])
            .build()
            .unwrap();

        let role_address = compute_role_address(ROLE_BETA_DRIVER);
        context
            .set_state_entry(role_address, role_to_bytes(beta_role))
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

        let result_drive = pc
            .has_permission(PUBLIC_KEY_BETA, PERM_CAN_DRIVE, ORG_ID_ALPHA)
            .unwrap();
        assert!(!result_drive);

        let result_fire = pc
            .has_permission(PUBLIC_KEY_BETA, PERM_CAN_FIRE, ORG_ID_ALPHA)
            .unwrap();
        assert!(!result_fire);

        let result_turn = pc
            .has_permission(PUBLIC_KEY_BETA, PERM_CAN_TURN_TURRET, ORG_ID_ALPHA)
            .unwrap();
        assert!(!result_turn);
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

        let alpha_role_address = compute_role_address(ROLE_ALPHA_DRIVER);
        context
            .set_state_entry(alpha_role_address, role_to_bytes(alpha_role))
            .unwrap();

        let gamma_role_navigator_builder = RoleBuilder::new();
        let gamma_role_navigator = gamma_role_navigator_builder
            .with_org_id(ORG_ID_GAMMA.to_string())
            .with_name(ROLE_GAMMA_NAVIGATOR.to_string())
            .with_permissions(vec![PERM_CAN_DRIVE.to_string()])
            .with_inherit_from(vec![ROLE_ALPHA_DRIVER.to_string()])
            .build()
            .unwrap();

        let gamma_role_navigator_address = compute_role_address(ROLE_GAMMA_NAVIGATOR);
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

        let result_drive = pc
            .has_permission(PUBLIC_KEY_GAMMA_1, PERM_CAN_DRIVE, ORG_ID_ALPHA)
            .unwrap();
        assert!(result_drive);

        let result_fire = pc
            .has_permission(PUBLIC_KEY_GAMMA_1, PERM_CAN_FIRE, ORG_ID_ALPHA)
            .unwrap();
        assert!(!result_fire);

        let result_turn = pc
            .has_permission(PUBLIC_KEY_GAMMA_1, PERM_CAN_TURN_TURRET, ORG_ID_ALPHA)
            .unwrap();
        assert!(!result_turn);

        let gamma_role_blaster_builder = RoleBuilder::new();
        let gamma_role_blaster = gamma_role_blaster_builder
            .with_org_id(ORG_ID_GAMMA.to_string())
            .with_name(ROLE_GAMMA_BLASTER.to_string())
            .with_permissions(vec![PERM_CAN_FIRE.to_string()])
            .with_inherit_from(vec![ROLE_ALPHA_DRIVER.to_string()])
            .build()
            .unwrap();

        let gamma_role_blaster_address = compute_role_address(ROLE_GAMMA_BLASTER);
        context
            .set_state_entry(
                gamma_role_blaster_address,
                role_to_bytes(gamma_role_blaster),
            )
            .unwrap();

        let agent_builder = AgentBuilder::new();
        let agent = agent_builder
            .with_org_id(ORG_ID_GAMMA.to_string())
            .with_public_key(PUBLIC_KEY_GAMMA_2.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_GAMMA_BLASTER.to_string()])
            .build()
            .unwrap();

        let agent_address = compute_agent_address(PUBLIC_KEY_GAMMA_2);
        context
            .set_state_entry(agent_address, agent_to_bytes(agent))
            .unwrap();

        let result_drive = pc
            .has_permission(PUBLIC_KEY_GAMMA_2, PERM_CAN_DRIVE, ORG_ID_ALPHA)
            .unwrap();
        assert!(!result_drive);

        let result_fire = pc
            .has_permission(PUBLIC_KEY_GAMMA_2, PERM_CAN_FIRE, ORG_ID_ALPHA)
            .unwrap();
        assert!(result_fire);

        let result_turn = pc
            .has_permission(PUBLIC_KEY_GAMMA_2, PERM_CAN_TURN_TURRET, ORG_ID_ALPHA)
            .unwrap();
        assert!(!result_turn);
    }

    /// has_permission() can properly check permissions for a role that inherits
    /// from multiple roles with different permissions. The check will return
    /// true if the inherited role from the record owner contains the permission
    /// in question, and false if it does not.
    #[test]
    fn test_beta_multiple_inherits() {
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

        let alpha_role_address = compute_role_address(ROLE_ALPHA_DRIVER);
        context
            .set_state_entry(alpha_role_address, role_to_bytes(alpha_role))
            .unwrap();

        let delta_role_builder = RoleBuilder::new();
        let delta_role = delta_role_builder
            .with_org_id(ORG_ID_DELTA.to_string())
            .with_name(ROLE_DELTA_TANK_OPERATOR.to_string())
            .with_permissions(vec![
                PERM_CAN_DECOMMISSION.to_string(),
                PERM_CAN_DRIVE.to_string(),
                PERM_CAN_FIRE.to_string(),
                PERM_CAN_TURN_TURRET.to_string(),
            ])
            .with_allowed_organizations(vec![ORG_ID_BETA.to_string()])
            .build()
            .unwrap();

        let delta_role_address = compute_role_address(ROLE_DELTA_TANK_OPERATOR);
        context
            .set_state_entry(delta_role_address, role_to_bytes(delta_role))
            .unwrap();

        let beta_role_builder = RoleBuilder::new();
        let beta_role = beta_role_builder
            .with_org_id(ORG_ID_BETA.to_string())
            .with_name(ROLE_BETA_DRIVER.to_string())
            .with_permissions(vec![
                PERM_CAN_DECOMMISSION.to_string(),
                PERM_CAN_DRIVE.to_string(),
                PERM_CAN_FIRE.to_string(),
                PERM_CAN_TURN_TURRET.to_string(),
            ])
            .with_inherit_from(vec![
                ROLE_ALPHA_DRIVER.to_string(),
                ROLE_DELTA_TANK_OPERATOR.to_string(),
            ])
            .build()
            .unwrap();

        let beta_role_address = compute_role_address(ROLE_BETA_DRIVER);
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

        let alpha_result_decommission = pc
            .has_permission(PUBLIC_KEY_BETA, PERM_CAN_DECOMMISSION, ORG_ID_ALPHA)
            .unwrap();
        assert!(!alpha_result_decommission);

        let alpha_result_drive = pc
            .has_permission(PUBLIC_KEY_BETA, PERM_CAN_DRIVE, ORG_ID_ALPHA)
            .unwrap();
        assert!(alpha_result_drive);

        let alpha_result_fire = pc
            .has_permission(PUBLIC_KEY_BETA, PERM_CAN_FIRE, ORG_ID_ALPHA)
            .unwrap();
        assert!(alpha_result_fire);

        let alpha_result_turn = pc
            .has_permission(PUBLIC_KEY_BETA, PERM_CAN_TURN_TURRET, ORG_ID_ALPHA)
            .unwrap();
        assert!(alpha_result_turn);

        let delta_result_decommission = pc
            .has_permission(PUBLIC_KEY_BETA, PERM_CAN_DECOMMISSION, ORG_ID_DELTA)
            .unwrap();
        assert!(delta_result_decommission);

        let delta_result_drive = pc
            .has_permission(PUBLIC_KEY_BETA, PERM_CAN_DRIVE, ORG_ID_DELTA)
            .unwrap();
        assert!(delta_result_drive);

        let delta_result_fire = pc
            .has_permission(PUBLIC_KEY_BETA, PERM_CAN_FIRE, ORG_ID_DELTA)
            .unwrap();
        assert!(delta_result_fire);

        let delta_result_turn = pc
            .has_permission(PUBLIC_KEY_BETA, PERM_CAN_TURN_TURRET, ORG_ID_DELTA)
            .unwrap();
        assert!(delta_result_turn);
    }
}
