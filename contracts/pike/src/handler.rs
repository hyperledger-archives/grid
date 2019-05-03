// Copyright 2018 Cargill Incorporated
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
use protobuf;

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

use addresser::{resource_to_byte, Resource};
use grid_sdk::protos::pike_payload::{
    CreateAgentAction, CreateOrganizationAction, PikePayload, PikePayload_Action as Action,
    UpdateAgentAction, UpdateOrganizationAction,
};
use grid_sdk::protos::pike_state::{Agent, AgentList, Organization, OrganizationList};

pub struct PikeTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

const NAMESPACE: &str = "cad11d";

fn compute_address(name: &str, resource: Resource) -> String {
    let mut sha = Sha512::new();
    sha.input(name.as_bytes());

    String::from(NAMESPACE) + &resource_to_byte(resource) + &sha.result_str()[..62].to_string()
}

pub struct PikeState<'a> {
    context: &'a mut TransactionContext,
}

impl<'a> PikeState<'a> {
    pub fn new(context: &'a mut TransactionContext) -> PikeState {
        PikeState { context }
    }

    pub fn get_agent(&mut self, public_key: &str) -> Result<Option<Agent>, ApplyError> {
        let address = compute_address(public_key, Resource::AGENT);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let agents: AgentList = match protobuf::parse_from_bytes(packed.as_slice()) {
                    Ok(agents) => agents,
                    Err(err) => {
                        return Err(ApplyError::InternalError(format!(
                            "Cannot deserialize record container: {:?}",
                            err,
                        )))
                    }
                };

                for agent in agents.get_agents() {
                    if agent.public_key == public_key {
                        return Ok(Some(agent.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_agent(&mut self, public_key: &str, new_agent: Agent) -> Result<(), ApplyError> {
        let address = compute_address(public_key, Resource::AGENT);
        let d = self.context.get_state_entry(&address)?;
        let mut agent_list = match d {
            Some(packed) => match protobuf::parse_from_bytes(packed.as_slice()) {
                Ok(agents) => agents,
                Err(err) => {
                    return Err(ApplyError::InternalError(format!(
                        "Cannot deserialize agent list: {}",
                        err,
                    )))
                }
            },
            None => AgentList::new(),
        };
        // remove old agent if it exists and sort the agents by public key
        let agents = agent_list.get_agents().to_vec();
        let mut index = None;
        let mut count = 0;
        for agent in agents.clone() {
            if agent.public_key == public_key {
                index = Some(count);
                break;
            }
            count = count + 1;
        }

        match index {
            Some(x) => {
                agent_list.agents.remove(x);
            }
            None => (),
        };
        agent_list.agents.push(new_agent);
        agent_list.agents.sort_by_key(|a| a.clone().public_key);
        let serialized = match protobuf::Message::write_to_bytes(&agent_list) {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize agent list",
                )))
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn get_organization(&mut self, id: &str) -> Result<Option<Organization>, ApplyError> {
        let address = compute_address(id, Resource::ORG);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let orgs: OrganizationList = match protobuf::parse_from_bytes(packed.as_slice()) {
                    Ok(orgs) => orgs,
                    Err(err) => {
                        return Err(ApplyError::InternalError(format!(
                            "Cannot deserialize organization list: {:?}",
                            err,
                        )))
                    }
                };

                for org in orgs.get_organizations() {
                    if org.org_id == id {
                        return Ok(Some(org.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_organization(
        &mut self,
        id: &str,
        new_organization: Organization,
    ) -> Result<(), ApplyError> {
        let address = compute_address(id, Resource::ORG);
        let d = self.context.get_state_entry(&address)?;
        let mut organization_list = match d {
            Some(packed) => match protobuf::parse_from_bytes(packed.as_slice()) {
                Ok(orgs) => orgs,
                Err(err) => {
                    return Err(ApplyError::InternalError(format!(
                        "Cannot deserialize organization list: {}",
                        err,
                    )))
                }
            },
            None => OrganizationList::new(),
        };
        // remove old agent if it exists and sort the agents by public key
        let organizations = organization_list.get_organizations().to_vec();
        let mut index = None;
        let mut count = 0;
        for organization in organizations.clone() {
            if organization.org_id == id {
                index = Some(count);
                break;
            }
            count = count + 1;
        }

        match index {
            Some(x) => {
                organization_list.organizations.remove(x);
            }
            None => (),
        };
        organization_list.organizations.push(new_organization);
        organization_list
            .organizations
            .sort_by_key(|o| o.clone().org_id);
        let serialized = match protobuf::Message::write_to_bytes(&organization_list) {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize organization list",
                )))
            }
        };

        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }
}

impl PikeTransactionHandler {
    pub fn new() -> PikeTransactionHandler {
        PikeTransactionHandler {
            family_name: "pike".to_string(),
            family_versions: vec!["0.1".to_string()],
            namespaces: vec![NAMESPACE.to_string()],
        }
    }
}

impl TransactionHandler for PikeTransactionHandler {
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
        let payload = protobuf::parse_from_bytes::<PikePayload>(request.get_payload())
            .map_err(|_| ApplyError::InternalError("Failed to parse payload".into()))?;

        let signer = request.get_header().get_signer_public_key();
        let mut state = PikeState::new(context);

        #[cfg(not(target_arch = "wasm32"))]
        info!(
            "{:?} {:?} {:?}",
            payload.get_action(),
            request.get_header().get_inputs(),
            request.get_header().get_outputs()
        );

        match payload.action {
            Action::CREATE_AGENT => create_agent(payload.get_create_agent(), signer, &mut state),
            Action::UPDATE_AGENT => update_agent(payload.get_update_agent(), signer, &mut state),
            Action::CREATE_ORGANIZATION => {
                create_org(payload.get_create_organization(), signer, &mut state)
            }
            Action::UPDATE_ORGANIZATION => {
                update_org(payload.get_update_organization(), signer, &mut state)
            }
            _ => Err(ApplyError::InvalidTransaction("Invalid action".into())),
        }
    }
}

fn create_agent(
    payload: &CreateAgentAction,
    signer: &str,
    state: &mut PikeState,
) -> Result<(), ApplyError> {
    if payload.get_public_key().is_empty() {
        return Err(ApplyError::InvalidTransaction("Public key required".into()));
    }

    if payload.get_org_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Organization ID required".into(),
        ));
    }

    // verify the signer of the transaction is authorized to create agent
    is_admin(signer, payload.get_org_id(), state)?;

    // Check if agent already exists
    match state.get_agent(payload.get_public_key()) {
        Ok(None) => (),
        Ok(Some(_)) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Agent already exists: {}",
                payload.get_public_key(),
            )))
        }
        Err(err) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Failed to retrieve state: {}",
                err,
            )))
        }
    };

    let mut agent = Agent::new();
    agent.set_public_key(payload.get_public_key().to_string());
    agent.set_org_id(payload.get_org_id().to_string());
    agent.set_active(payload.get_active());
    agent.set_roles(protobuf::RepeatedField::from_vec(
        payload.get_roles().to_vec(),
    ));
    agent.set_metadata(protobuf::RepeatedField::from_vec(
        payload.get_metadata().to_vec(),
    ));

    state
        .set_agent(payload.get_public_key(), agent)
        .map_err(|e| ApplyError::InternalError(format!("Failed to create agent: {:?}", e)))
}

