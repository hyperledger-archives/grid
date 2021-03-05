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

use grid_sdk::{
    permissions::PermissionChecker,
    pike::addressing::{
        compute_agent_address, compute_organization_address, compute_role_address, PIKE_NAMESPACE,
    },
    protocol::pike::state::{Role, RoleBuilder, RoleList, RoleListBuilder},
    protos::{
        pike_payload::{
            CreateAgentAction, CreateOrganizationAction, CreateRoleAction, DeleteRoleAction,
            PikePayload, PikePayload_Action as Action, UpdateAgentAction, UpdateOrganizationAction,
            UpdateRoleAction,
        },
        pike_state::{Agent, AgentList, Organization, OrganizationList},
        FromBytes, IntoBytes,
    },
};

use crate::permissions::{permission_to_perm_string, Permission};

pub struct PikeTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

pub struct PikeState<'a> {
    context: &'a dyn TransactionContext,
}

impl<'a> PikeState<'a> {
    pub fn new(context: &'a dyn TransactionContext) -> Self {
        Self { context }
    }

    pub fn get_role(&self, name: &str, org_id: &str) -> Result<Option<Role>, ApplyError> {
        let address = compute_role_address(name, org_id);
        match self.context.get_state_entry(&address)? {
            Some(packed) => {
                let roles: RoleList = match RoleList::from_bytes(packed.as_slice()) {
                    Ok(role) => role,
                    Err(err) => {
                        return Err(ApplyError::InvalidTransaction(format!(
                            "Cannot deserialize role list: {:?}",
                            err,
                        )));
                    }
                };

                for role in roles.roles() {
                    if role.name() == name && role.org_id() == org_id {
                        return Ok(Some(role.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_role(&self, role: Role) -> Result<(), ApplyError> {
        let address = compute_role_address(&role.name(), &role.org_id());
        let mut roles = match self.context.get_state_entry(&address)? {
            Some(packed) => match RoleList::from_bytes(packed.as_slice()) {
                Ok(role_list) => role_list.roles().to_vec(),
                Err(err) => {
                    return Err(ApplyError::InternalError(format!(
                        "Cannot deserialize role list: {:?}",
                        err
                    )));
                }
            },
            None => vec![],
        };

        let mut index = None;
        for (i, r) in roles.iter().enumerate() {
            if role.name() == r.name() && role.org_id() == r.org_id() {
                index = Some(i);
                break;
            }
        }

        if let Some(i) = index {
            roles.remove(i);
        }

        roles.push(role);
        roles.sort_by_key(|role| role.name().to_string());
        let role_list = RoleListBuilder::new()
            .with_roles(roles)
            .build()
            .map_err(|err| {
                ApplyError::InvalidTransaction(format!("Cannot build role list: {:?}", err))
            })?;

        let serialized = match role_list.into_bytes() {
            Ok(serialized) => serialized,
            Err(err) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Cannot serialize role list: {:?}",
                    err
                )));
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn remove_role(&mut self, name: &str, org_id: &str) -> Result<(), ApplyError> {
        let address = compute_role_address(&name, &org_id);
        let roles = match self.context.get_state_entry(&address)? {
            Some(packed) => match RoleList::from_bytes(packed.as_slice()) {
                Ok(role_list) => role_list.roles().to_vec(),
                Err(err) => {
                    return Err(ApplyError::InternalError(format!(
                        "Cannot deserialize role list: {:?}",
                        err
                    )));
                }
            },
            None => vec![],
        };

        let filtered_roles = roles
            .into_iter()
            .filter(|role| role.name() != name && role.org_id() != org_id)
            .collect::<Vec<_>>();

        if filtered_roles.is_empty() {
            self.context
                .delete_state_entries(&[address])
                .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        } else {
            let role_list = RoleListBuilder::new()
                .with_roles(filtered_roles)
                .build()
                .map_err(|err| {
                    ApplyError::InvalidTransaction(format!("Cannot build role list: {:?}", err))
                })?;

            let serialized = match role_list.into_bytes() {
                Ok(serialized) => serialized,
                Err(_) => {
                    return Err(ApplyError::InternalError(String::from(
                        "Cannot serialize role list",
                    )));
                }
            };

            self.context
                .set_state_entry(address, serialized)
                .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        }

        Ok(())
    }

    pub fn get_agent(&mut self, public_key: &str) -> Result<Option<Agent>, ApplyError> {
        let address = compute_agent_address(public_key);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let agents: AgentList = match protobuf::Message::parse_from_bytes(packed.as_slice())
                {
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
        let address = compute_agent_address(public_key);
        let d = self.context.get_state_entry(&address)?;
        let mut agent_list = match d {
            Some(packed) => match protobuf::Message::parse_from_bytes(packed.as_slice()) {
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
        for (i, agent) in agents.iter().enumerate() {
            if agent.public_key == public_key {
                index = Some(i);
                break;
            }
        }

        if let Some(i) = index {
            agent_list.agents.remove(i);
        }
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
        let address = compute_organization_address(id);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let orgs: OrganizationList =
                    match protobuf::Message::parse_from_bytes(packed.as_slice()) {
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
        let address = compute_organization_address(id);
        let d = self.context.get_state_entry(&address)?;
        let mut organization_list = match d {
            Some(packed) => match protobuf::Message::parse_from_bytes(packed.as_slice()) {
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
        for (i, organization) in organizations.iter().enumerate() {
            if organization.org_id == id {
                index = Some(i);
                break;
            }
        }

        if let Some(i) = index {
            organization_list.organizations.remove(i);
        }
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
    #[allow(clippy::new_without_default)]
    pub fn new() -> PikeTransactionHandler {
        PikeTransactionHandler {
            family_name: "pike".to_string(),
            family_versions: vec!["2".to_string()],
            namespaces: vec![PIKE_NAMESPACE.to_string()],
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
        let payload: PikePayload = protobuf::Message::parse_from_bytes(request.get_payload())
            .map_err(|_| ApplyError::InternalError("Failed to parse payload".into()))?;

        let signer = request.get_header().get_signer_public_key();
        let mut state = PikeState::new(context);
        let perm_checker = PermissionChecker::new(context);

        info!("Pike Payload {:?}", payload.get_action(),);

        match payload.action {
            Action::CREATE_AGENT => create_agent(
                payload.get_create_agent(),
                signer,
                &mut state,
                &perm_checker,
            ),
            Action::UPDATE_AGENT => update_agent(
                payload.get_update_agent(),
                signer,
                &mut state,
                &perm_checker,
            ),
            Action::CREATE_ORGANIZATION => {
                create_org(payload.get_create_organization(), signer, &mut state)
            }
            Action::UPDATE_ORGANIZATION => update_org(
                payload.get_update_organization(),
                signer,
                &mut state,
                &perm_checker,
            ),
            Action::CREATE_ROLE => {
                create_role(payload.get_create_role(), signer, &mut state, &perm_checker)
            }
            Action::UPDATE_ROLE => {
                update_role(payload.get_update_role(), signer, &mut state, &perm_checker)
            }
            Action::DELETE_ROLE => {
                delete_role(payload.get_delete_role(), signer, &mut state, &perm_checker)
            }
            _ => Err(ApplyError::InvalidTransaction("Invalid action".into())),
        }
    }
}

fn create_role(
    payload: &CreateRoleAction,
    signer: &str,
    state: &mut PikeState,
    perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    if payload.get_org_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Organization ID required".into(),
        ));
    }

    if payload.get_name().is_empty() {
        return Err(ApplyError::InvalidTransaction("Name required".into()));
    }

    if payload.get_description().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Description required".into(),
        ));
    }

    let agent = match state.get_agent(signer)? {
        Some(agent) => agent,
        None => {
            return Err(ApplyError::InvalidTransaction(format!(
                "The signer is not an Agent: {}",
                signer
            )));
        }
    };

    let name = &payload.get_name();

    if name.contains('.') {
        return Err(ApplyError::InvalidTransaction(
            "Role name is not properly formatted. Roles may not contain the '.' character. This is used to reference roles from outside organizations"
                .to_string(),
        ));
    }

    check_permission(
        perm_checker,
        signer,
        &permission_to_perm_string(Permission::CanCreateRoles),
        &agent.org_id,
    )?;

    match state.get_role(payload.get_name(), payload.get_org_id()) {
        Ok(None) => (),
        Ok(Some(_)) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Role already exists: {}",
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

    let role_builder = RoleBuilder::new();
    let role = role_builder
        .with_org_id(payload.get_org_id().to_string())
        .with_name(payload.get_name().to_string())
        .with_description(payload.get_description().to_string())
        .with_permissions(payload.get_permissions().to_vec())
        .with_active(payload.get_active())
        .with_allowed_organizations(payload.get_allowed_organizations().to_vec())
        .with_inherit_from(payload.get_inherit_from().to_vec())
        .build()
        .map_err(|err| ApplyError::InvalidTransaction(format!("Cannot build role: {}", err)))?;

    state
        .set_role(role)
        .map_err(|e| ApplyError::InternalError(format!("Failed to create role: {:?}", e)))
}

fn update_role(
    payload: &UpdateRoleAction,
    signer: &str,
    state: &mut PikeState,
    perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    if payload.get_org_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Organization ID required".into(),
        ));
    }

    if payload.get_name().is_empty() {
        return Err(ApplyError::InvalidTransaction("Name required".into()));
    }

    if payload.get_description().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Description required".into(),
        ));
    }

    let agent = match state.get_agent(signer)? {
        Some(agent) => agent,
        None => {
            return Err(ApplyError::InvalidTransaction(format!(
                "The signer is not an Agent: {}",
                signer
            )));
        }
    };

    let name = &payload.get_name();

    if name.contains('.') {
        return Err(ApplyError::InvalidTransaction(
            "Role name is not properly formatted. Roles may not contain the '.' character. This is used to reference roles from outside organizations"
                .to_string(),
        ));
    }

    check_permission(
        perm_checker,
        signer,
        &permission_to_perm_string(Permission::CanUpdateRoles),
        &agent.org_id,
    )?;

    match state.get_role(payload.get_name(), payload.get_org_id()) {
        Ok(None) => (),
        Ok(Some(_)) => (),
        Err(err) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Failed to retrieve state: {}",
                err,
            )))
        }
    };

    let role_builder = RoleBuilder::new();
    let role = role_builder
        .with_org_id(payload.get_org_id().to_string())
        .with_name(payload.get_name().to_string())
        .with_description(payload.get_description().to_string())
        .with_permissions(payload.get_permissions().to_vec())
        .with_active(payload.get_active())
        .with_allowed_organizations(payload.get_allowed_organizations().to_vec())
        .with_inherit_from(payload.get_inherit_from().to_vec())
        .build()
        .map_err(|err| ApplyError::InvalidTransaction(format!("Cannot build role: {}", err)))?;

    state
        .set_role(role)
        .map_err(|e| ApplyError::InternalError(format!("Failed to update role: {:?}", e)))
}

