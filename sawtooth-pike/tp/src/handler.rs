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

use protobuf;
use crypto::digest::Digest;
use crypto::sha2::Sha512;
use std::collections::HashMap;

use sawtooth_sdk::processor::handler::ApplyError;
use sawtooth_sdk::processor::handler::TransactionContext;
use sawtooth_sdk::processor::handler::TransactionHandler;
use sawtooth_sdk::messages::processor::TpProcessRequest;

use protos::payload::{CreateAgentAction, CreateOrganizationAction, CreateSmartPermissionAction,
                      DeleteSmartPermissionAction, PikePayload,
                      PikePayload_Action as Action, UpdateAgentAction,
                      UpdateOrganizationAction, UpdateSmartPermissionAction};
use protos::state::{Agent, AgentList, Organization, OrganizationList, SmartPermission,
                    SmartPermissionList};
use addresser::{resource_to_byte, Resource};

pub struct PikeTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

const NAMESPACE: &'static str = "cad11d";

fn compute_address(name: &str, resource: Resource) -> String {
    let mut sha = Sha512::new();
    sha.input(name.as_bytes());

    String::from(NAMESPACE) + &resource_to_byte(resource) + &sha.result_str()[..62].to_string()
}

fn compute_smart_permission_address(org_id: &str, name: &str) -> String {
    let mut sha_org_id = Sha512::new();
    sha_org_id.input(org_id.as_bytes());

    let mut sha_name = Sha512::new();
    sha_name.input(name.as_bytes());

    String::from(NAMESPACE) + &resource_to_byte(Resource::SPF)
        + &sha_org_id.result_str()[..6].to_string() + &sha_name.result_str()[..56].to_string()
}

pub struct PikeState<'a> {
    context: &'a mut TransactionContext,
}

impl<'a> PikeState<'a> {
    pub fn new(context: &'a mut TransactionContext) -> PikeState {
        PikeState { context: context }
    }

    pub fn get_agent(&mut self, public_key: &str) -> Result<Option<Agent>, ApplyError> {
        let address = compute_address(public_key, Resource::AGENT);
        let d = self.context.get_state(vec![address])?;
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
        let d = self.context.get_state(vec![address.clone()])?;
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
        let mut sets = HashMap::new();
        sets.insert(address, serialized);
        self.context
            .set_state(sets)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn get_organization(&mut self, id: &str) -> Result<Option<Organization>, ApplyError> {
        let address = compute_address(id, Resource::ORG);
        let d = self.context.get_state(vec![address])?;
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
        let d = self.context.get_state(vec![address.clone()])?;
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

        let mut sets = HashMap::new();
        sets.insert(address, serialized);
        self.context
            .set_state(sets)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn get_smart_permission(
        &mut self,
        org_id: &str,
        name: &str,
    ) -> Result<Option<SmartPermission>, ApplyError> {
        let address = compute_smart_permission_address(org_id, name);
        let d = self.context.get_state(vec![address])?;
        match d {
            Some(packed) => {
                let smart_permissions: SmartPermissionList =
                    match protobuf::parse_from_bytes(packed.as_slice()) {
                        Ok(smart_permissions) => smart_permissions,
                        Err(err) => {
                            return Err(ApplyError::InternalError(format!(
                                "Cannot deserialize smart permission list: {:?}",
                                err,
                            )))
                        }
                    };

                for smart_permission in smart_permissions.get_smart_permissions() {
                    if smart_permission.name == name {
                        return Ok(Some(smart_permission.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_smart_permission(
        &mut self,
        org_id: &str,
        name: &str,
        new_smart_permission: SmartPermission,
    ) -> Result<(), ApplyError> {
        let address = compute_smart_permission_address(org_id, name);
        let d = self.context.get_state(vec![address.clone()])?;
        let mut smart_permission_list = match d {
            Some(packed) => match protobuf::parse_from_bytes(packed.as_slice()) {
                Ok(smart_permissions) => smart_permissions,
                Err(err) => {
                    return Err(ApplyError::InternalError(format!(
                        "Cannot deserialize smart permission list: {}",
                        err,
                    )))
                }
            },
            None => SmartPermissionList::new(),
        };
        // remove old smart_permission if it exists and sort the smart_permission by name
        let smart_permissions = smart_permission_list.get_smart_permissions().to_vec();
        let mut index = None;
        let mut count = 0;
        for smart_permission in smart_permissions.clone() {
            if smart_permission.name == name {
                index = Some(count);
                break;
            }
            count = count + 1;
        }

        match index {
            Some(x) => {
                smart_permission_list.smart_permissions.remove(x);
            }
            None => (),
        };
        smart_permission_list
            .smart_permissions
            .push(new_smart_permission);
        smart_permission_list
            .smart_permissions
            .sort_by_key(|sp| sp.clone().name);
        let serialized = match protobuf::Message::write_to_bytes(&smart_permission_list) {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize smart permission list",
                )))
            }
        };
        let mut sets = HashMap::new();
        sets.insert(address, serialized);
        self.context
            .set_state(sets)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn delete_smart_permission(&mut self, org_id: &str, name: &str) -> Result<(), ApplyError> {
        let address = compute_smart_permission_address(org_id, name);
        let d = self.context.delete_state(vec![address.clone()])?;
        let deleted: Vec<String> = match d {
            Some(deleted) => deleted.to_vec(),
            None => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot delete smart_permission",
                )))
            }
        };
        if !deleted.contains(&address) {
            return Err(ApplyError::InternalError(String::from(
                "Cannot delete smart_permission",
            )));
        };
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
        return self.family_name.clone();
    }

    fn family_versions(&self) -> Vec<String> {
        return self.family_versions.clone();
    }

    fn namespaces(&self) -> Vec<String> {
        return self.namespaces.clone();
    }

    fn apply(
        &self,
        request: &TpProcessRequest,
        context: &mut TransactionContext,
    ) -> Result<(), ApplyError> {
        let payload = protobuf::parse_from_bytes::<PikePayload>(request.get_payload())
            .map_err(|_| ApplyError::InternalError("Failed to parse payload".into()))?;

        let signer = request.get_header().get_signer_public_key();
        let mut state = PikeState::new(context);

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
            Action::CREATE_SMART_PERMISSION => {
                create_smart_perm(payload.get_create_smart_permission(), signer, &mut state)
            }
            Action::UPDATE_SMART_PERMISSION => {
                update_smart_perm(payload.get_update_smart_permission(), signer, &mut state)
            }
            Action::DELETE_SMART_PERMISSION => {
                delete_smart_perm(payload.get_delete_smart_permission(), signer, &mut state)
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
    state.set_organization(payload.get_id(), organization)?;

    state
        .get_agent(signer)
        .map_err(|e| ApplyError::InternalError(format!("Failed to create organization: {:?}", e)))?;

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
    agent.set_roles(protobuf::RepeatedField::from_vec(vec![
        String::from("admin"),
    ]));

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
    state.set_organization(payload.get_id(), organization)
}

fn create_smart_perm(
    payload: &CreateSmartPermissionAction,
    signer: &str,
    state: &mut PikeState,
) -> Result<(), ApplyError> {
    if payload.get_name().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Smart Permission must have a name".into(),
        ));
    }

    if payload.get_org_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Smart Permission must have an org id".into(),
        ));
    }

    if payload.get_function().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Smart Permission must have a function".into(),
        ));
    }

    // Check if the smart permissions already exists
    match state.get_smart_permission(payload.get_org_id(), payload.get_name()) {
        Ok(None) => (),
        Ok(Some(_)) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Smart Permission already exists: {} ",
                payload.get_name(),
            )))
        }
        Err(err) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Failed to retrieve state: {}",
                err,
            )))
        }
    };

    // verify the signer of the transaction is authorized to create smart permissions
    is_admin(signer, payload.get_org_id(), state)?;

    // Check that organizations exists
    match state.get_organization(payload.get_org_id()) {
        Ok(None) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Organization does not exist exists: {}",
                payload.get_org_id(),
            )))
        }
        Ok(Some(_)) => (),
        Err(err) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Failed to retrieve state: {}",
                err,
            )))
        }
    };

    let mut smart_permission = SmartPermission::new();
    smart_permission.set_org_id(payload.get_org_id().to_string());
    smart_permission.set_name(payload.get_name().to_string());
    smart_permission.set_function(payload.get_function().to_vec());
    state.set_smart_permission(payload.get_org_id(), payload.get_name(), smart_permission)
}

