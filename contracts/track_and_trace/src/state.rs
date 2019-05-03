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

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::ApplyError;
        use sabre_sdk::TransactionContext;
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
        use sawtooth_sdk::processor::handler::TransactionContext;
    }
}

use grid_sdk::protos::track_and_trace_agent::TrackAndTraceAgent as Agent;
use grid_sdk::protos::track_and_trace_agent::TrackAndTraceAgentContainer as AgentContainer;
use grid_sdk::protos::track_and_trace_property::{
    Property, PropertyContainer, PropertyPage, PropertyPageContainer,
};
use grid_sdk::protos::track_and_trace_proposal::ProposalContainer;
use grid_sdk::protos::track_and_trace_record::{
    Record, RecordContainer, RecordType, RecordTypeContainer,
};
use protobuf::Message;

use crate::addressing::*;

pub struct SupplyChainState<'a> {
    context: &'a mut TransactionContext,
}

impl<'a> SupplyChainState<'a> {
    pub fn new(context: &'a mut TransactionContext) -> SupplyChainState {
        SupplyChainState { context }
    }

    pub fn get_record(&mut self, record_id: &str) -> Result<Option<Record>, ApplyError> {
        let address = make_record_address(record_id);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let records: RecordContainer = match protobuf::parse_from_bytes(packed.as_slice()) {
                    Ok(records) => records,
                    Err(_) => {
                        return Err(ApplyError::InternalError(String::from(
                            "Cannot deserialize record container",
                        )));
                    }
                };

                for record in records.get_entries() {
                    if record.record_id == record_id {
                        return Ok(Some(record.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_record(&mut self, record_id: &str, record: Record) -> Result<(), ApplyError> {
        let address = make_record_address(record_id);
        let d = self.context.get_state_entry(&address)?;
        let mut record_container = match d {
            Some(packed) => match protobuf::parse_from_bytes(packed.as_slice()) {
                Ok(records) => records,
                Err(_) => {
                    return Err(ApplyError::InternalError(String::from(
                        "Cannot deserialize record container",
                    )));
                }
            },
            None => RecordContainer::new(),
        };
        // remove old record if it exists and sort the records by record id
        let records = record_container.get_entries().to_vec();
        let mut index = None;
        let mut count = 0;
        for record in records.clone() {
            if record.record_id == record_id {
                index = Some(count);
                break;
            }
            count = count + 1;
        }

        match index {
            Some(x) => {
                record_container.entries.remove(x);
            }
            None => (),
        };
        record_container.entries.push(record);
        record_container
            .entries
            .sort_by_key(|r| r.clone().record_id);
        let serialized = match record_container.write_to_bytes() {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize record container",
                )));
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn get_record_type(&mut self, type_name: &str) -> Result<Option<RecordType>, ApplyError> {
        let address = make_record_type_address(type_name);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let record_types: RecordTypeContainer =
                    match protobuf::parse_from_bytes(packed.as_slice()) {
                        Ok(record_types) => record_types,
                        Err(_) => {
                            return Err(ApplyError::InternalError(String::from(
                                "Cannot deserialize record type container",
                            )));
                        }
                    };

                for record_type in record_types.get_entries() {
                    if record_type.name == type_name {
                        return Ok(Some(record_type.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_record_type(
        &mut self,
        type_name: &str,
        record_type: RecordType,
    ) -> Result<(), ApplyError> {
        let address = make_record_type_address(type_name);
        let d = self.context.get_state_entry(&address)?;
        let mut record_types = match d {
            Some(packed) => match protobuf::parse_from_bytes(packed.as_slice()) {
                Ok(record_types) => record_types,
                Err(_) => {
                    return Err(ApplyError::InternalError(String::from(
                        "Cannot deserialize record container",
                    )));
                }
            },
            None => RecordTypeContainer::new(),
        };

        record_types.entries.push(record_type);
        record_types.entries.sort_by_key(|rt| rt.clone().name);
        let serialized = match record_types.write_to_bytes() {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize record type container",
                )));
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn get_agent(&mut self, agent_id: &str) -> Result<Option<Agent>, ApplyError> {
        let address = make_agent_address(agent_id);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let agents: AgentContainer = match protobuf::parse_from_bytes(packed.as_slice()) {
                    Ok(agents) => agents,
                    Err(_) => {
                        return Err(ApplyError::InternalError(String::from(
                            "Cannot deserialize agent container",
                        )));
                    }
                };

                for agent in agents.get_entries() {
                    if agent.public_key == agent_id {
                        return Ok(Some(agent.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_agent(&mut self, agent_id: &str, agent: Agent) -> Result<(), ApplyError> {
        let address = make_agent_address(agent_id);
        let d = self.context.get_state_entry(&address)?;
        let mut agents = match d {
            Some(packed) => match protobuf::parse_from_bytes(packed.as_slice()) {
                Ok(agents) => agents,
                Err(_) => {
                    return Err(ApplyError::InternalError(String::from(
                        "Cannot deserialize agent container",
                    )));
                }
            },
            None => AgentContainer::new(),
        };

        agents.entries.push(agent);
        agents.entries.sort_by_key(|a| a.clone().public_key);
        let serialized = match agents.write_to_bytes() {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize agent container",
                )));
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn get_property(
        &mut self,
        record_id: &str,
        property_name: &str,
    ) -> Result<Option<Property>, ApplyError> {
        let address = make_property_address(record_id, property_name, 0);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let properties: PropertyContainer =
                    match protobuf::parse_from_bytes(packed.as_slice()) {
                        Ok(properties) => properties,
                        Err(_) => {
                            return Err(ApplyError::InternalError(String::from(
                                "Cannot deserialize property container",
                            )));
                        }
                    };

                for property in properties.get_entries() {
                    if property.name == property_name {
                        return Ok(Some(property.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_property(
        &mut self,
        record_id: &str,
        property_name: &str,
        property: Property,
    ) -> Result<(), ApplyError> {
        let address = make_property_address(record_id, property_name, 0);
        let d = self.context.get_state_entry(&address)?;
        let mut property_container = match d {
            Some(packed) => match protobuf::parse_from_bytes(packed.as_slice()) {
                Ok(properties) => properties,
                Err(_) => {
                    return Err(ApplyError::InternalError(String::from(
                        "Cannot deserialize property container",
                    )));
                }
            },
            None => PropertyContainer::new(),
        };
        // remove old property if it exists and sort the properties by name
        let properties = property_container.get_entries().to_vec();
        let mut index = None;
        let mut count = 0;
        for prop in properties.clone() {
            if prop.name == property_name {
                index = Some(count);
                break;
            }
            count = count + 1;
        }

        match index {
            Some(x) => {
                property_container.entries.remove(x);
            }
            None => (),
        };
        property_container.entries.push(property);
        property_container.entries.sort_by_key(|p| p.clone().name);
        let serialized = match property_container.write_to_bytes() {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize property container",
                )));
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn get_property_page(
        &mut self,
        record_id: &str,
        property_name: &str,
        page: u32,
    ) -> Result<Option<PropertyPage>, ApplyError> {
        let address = make_property_address(record_id, property_name, page);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let property_pages: PropertyPageContainer =
                    match protobuf::parse_from_bytes(packed.as_slice()) {
                        Ok(property_pages) => property_pages,
                        Err(_) => {
                            return Err(ApplyError::InternalError(String::from(
                                "Cannot deserialize property page container",
                            )));
                        }
                    };

                for property_page in property_pages.get_entries() {
                    if property_page.name == property_name {
                        return Ok(Some(property_page.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn set_property_page(
        &mut self,
        record_id: &str,
        property_name: &str,
        page_num: u32,
        property_page: PropertyPage,
    ) -> Result<(), ApplyError> {
        let address = make_property_address(record_id, property_name, page_num);
        let d = self.context.get_state_entry(&address)?;
        let mut property_pages = match d {
            Some(packed) => match protobuf::parse_from_bytes(packed.as_slice()) {
                Ok(property_pages) => property_pages,
                Err(_) => {
                    return Err(ApplyError::InternalError(String::from(
                        "Cannot deserialize property page container",
                    )));
                }
            },
            None => PropertyPageContainer::new(),
        };
        // remove old property page if it exists and sort the property pages by name
        let pages = property_pages.get_entries().to_vec();
        let mut index = None;
        let mut count = 0;
        for page in pages.clone() {
            if page.name == property_name {
                index = Some(count);
                break;
            }
            count = count + 1;
        }

        match index {
            Some(x) => {
                property_pages.entries.remove(x);
            }
            None => (),
        };
        property_pages.entries.push(property_page);
        property_pages.entries.sort_by_key(|pp| pp.clone().name);
        let serialized = match property_pages.write_to_bytes() {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize property page container",
                )));
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn get_proposal_container(
        &mut self,
        record_id: &str,
        agent_id: &str,
    ) -> Result<Option<ProposalContainer>, ApplyError> {
        let address = make_proposal_address(record_id, agent_id);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let proposals: ProposalContainer =
                    match protobuf::parse_from_bytes(packed.as_slice()) {
                        Ok(property_pages) => property_pages,
                        Err(_) => {
                            return Err(ApplyError::InternalError(String::from(
                                "Cannot deserialize proposal container",
                            )));
                        }
                    };

                Ok(Some(proposals))
            }
            None => Ok(None),
        }
    }

    pub fn set_proposal_container(
        &mut self,
        record_id: &str,
        agent_id: &str,
        proposals: ProposalContainer,
    ) -> Result<(), ApplyError> {
        let address = make_proposal_address(record_id, agent_id);
        let serialized = match proposals.write_to_bytes() {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize proposal container",
                )));
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }
}
