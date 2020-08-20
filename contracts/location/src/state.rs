// Copyright 2020 Cargill Incorporated
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
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
        use sawtooth_sdk::processor::handler::TransactionContext;
    }
}

use grid_sdk::{
    protocol::{
        location::state::{Location, LocationList, LocationListBuilder},
        pike::state::{Agent, AgentList, Organization, OrganizationList},
        schema::state::{Schema, SchemaList},
    },
    protos::{FromBytes, IntoBytes},
};

use crate::addressing::{
    compute_agent_address, compute_gs1_location_address, compute_org_address,
    compute_schema_address,
};

pub struct LocationState<'a> {
    context: &'a dyn TransactionContext,
}

impl<'a> LocationState<'a> {
    pub fn new(context: &'a dyn TransactionContext) -> Self {
        Self { context }
    }

    pub fn get_location(&self, location_id: &str) -> Result<Option<Location>, ApplyError> {
        let address = compute_gs1_location_address(location_id);
        match self.context.get_state_entry(&address)? {
            Some(packed) => {
                let locations: LocationList = match LocationList::from_bytes(packed.as_slice()) {
                    Ok(location) => location,
                    Err(err) => {
                        return Err(ApplyError::InvalidTransaction(format!(
                            "Cannot deserialize location list: {:?}",
                            err,
                        )));
                    }
                };

                for location in locations.locations() {
                    if location.location_id() == location_id {
                        return Ok(Some(location.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_location(&self, location: Location) -> Result<(), ApplyError> {
        let address = compute_gs1_location_address(&location.location_id());
        let mut locations = match self.context.get_state_entry(&address)? {
            Some(packed) => match LocationList::from_bytes(packed.as_slice()) {
                Ok(location_list) => location_list.locations().to_vec(),
                Err(err) => {
                    return Err(ApplyError::InternalError(format!(
                        "Cannot deserialize location list: {:?}",
                        err
                    )));
                }
            },
            None => vec![],
        };

        let mut index = None;
        for (i, l) in locations.iter().enumerate() {
            if location.location_id() == l.location_id() {
                index = Some(i);
                break;
            }
        }

        if let Some(i) = index {
            locations.remove(i);
        }

        locations.push(location);
        locations.sort_by_key(|location| location.location_id().to_string());
        let location_list = LocationListBuilder::new()
            .with_locations(locations)
            .build()
            .map_err(|err| {
                ApplyError::InvalidTransaction(format!("Cannot build location list: {:?}", err))
            })?;

        let serialized = match location_list.into_bytes() {
            Ok(serialized) => serialized,
            Err(err) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Cannot serialize location list: {:?}",
                    err
                )));
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn remove_location(&self, location_id: &str) -> Result<(), ApplyError> {
        let address = compute_gs1_location_address(&location_id);
        let locations = match self.context.get_state_entry(&address)? {
            Some(packed) => match LocationList::from_bytes(packed.as_slice()) {
                Ok(location_list) => location_list.locations().to_vec(),
                Err(err) => {
                    return Err(ApplyError::InternalError(format!(
                        "Cannot deserialize location list: {:?}",
                        err
                    )));
                }
            },
            None => vec![],
        };

        let filtered_locations = locations
            .into_iter()
            .filter(|location| location.location_id() != location_id)
            .collect::<Vec<_>>();

        // If the only location at the address was the one we are removing, we can delete the entire state entry
        // Else, we can set the the filtered address list at the address
        if filtered_locations.is_empty() {
            self.context
                .delete_state_entries(&[address])
                .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        } else {
            let location_list = LocationListBuilder::new()
                .with_locations(filtered_locations)
                .build()
                .map_err(|err| {
                    ApplyError::InvalidTransaction(format!("Cannot build location list: {:?}", err))
                })?;

            let serialized = match location_list.into_bytes() {
                Ok(serialized) => serialized,
                Err(_) => {
                    return Err(ApplyError::InternalError(String::from(
                        "Cannot serialize location list",
                    )));
                }
            };
            self.context
                .set_state_entry(address, serialized)
                .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        }

        Ok(())
    }

    pub fn get_agent(&self, public_key: &str) -> Result<Option<Agent>, ApplyError> {
        let address = compute_agent_address(public_key);
        match self.context.get_state_entry(&address)? {
            Some(packed) => {
                let agents: AgentList = match AgentList::from_bytes(packed.as_slice()) {
                    Ok(agents) => agents,
                    Err(err) => {
                        return Err(ApplyError::InvalidTransaction(format!(
                            "Cannot deserialize agent list: {:?}",
                            err,
                        )));
                    }
                };

                for agent in agents.agents() {
                    if agent.public_key() == public_key {
                        return Ok(Some(agent.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn get_organization(&self, org_id: &str) -> Result<Option<Organization>, ApplyError> {
        let address = compute_org_address(org_id);
        match self.context.get_state_entry(&address)? {
            Some(packed) => {
                let orgs: OrganizationList = match OrganizationList::from_bytes(packed.as_slice()) {
                    Ok(orgs) => orgs,
                    Err(err) => {
                        return Err(ApplyError::InternalError(format!(
                            "Cannot deserialize organization list: {:?}",
                            err,
                        )))
                    }
                };

                for org in orgs.organizations() {
                    if org.org_id() == org_id {
                        return Ok(Some(org.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn get_schema(&self, name: &str) -> Result<Option<Schema>, ApplyError> {
        let address = compute_schema_address(name);
        match self.context.get_state_entry(&address)? {
            Some(packed) => {
                let schemas = match SchemaList::from_bytes(packed.as_slice()) {
                    Ok(schemas) => schemas,
                    Err(err) => {
                        return Err(ApplyError::InvalidTransaction(format!(
                            "Cannot deserialize schema list: {:?}",
                            err,
                        )));
                    }
                };

                for schema in schemas.schemas() {
                    if schema.name() == name {
                        return Ok(Some(schema.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }
}