fn delete_role(
    payload: &DeleteRoleAction,
    signer: &str,
    state: &mut PikeState,
    perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    if payload.get_org_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Organization ID required".into(),
        ));
    }

    if payload.get_name().is_empty() {
        return Err(ApplyError::InvalidTransaction("Name required".into()));
    }

    let agent = match state.get_agent(signer)? {
        Some(agent) => agent,
        None => {
            return Err(ApplyError::InvalidTransaction(format!(
                "The signer is not an Agent: {}",
                signer
            )));
        }
    };

    let name = &payload.get_name();

    if name.contains('.') {
        return Err(ApplyError::InvalidTransaction(
            "Role name is not properly formatted. Roles may not contain the '.' character. This is used to reference roles from outside organizations"
                .to_string(),
        ));
    }

    check_permission(
        perm_checker,
        signer,
        &permission_to_perm_string(Permission::CanDeleteRoles),
        &agent.org_id,
    )?;

    match state.get_role(payload.get_name(), payload.get_org_id()) {
        Ok(None) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Role does not exist: {}",
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

    state
        .remove_role(payload.get_name(), payload.get_org_id())
        .map_err(|e| ApplyError::InternalError(format!("Failed to delete role: {:?}", e)))
}

fn create_agent(
    payload: &CreateAgentAction,
    signer: &str,
    state: &mut PikeState,
    perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    if payload.get_public_key().is_empty() {
        return Err(ApplyError::InvalidTransaction("Public key required".into()));
    }

    if payload.get_org_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Organization ID required".into(),
        ));
    }

    check_permission(
        perm_checker,
        signer,
        &permission_to_perm_string(Permission::CanCreateAgents),
        payload.get_org_id(),
    )?;

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
    perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    if payload.get_public_key().is_empty() {
        return Err(ApplyError::InvalidTransaction("Public key required".into()));
    }

    if payload.get_org_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Organization ID required".into(),
        ));
    }

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

    check_permission(
        perm_checker,
        signer,
        &permission_to_perm_string(Permission::CanUpdateAgents),
        &agent.org_id,
    )?;

    if !payload.get_roles().is_empty() {
        // verify that an admin is not removing the role admin from themselves.
        if signer == payload.get_public_key()
            && !payload.get_roles().iter().any(|role| role == "admin")
        {
            return Err(ApplyError::InvalidTransaction(
                "An admin cannot remove themselves as admin. 'admin' role must be included
                    in the roles list."
                    .to_string(),
            ));
        }

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
    organization.set_metadata(protobuf::RepeatedField::from_vec(
        payload.get_metadata().to_vec(),
    ));
    state.set_organization(payload.get_id(), organization)?;

    state.get_agent(signer).map_err(|e| {
        ApplyError::InternalError(format!("Failed to create organization: {:?}", e))
    })?;

    let role_builder = RoleBuilder::new();
    let role = role_builder
        .with_org_id(payload.get_id().to_string())
        .with_name("admin".to_string())
        .with_description("The Pike administrator".to_string())
        .with_permissions(vec![
            permission_to_perm_string(Permission::CanCreateAgents),
            permission_to_perm_string(Permission::CanUpdateAgents),
            permission_to_perm_string(Permission::CanDeleteAgents),
            permission_to_perm_string(Permission::CanUpdateOrganization),
            permission_to_perm_string(Permission::CanCreateRoles),
            permission_to_perm_string(Permission::CanUpdateRoles),
            permission_to_perm_string(Permission::CanDeleteRoles),
        ])
        .with_active(true)
        .build()
        .unwrap();
    state.set_role(role)?;

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
                "Failed to retrieve state: {}",
                err,
            )))
        }
    };
    let mut agent = Agent::new();
    agent.set_public_key(signer.to_string());
    agent.set_org_id(payload.get_id().to_string());
    agent.set_active(true);
    agent.set_roles(protobuf::RepeatedField::from_vec(vec!["admin".to_string()]));

    state
        .set_agent(signer, agent)
        .map_err(|e| ApplyError::InternalError(format!("Failed to create agent: {:?}", e)))
}

