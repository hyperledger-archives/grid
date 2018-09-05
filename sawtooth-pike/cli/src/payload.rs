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

//! Functions to assist with Pike payload creation

use protobuf;
use protos::state::KeyValueEntry;
use protos::payload::{
    CreateAgentAction,
    CreateOrganizationAction,
    UpdateAgentAction,
    UpdateOrganizationAction
};
use protos::payload::PikePayload;
use protos::payload::PikePayload_Action;

/// Creates a payload with a create agent action within
///
/// # Arguments
///
/// * `org_id` - The id for the organization that the agent belongs to.
/// * `name` - The agent's name
/// * `public_key` - The agent's public key
/// * `roles` - A list of the agents roles
pub fn create_agent_payload(
    org_id: &str,
    public_key: &str,
    roles: Vec<String>,
    metadata: Vec<KeyValueEntry>,
) -> PikePayload {
    let mut create_agent = CreateAgentAction::new();
    create_agent.set_org_id(String::from(org_id));
    create_agent.set_public_key(String::from(public_key));
    create_agent.set_roles(protobuf::RepeatedField::from_vec(roles));
    create_agent.set_metadata(protobuf::RepeatedField::from_vec(metadata));

    let mut payload = PikePayload::new();
    payload.action = PikePayload_Action::CREATE_AGENT;
    payload.set_create_agent(create_agent);
    
    payload
}

/// Creates a payload with an update agent action within
///
/// # Arguments
///
/// * `org_id` - The id for the organization that the agent belongs to.
/// * `name` - The agent's name
/// * `public_key` - The agent's public key
/// * `roles` - A list of the agents roles
pub fn update_agent_payload(
    org_id: &str,
    public_key: &str,
    roles: Vec<String>,
    metadata: Vec<KeyValueEntry>,
) -> PikePayload {
    let mut update_agent = UpdateAgentAction::new();
    update_agent.set_org_id(String::from(org_id));
    update_agent.set_public_key(String::from(public_key));
    update_agent.set_roles(protobuf::RepeatedField::from_vec(roles));
    update_agent.set_metadata(protobuf::RepeatedField::from_vec(metadata));

    let mut payload = PikePayload::new();
    payload.action = PikePayload_Action::UPDATE_AGENT;
    payload.set_update_agent(update_agent);

    payload
}

/// Creates a payload with a create organization action within
///
/// # Arguments
///
/// * `id` - Unique ID for organization
/// * `name` - The organization's name
/// * `address` - The physical address of the organization
pub fn create_org_payload(id: &str, name: &str, address: Option<&str>) -> PikePayload {
    let mut create_org = CreateOrganizationAction::new();
    create_org.set_id(String::from(id));
    create_org.set_name(String::from(name));

    if let Some(addr) = address {
        create_org.set_address(String::from(addr));
    }

    let mut payload = PikePayload::new();
    payload.action = PikePayload_Action::CREATE_ORGANIZATION;
    payload.set_create_organization(create_org);

    payload
}

/// Creates a payload with an update organization action within
///
/// # Arguments
///
/// * `id` - Unique ID for organization
/// * `name` - The organization's name
/// * `address` - The physical address of the organization
pub fn update_org_payload(id: &str, name: &str, address: Option<&str>) -> PikePayload {
    let mut update_org = UpdateOrganizationAction::new();
    update_org.set_id(String::from(id));
    update_org.set_name(String::from(name));

    if let Some(addr) = address {
        update_org.set_address(String::from(addr));
    }

    let mut payload = PikePayload::new();
    payload.action = PikePayload_Action::UPDATE_ORGANIZATION;
    payload.set_update_organization(update_org);

    payload
}
