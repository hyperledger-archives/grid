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

use crate::pike::addressing::{compute_agent_address, compute_role_address};
use crate::protocol::pike::state::{Agent, AgentList, Role, RoleList};
use crate::protos::{FromBytes, ProtoConversionError};

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
            PermissionCheckerError::InvalidRole(ref msg) => write!(f, "InvalidRole: {}", msg),
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

                if agent.org_id() != record_owner {
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
                    .filter_map(|r| r)
                    .collect();

                Ok(self.check_roles_for_permission(
                    &permission,
                    &agent_roles,
                    &record_owner,
                    &agent.org_id(),
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
                    .filter_map(|r| {
                        if r.contains('.') {
                            self.get_role(r, None).ok()
                        } else {
                            self.get_role(r, Some(agent_org_id)).ok()
                        }
                    })
                    .filter_map(|r| r)
                    .collect();

                self.check_roles_for_permission(
                    &permission,
                    &inheriting_roles,
                    &record_owner,
                    &agent_org_id,
                )
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

    fn get_role(
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
                    return Err(PermissionCheckerError::InvalidRole("External roles need to be prefixed with their org ID. Format: <org_id>.<role_name>".to_string()));
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
}
