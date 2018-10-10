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

use sawtooth_sdk::messages::transaction_receipt::StateChange;
use sawtooth_sdk::messages::transaction_receipt::StateChange_Type;

use pike_db as db;
use pike_db::{PgConnection, NotFound, QueryError};
use pike_db::models::{NewAgent, NewOrganization, NewSmartPermission};

use addresser::{Resource, ResourceError, byte_to_resource};

use protos::state::{
    Agent,
    Organization,
    SmartPermission,
    AgentList,
    OrganizationList,
    SmartPermissionList,
};

pub fn apply_state_change(conn: &PgConnection, state_change: &StateChange) -> Result<(), StateChangeError> {
    match state_change.field_type{
        StateChange_Type::SET => set(conn, &state_change.address, &state_change.value),
        StateChange_Type::DELETE => delete(conn,  &state_change.address),
        _ => Err(
            StateChangeError::UnsupportedTypeError(
                format!("unsuppoted type {:?}", state_change.field_type)))
    }
}

fn set(conn: &PgConnection, address: &str, value: &[u8]) -> Result<(), StateChangeError> {
    let resource_byte = &address[6..8];

    let results: Vec<StateChangeError> = match byte_to_resource(resource_byte)? {
        Resource::AGENT => protobuf::parse_from_bytes::<AgentList>(value)?
            .get_agents()
            .into_iter()
            .filter_map(|agent| set_agent(conn, agent).err())
            .collect(),
        Resource::ORG => protobuf::parse_from_bytes::<OrganizationList>(value)?
            .get_organizations()
            .into_iter()
            .filter_map(|org| set_org(conn, org).err())
            .collect(),
        Resource::SPF => protobuf::parse_from_bytes::<SmartPermissionList>(value)?
            .get_smart_permissions()
            .into_iter()
            .filter_map(|spf| set_spf(conn, spf, address).err())
            .collect()
    };

    if results.is_empty() {
        Ok(())
    } else {
        Err(StateChangeError::SetErrors(results))
    }
}

fn delete(conn: &PgConnection, address: &str) -> Result<(), StateChangeError> {
    let resource_byte = &address[6..8];

    match byte_to_resource(resource_byte)? {
        Resource::SPF => delete_spf(conn, address),
        _ => {
            return Err(
                StateChangeError::UnsupportedResourceError(
                    "Resource does not support DELETE".into()));
        }
    }
}

fn set_agent(conn: &PgConnection, agent: &Agent) -> Result<(), StateChangeError> {
    let metadata = agent
        .metadata
        .iter()
        .map(|x| json!({
            "key": x.get_key(),
            "value": x.get_value()
        }))
    .collect();
    let new_agent = NewAgent {
        org_id: &agent.org_id,
        public_key: &agent.public_key,
        active: agent.active,
        roles: agent.roles.to_vec(),
        metadata,
    };
    match db::get_agent(conn, &new_agent.public_key) {
        Ok(_) => db::update_agent(conn, &new_agent.public_key, new_agent)
            .and_then(|_| Ok(()))
            .map_err(StateChangeError::from),
        Err(NotFound) => db::create_agent(conn, new_agent)
            .and_then(|_| Ok(()))
            .map_err(StateChangeError::from),
        Err(e) => Err(StateChangeError::from(e))
    }
}

fn set_org(conn: &PgConnection, org: &Organization) -> Result<(), StateChangeError> {
    let new_org = NewOrganization {
        id: &org.org_id,
        name: &org.name,
        address: &org.address
    };

    match db::get_org(conn, &new_org.id) {
        Ok(_) => db::update_organization(conn, &new_org.id, new_org)
            .and_then(|_| Ok(()))
            .map_err(StateChangeError::from),
        Err(NotFound) => db::create_organization(conn, new_org)
            .and_then(|_| Ok(()))
            .map_err(StateChangeError::from),
        Err(e) => Err(StateChangeError::from(e))
    }
}

fn set_spf(conn: &PgConnection, spf: &SmartPermission, address: &str) -> Result<(), StateChangeError> {
    let new_spf = NewSmartPermission {
        org_id: &spf.org_id,
        name: &spf.name,
        address: address
    };

    db::create_smart_permission(conn, new_spf)
        .and_then(|_| Ok(()))
        .map_err(StateChangeError::from)
}

fn delete_spf(conn: &PgConnection, address: &str) -> Result<(), StateChangeError> {
    db::delete_smart_permission(conn, address)
        .and_then(|_| Ok(()))
        .map_err(StateChangeError::from)
}

#[derive(Debug)]
pub enum StateChangeError {
    UnsupportedTypeError(String),
    ResourceError(ResourceError),
    UnsupportedResourceError(String),
    SqlQueryError(QueryError),
    ParseError(protobuf::ProtobufError),
    SetErrors(Vec<StateChangeError>)
}

impl From<protobuf::ProtobufError> for StateChangeError {
    fn from(e: protobuf::ProtobufError) -> Self {
        StateChangeError::ParseError(e)
    }
}

impl From<QueryError> for StateChangeError {
    fn from(e: QueryError) -> Self {
        StateChangeError::SqlQueryError(e)
    }
}


impl From<ResourceError> for StateChangeError {
    fn from(e: ResourceError) -> Self {
        StateChangeError::ResourceError(e)
    }
}