fn update_smart_perm(
    payload: &UpdateSmartPermissionAction,
    signer: &str,
    state: &mut PikeState,
) -> Result<(), ApplyError> {
    if payload.get_name().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Smart Permission must have a name".into(),
        ));
    }

    if payload.get_org_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Smart Permission must have an org id".into(),
        ));
    }

    if payload.get_function().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Smart Permission must have a function".into(),
        ));
    }

    // verify the signer of the transaction is authorized to update smart permissions
    is_admin(signer, payload.get_org_id(), state)?;

    // verify that the smart permission exists
    let mut smart_permission =
        match state.get_smart_permission(payload.get_org_id(), payload.get_name()) {
            Ok(None) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Smart Permission does not exists: {} ",
                    payload.get_name(),
                )))
            }
            Ok(Some(smart_permission)) => smart_permission,
            Err(err) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Failed to retrieve state: {}",
                    err,
                )))
            }
        };

    smart_permission.set_function(payload.get_function().to_vec());
    state.set_smart_permission(payload.get_org_id(), payload.get_name(), smart_permission)
}

fn delete_smart_perm(
    payload: &DeleteSmartPermissionAction,
    signer: &str,
    state: &mut PikeState,
) -> Result<(), ApplyError> {
    if payload.get_name().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Smart Permission must have a name".into(),
        ));
    }

    if payload.get_org_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Smart Permission must have an org id".into(),
        ));
    }

    // verify the signer of the transaction is authorized to delete smart permissions
    is_admin(signer, payload.get_org_id(), state)?;

    // verify that the smart permission exists
    match state.get_smart_permission(payload.get_org_id(), payload.get_name()) {
        Ok(None) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Smart Permission does not exists: {} ",
                payload.get_name(),
            )))
        }
        Ok(Some(_)) => (),
        Err(err) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Failed to retrieve state: {}",
                err,
            )))
        }
    };

    state.delete_smart_permission(payload.get_org_id(), payload.get_name())
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