fn update_agent(
    payload: &UpdateAgentAction,
    signer: &str,
    state: &mut PikeState,
) -> Result<(), ApplyError> {
    if payload.get_public_key().is_empty() {
        return Err(ApplyError::InvalidTransaction("Public key required".into()));
    }

    if payload.get_org_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Organization ID required".into(),
        ));
    }
    // verify the signer of the transaction is authorized to update agent
    is_admin(signer, payload.get_org_id(), state)?;

    // make sure agent already exists
    let mut agent = match state.get_agent(payload.get_public_key()) {
        Ok(None) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Agent does not exists: {}",
                payload.get_public_key(),
            )))
        }
        Ok(Some(agent)) => agent,
        Err(err) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Failed to retrieve state: {}",
                err,
            )))
        }
    };

    if !payload.get_roles().is_empty() {
        agent.set_roles(protobuf::RepeatedField::from_vec(
            payload.get_roles().to_vec(),
        ));
    }

    if !payload.get_metadata().is_empty() {
        agent.set_metadata(protobuf::RepeatedField::from_vec(
            payload.get_metadata().to_vec(),
        ));
    }

    if payload.get_active() != agent.get_active() {
        if signer == payload.get_public_key() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Admin may not deactivate themselves: {}",
                signer,
            )));
        }
        agent.set_active(payload.get_active());
    }
    state
        .set_agent(payload.get_public_key(), agent)
        .map_err(|e| ApplyError::InternalError(format!("Failed to create agent: {:?}", e)))
}

