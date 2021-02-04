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

use crate::pike::addressing::compute_agent_address;
use crate::protocol::pike::state::{Agent, AgentList};
use crate::protos::{FromBytes, ProtoConversionError};

#[derive(Debug)]
pub enum PermissionCheckerError {
    /// Returned for an error originating at the TransactionContext.
    Context(ContextError),
    /// Returned for an invalid agent public key.
    InvalidPublicKey(String),
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
            PermissionCheckerError::ProtoConversion(ref e) => e.fmt(f),
        }
    }
}

impl Error for PermissionCheckerError {
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            PermissionCheckerError::Context(_) => None,
            PermissionCheckerError::InvalidPublicKey(_) => None,
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
                if agent.org_id() != record_owner {
                    return Ok(false);
                }
                Ok(agent.roles().iter().any(|r| r == permission))
            }
            None => Err(PermissionCheckerError::InvalidPublicKey(format!(
                "The signer is not an Agent: {}",
                public_key
            ))),
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::collections::HashMap;

    use crate::protocol::pike::state::{AgentBuilder, AgentListBuilder};
    use crate::protos::IntoBytes;

    const ROLE_A: &str = "Role A";
    const ROLE_B: &str = "Role B";

    const PUBLIC_KEY: &str = "test_public_key";
    const ORG_ID: &str = "test_org";
    const WRONG_ORG_ID: &str = "test_wrong_org";

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
    // Test that if an agent has no roles and Role A is checked, false is returned
    fn test_has_permission_a_has_none() {
        let context = MockTransactionContext::default();
        let pc = PermissionChecker::new(&context);

        let builder = AgentBuilder::new();
        let agent = builder
            .with_org_id(ORG_ID.to_string())
            .with_public_key(PUBLIC_KEY.to_string())
            .with_active(true)
            .build()
            .unwrap();
        let builder = AgentListBuilder::new();
        let agent_list = builder.with_agents(vec![agent.clone()]).build().unwrap();
        let agent_bytes = agent_list.into_bytes().unwrap();
        let agent_address = compute_agent_address(PUBLIC_KEY);
        context.set_state_entry(agent_address, agent_bytes).unwrap();

        let result = pc.has_permission(PUBLIC_KEY, ROLE_A, ORG_ID).unwrap();
        assert!(!result);
    }

    #[test]
    // Test that if an agent has Role A and Role A is checked, true is returned
    fn test_has_permission_a_has_a() {
        let context = MockTransactionContext::default();
        let pc = PermissionChecker::new(&context);

        let builder = AgentBuilder::new();
        let agent = builder
            .with_org_id(ORG_ID.to_string())
            .with_public_key(PUBLIC_KEY.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_A.to_string()])
            .build()
            .unwrap();
        let builder = AgentListBuilder::new();
        let agent_list = builder.with_agents(vec![agent.clone()]).build().unwrap();
        let agent_bytes = agent_list.into_bytes().unwrap();
        let agent_address = compute_agent_address(PUBLIC_KEY);
        context.set_state_entry(agent_address, agent_bytes).unwrap();

        let result = pc.has_permission(PUBLIC_KEY, ROLE_A, ORG_ID).unwrap();
        assert!(result);
    }

    #[test]
    // Test that if an agent has Role A and Role B is checked, false is returned
    fn test_has_permission_b_has_a() {
        let context = MockTransactionContext::default();
        let pc = PermissionChecker::new(&context);

        let builder = AgentBuilder::new();
        let agent = builder
            .with_org_id(ORG_ID.to_string())
            .with_public_key(PUBLIC_KEY.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_A.to_string()])
            .build()
            .unwrap();
        let builder = AgentListBuilder::new();
        let agent_list = builder.with_agents(vec![agent.clone()]).build().unwrap();
        let agent_bytes = agent_list.into_bytes().unwrap();
        let agent_address = compute_agent_address(PUBLIC_KEY);
        context.set_state_entry(agent_address, agent_bytes).unwrap();

        let result = pc.has_permission(PUBLIC_KEY, ROLE_B, ORG_ID).unwrap();
        assert!(!result);
    }

    #[test]
    // Test that if an agent has Roles A and B and Role A is checked, true is returned
    fn test_has_permission_a_has_ab() {
        let context = MockTransactionContext::default();
        let pc = PermissionChecker::new(&context);

        let builder = AgentBuilder::new();
        let agent = builder
            .with_org_id(ORG_ID.to_string())
            .with_public_key(PUBLIC_KEY.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_A.to_string(), ROLE_B.to_string()])
            .build()
            .unwrap();
        let builder = AgentListBuilder::new();
        let agent_list = builder.with_agents(vec![agent.clone()]).build().unwrap();
        let agent_bytes = agent_list.into_bytes().unwrap();
        let agent_address = compute_agent_address(PUBLIC_KEY);
        context.set_state_entry(agent_address, agent_bytes).unwrap();

        let result = pc.has_permission(PUBLIC_KEY, ROLE_A, ORG_ID).unwrap();
        assert!(result);
    }

    #[test]
    // Test that if an agent has Roles A and B and Role B is checked, true is returned
    fn test_has_permission_b_has_ab() {
        let context = MockTransactionContext::default();
        let pc = PermissionChecker::new(&context);

        let builder = AgentBuilder::new();
        let agent = builder
            .with_org_id(ORG_ID.to_string())
            .with_public_key(PUBLIC_KEY.to_string())
            .with_active(true)
            .with_roles(vec![ROLE_A.to_string(), ROLE_B.to_string()])
            .build()
            .unwrap();
        let builder = AgentListBuilder::new();
        let agent_list = builder.with_agents(vec![agent.clone()]).build().unwrap();
        let agent_bytes = agent_list.into_bytes().unwrap();
        let agent_address = compute_agent_address(PUBLIC_KEY);
        context.set_state_entry(agent_address, agent_bytes).unwrap();

        let result = pc.has_permission(PUBLIC_KEY, ROLE_B, ORG_ID).unwrap();
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
            .with_roles(vec![ROLE_A.to_string()])
            .build()
            .unwrap();
        let builder = AgentListBuilder::new();
        let agent_list = builder.with_agents(vec![agent.clone()]).build().unwrap();
        let agent_bytes = agent_list.into_bytes().unwrap();
        let agent_address = compute_agent_address(PUBLIC_KEY);
        context.set_state_entry(agent_address, agent_bytes).unwrap();

        let result = pc.has_permission(PUBLIC_KEY, ROLE_A, WRONG_ORG_ID).unwrap();
        assert!(!result);
    }
}