fn update_org(
    payload: &UpdateOrganizationAction,
    signer: &str,
    state: &mut PikeState,
    perm_checker: &PermissionChecker,
) -> Result<(), ApplyError> {
    if payload.get_id().is_empty() {
        return Err(ApplyError::InvalidTransaction(
            "Unique organization ID required".into(),
        ));
    }

    check_permission(
        perm_checker,
        signer,
        &permission_to_perm_string(Permission::CanUpdateOrganization),
        payload.get_id(),
    )?;

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
    if !payload.get_locations().is_empty() {
        organization.set_locations(protobuf::RepeatedField::from_vec(
            payload.get_locations().to_vec(),
        ));
    }
    if !payload.get_metadata().is_empty() {
        organization.set_metadata(protobuf::RepeatedField::from_vec(
            payload.get_metadata().to_vec(),
        ));
    }
    state.set_organization(payload.get_id(), organization)
}

fn check_permission(
    perm_checker: &PermissionChecker,
    signer: &str,
    permission: &str,
    record_owner: &str,
) -> Result<(), ApplyError> {
    match perm_checker.has_permission(signer, permission, record_owner) {
        Ok(true) => Ok(()),
        Ok(false) => Err(ApplyError::InvalidTransaction(format!(
            "The signer \"{}\" does not have the \"{}\" permission for org \"{}\"",
            signer, permission, record_owner
        ))),
        Err(err) => Err(ApplyError::InvalidTransaction(format!(
            "Permission check failed: {}",
            err
        ))),
    }
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