fn create_org(
    payload: &CreateOrganizationAction,
    signer: &str,
    state: &mut PikeState,
) -> Result<(), ApplyError> {
    if payload.get_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Unique organization ID required".into(),
        ));
    }

    if payload.get_name().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "organization name required".into(),
        ));
    }

    // Check if the organization already exists
    match state.get_organization(payload.get_id()) {
        Ok(None) => (),
        Ok(Some(_)) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Organization already exists: {}",
                payload.get_id(),
            )))
        }
        Err(err) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Failed to retrievestate: {}",
                err,
            )))
        }
    };
    let mut organization = Organization::new();
    organization.set_org_id(payload.get_id().to_string());
    organization.set_name(payload.get_name().to_string());
    organization.set_address(payload.get_address().to_string());
    organization.set_metadata(protobuf::RepeatedField::from_vec(
        payload.get_metadata().to_vec(),
    ));
    state.set_organization(payload.get_id(), organization)?;

    state.get_agent(signer).map_err(|e| {
        ApplyError::InternalError(format!("Failed to create organization: {:?}", e))
    })?;

    // Check if the agent already exists
    match state.get_agent(signer) {
        Ok(None) => (),
        Ok(Some(_)) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Agent already exists: {}",
                payload.get_id(),
            )))
        }
        Err(err) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Failed to retrievestate: {}",
                err,
            )))
        }
    };
    let mut agent = Agent::new();
    agent.set_public_key(signer.to_string());
    agent.set_org_id(payload.get_id().to_string());
    agent.set_active(true);
    agent.set_roles(protobuf::RepeatedField::from_vec(vec![String::from(
        "admin",
    )]));

    state
        .set_agent(signer, agent)
        .map_err(|e| ApplyError::InternalError(format!("Failed to create agent: {:?}", e)))
}

fn update_org(
    payload: &UpdateOrganizationAction,
    signer: &str,
    state: &mut PikeState,
) -> Result<(), ApplyError> {
    if payload.get_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Unique organization ID required".into(),
        ));
    }

    // verify the signer of the transaction is authorized to update organization
    is_admin(signer, payload.get_id(), state)?;

    // Make sure the organization already exists
    let mut organization = match state.get_organization(payload.get_id()) {
        Ok(None) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Organization does not exist exists: {}",
                payload.get_id(),
            )))
        }
        Ok(Some(org)) => org,
        Err(err) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Failed to retrieve state: {}",
                err,
            )))
        }
    };

    if !payload.get_name().is_empty() {
        organization.set_name(payload.get_name().to_string());
    }
    if !payload.get_address().is_empty() {
        organization.set_address(payload.get_address().to_string());
    }
    if !payload.get_metadata().is_empty() {
        organization.set_metadata(protobuf::RepeatedField::from_vec(
            payload.get_metadata().to_vec(),
        ));
    }
    state.set_organization(payload.get_id(), organization)
}

pub fn is_admin(signer: &str, org_id: &str, state: &mut PikeState) -> Result<(), ApplyError> {
    let admin = match state.get_agent(signer) {
        Ok(None) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Signer is not an agent: {}",
                signer,
            )))
        }
        Ok(Some(admin)) => admin,
        Err(err) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Failed to retrieve state: {}",
                err,
            )))
        }
    };

    if admin.get_org_id() != org_id {
        return Err(ApplyError::InvalidTransaction(format!(
            "Signer is not associated with the organization: {}",
            signer,
        )));
    }
    if !admin.roles.contains(&"admin".to_string()) {
        return Err(ApplyError::InvalidTransaction(format!(
            "Signer is not an admin: {}",
            signer,
        )));
    };

    if !admin.active {
        return Err(ApplyError::InvalidTransaction(format!(
            "Admin is not currently an active agent: {}",
            signer,
        )));
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
// Sabre apply must return a bool
fn apply(
    request: &TpProcessRequest,
    context: &mut dyn TransactionContext,
) -> Result<bool, ApplyError> {
    let handler = PikeTransactionHandler::new();
    match handler.apply(request, context) {
        Ok(_) => Ok(true),
        Err(err) => Err(err),
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub unsafe fn entrypoint(payload: WasmPtr, signer: WasmPtr, signature: WasmPtr) -> i32 {
    execute_entrypoint(payload, signer, signature, apply)
}
