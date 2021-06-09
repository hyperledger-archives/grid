// Copyright 2018-2021 Cargill Incorporated
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
    }
}

use grid_sdk::{
    pike::addressing::{
        compute_agent_address, compute_alternate_id_index_entry_address,
        compute_organization_address, compute_role_address,
    },
    protocol::pike::state::{
        AlternateIdIndexEntry, AlternateIdIndexEntryList, AlternateIdIndexEntryListBuilder, Role,
        RoleList, RoleListBuilder,
    },
    protos::{
        pike_state::{Agent, AgentList, Organization, OrganizationList},
        FromBytes, IntoBytes,
    },
};

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
                let role = roles
                    .roles()
                    .iter()
                    .find(|role| role.name() == name && role.org_id() == org_id)
                    .map(ToOwned::to_owned);
                Ok(role)
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
                            "Cannot deserialize agent list: {:?}",
                            err,
                        )))
                    }
                };
                let agent = agents
                    .get_agents()
                    .iter()
                    .find(|agent| agent.public_key == public_key)
                    .map(ToOwned::to_owned);
                Ok(agent)
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
                let org = orgs
                    .get_organizations()
                    .iter()
                    .find(|org| org.org_id == id)
                    .map(ToOwned::to_owned);
                Ok(org)
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
        // remove old organization if it exists and sort the organizations by org ID
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

    pub fn get_alternate_id_index(
        &self,
        id_type: &str,
        id: &str,
    ) -> Result<Option<AlternateIdIndexEntry>, ApplyError> {
        let address = compute_alternate_id_index_entry_address(id_type, id);
        match self.context.get_state_entry(&address)? {
            Some(packed) => {
                let entries: AlternateIdIndexEntryList =
                    match AlternateIdIndexEntryList::from_bytes(packed.as_slice()) {
                        Ok(entry) => entry,
                        Err(err) => {
                            return Err(ApplyError::InvalidTransaction(format!(
                                "Cannot deseralize alternate ID index entry list: {:?}",
                                err,
                            )));
                        }
                    };
                let entry = entries
                    .entries()
                    .iter()
                    .find(|entry| entry.id_type() == id_type && entry.id() == id)
                    .map(ToOwned::to_owned);
                Ok(entry)
            }
            None => Ok(None),
        }
    }

    pub fn set_alternate_id_index(&self, alt_id: AlternateIdIndexEntry) -> Result<(), ApplyError> {
        let address = compute_alternate_id_index_entry_address(alt_id.id_type(), alt_id.id());
        let mut entries = match self.context.get_state_entry(&address)? {
            Some(packed) => match AlternateIdIndexEntryList::from_bytes(packed.as_slice()) {
                Ok(entry_list) => entry_list.entries().to_vec(),
                Err(err) => {
                    return Err(ApplyError::InvalidTransaction(format!(
                        "Cannot deseralize alternate ID index entry list: {:?}",
                        err,
                    )));
                }
            },
            None => vec![],
        };

        let mut index = None;
        for (i, e) in entries.iter().enumerate() {
            if alt_id.id_type() == e.id_type() && alt_id.id() == e.id() {
                index = Some(i);
                break;
            }
        }

        if let Some(i) = index {
            entries.remove(i);
        }

        entries.push(alt_id);
        entries.sort_by_key(|alt_id| alt_id.id().to_string());
        let entry_list = AlternateIdIndexEntryListBuilder::new()
            .with_entries(entries)
            .build()
            .map_err(|err| {
                ApplyError::InvalidTransaction(format!(
                    "Cannot build alternate ID index entry list: {:?}",
                    err
                ))
            })?;

        let serialized = match entry_list.into_bytes() {
            Ok(serialized) => serialized,
            Err(err) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Cannot serialize alternate ID index entry list: {:?}",
                    err
                )));
            }
        };

        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn remove_alternate_id_index(&mut self, id_type: &str, id: &str) -> Result<(), ApplyError> {
        let address = compute_alternate_id_index_entry_address(&id_type, &id);
        let entries = match self.context.get_state_entry(&address)? {
            Some(packed) => match AlternateIdIndexEntryList::from_bytes(packed.as_slice()) {
                Ok(entry_list) => entry_list.entries().to_vec(),
                Err(err) => {
                    return Err(ApplyError::InvalidTransaction(format!(
                        "Cannot serialize alternate ID index entry list: {:?}",
                        err
                    )));
                }
            },
            None => vec![],
        };

        let filtered_entries = entries
            .into_iter()
            .filter(|entry| entry.id_type() != id_type && entry.id() != id)
            .collect::<Vec<_>>();

        if filtered_entries.is_empty() {
            self.context
                .delete_state_entries(&[address])
                .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        } else {
            let entry_list = AlternateIdIndexEntryListBuilder::new()
                .with_entries(filtered_entries)
                .build()
                .map_err(|err| {
                    ApplyError::InvalidTransaction(format!(
                        "Cannot build alternate ID index entry list: {:?}",
                        err
                    ))
                })?;

            let serialized = match entry_list.into_bytes() {
                Ok(serialized) => serialized,
                Err(_) => {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Cannot serialize alternate ID index entry list",
                    )));
                }
            };

            self.context
                .set_state_entry(address, serialized)
                .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        }

        Ok(())
    }
}
