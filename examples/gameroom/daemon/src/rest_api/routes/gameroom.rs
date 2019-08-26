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
use std::collections::HashMap;

use actix_web::{web, Error, HttpResponse};
use futures::{Future, IntoFuture};
use gameroom_database::{helpers, models::Gameroom, ConnectionPool};
use libsplinter::admin::messages::{
    AuthorizationType, CreateCircuit, DurabilityType, PersistenceType, RouteType, SplinterNode,
    SplinterService,
};
use libsplinter::node_registry::Node;
use libsplinter::protos::admin::{
    CircuitManagementPayload, CircuitManagementPayload_Action as Action,
    CircuitManagementPayload_Header as Header,
};
use openssl::hash::{hash, MessageDigest};
use protobuf::Message;
use uuid::Uuid;

use crate::rest_api::RestApiResponseError;

use super::{get_response_paging_info, Paging, DEFAULT_LIMIT, DEFAULT_OFFSET};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGameroomForm {
    alias: String,
    member: Vec<GameroomMember>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameroomMember {
    identity: String,
    metadata: MemberMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemberMetadata {
    organization: String,
    endpoint: String,
    public_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationMetadata {
    alias: String,
}

#[derive(Debug, Serialize)]
struct GameroomListResponse {
    data: Vec<ApiGameroom>,
    paging: Paging,
}

#[derive(Debug, Serialize)]
struct ApiGameroom {
    circuit_id: String,
    authorization_type: String,
    persistence: String,
    routes: String,
    circuit_management_type: String,
    application_metadata: Vec<u8>,
}

impl ApiGameroom {
    fn from(db_gameroom: Gameroom) -> Self {
        Self {
            circuit_id: db_gameroom.circuit_id.to_string(),
            authorization_type: db_gameroom.authorization_type.to_string(),
            persistence: db_gameroom.persistence.to_string(),
            routes: db_gameroom.routes.to_string(),
            circuit_management_type: db_gameroom.circuit_management_type.to_string(),
            application_metadata: db_gameroom.application_metadata.to_vec(),
        }
    }
}

pub fn propose_gameroom(
    create_gameroom: web::Json<CreateGameroomForm>,
    node_info: web::Data<Node>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let mut members = create_gameroom
        .member
        .iter()
        .map(|node| SplinterNode {
            node_id: node.identity.to_string(),
            endpoint: node.metadata.endpoint.to_string(),
        })
        .collect::<Vec<SplinterNode>>();

    members.push(SplinterNode {
        node_id: node_info.identity.to_string(),
        endpoint: node_info
            .metadata
            .get("endpoint")
            .unwrap_or(&"".to_string())
            .to_string(),
    });

    let partial_circuit_id = members.iter().fold(String::new(), |mut acc, member| {
        acc.push_str(&format!("::{}", member.node_id));
        acc
    });

    let application_metadata = match make_application_metadata(&create_gameroom.alias) {
        Ok(bytes) => bytes,
        Err(err) => {
            debug!("Failed to serialize application metadata: {}", err);
            return HttpResponse::InternalServerError().finish().into_future();
        }
    };

    let scabbard_admin_keys = match serde_json::to_string(
        &create_gameroom
            .member
            .iter()
            .map(|member| member.metadata.public_key.clone())
            .collect::<Vec<_>>(),
    ) {
        Ok(s) => s,
        Err(err) => {
            return HttpResponse::InternalServerError()
                .json(format!("failed to serialize member public keys: {}", err))
                .into_future()
        }
    };
    let mut scabbard_args = HashMap::new();
    scabbard_args.insert("admin_keys".into(), scabbard_admin_keys);

    let mut roster = vec![];
    for node in members.iter() {
        let peer_services = match serde_json::to_string(
            &members
                .iter()
                .filter_map(|other_node| {
                    if other_node.node_id != node.node_id {
                        Some(format!("gameroom_{}", other_node.node_id))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
        ) {
            Ok(s) => s,
            Err(err) => {
                return HttpResponse::InternalServerError()
                    .json(format!("failed to serialize peer services: {}", err))
                    .into_future()
            }
        };

        let mut service_args = scabbard_args.clone();
        service_args.insert("peer_services".into(), peer_services);

        roster.push(SplinterService {
            service_id: format!("gameroom_{}", node.node_id),
            service_type: "scabbard".to_string(),
            allowed_nodes: vec![node.node_id.to_string()],
            arguments: service_args,
        });
    }

    let create_request = CreateCircuit {
        circuit_id: format!(
            "gameroom{}::{}",
            partial_circuit_id,
            Uuid::new_v4().to_string()
        ),
        roster,
        members,
        authorization_type: AuthorizationType::Trust,
        persistence: PersistenceType::Any,
        durability: DurabilityType::NoDurabilty,
        routes: RouteType::Any,
        circuit_management_type: "gameroom".to_string(),
        application_metadata,
    };

    let payload_bytes = match make_payload(create_request) {
        Ok(bytes) => bytes,
        Err(err) => {
            debug!("Failed to make circuit management payload: {}", err);
            return HttpResponse::InternalServerError().finish().into_future();
        }
    };

    HttpResponse::Ok()
        .json(json!({ "data": { "payload_bytes": payload_bytes } }))
        .into_future()
}

fn make_application_metadata(alias: &str) -> Result<Vec<u8>, RestApiResponseError> {
    serde_json::to_vec(&ApplicationMetadata {
        alias: alias.to_string(),
    })
    .map_err(|err| RestApiResponseError::InternalError(err.to_string()))
}

fn make_payload(create_request: CreateCircuit) -> Result<Vec<u8>, RestApiResponseError> {
    let circuit_proto = create_request.into_proto()?;
    let circuit_bytes = circuit_proto.write_to_bytes()?;
    let hashed_bytes = hash(MessageDigest::sha512(), &circuit_bytes)?;

    let mut header = Header::new();
    header.set_action(Action::CIRCUIT_CREATE_REQUEST);
    header.set_payload_sha512(hashed_bytes.to_vec());
    let header_bytes = header.write_to_bytes()?;

    let mut circuit_management_payload = CircuitManagementPayload::new();
    circuit_management_payload.set_header(header_bytes);
    circuit_management_payload.set_circuit_create_request(circuit_proto);
    let payload_bytes = circuit_management_payload.write_to_bytes()?;
    Ok(payload_bytes)
}

pub fn list_gamerooms(
    pool: web::Data<ConnectionPool>,
    query: web::Query<HashMap<String, usize>>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let offset: usize = query
        .get("offset")
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| DEFAULT_OFFSET);

    let limit: usize = query
        .get("limit")
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| DEFAULT_LIMIT);

    Box::new(
        web::block(move || list_gamerooms_from_db(pool, limit, offset)).then(
            move |res| match res {
                Ok((gamerooms, query_count)) => {
                    let paging_info = get_response_paging_info(
                        limit,
                        offset,
                        "api/gamerooms?",
                        query_count as usize,
                    );
                    Ok(HttpResponse::Ok().json(GameroomListResponse {
                        data: gamerooms,
                        paging: paging_info,
                    }))
                }
                Err(err) => Ok(HttpResponse::InternalServerError().json(json!({
                    "message": format!("Internal Server error: {}", err.to_string())
                }))),
            },
        ),
    )
}

fn list_gamerooms_from_db(
    pool: web::Data<ConnectionPool>,
    limit: usize,
    offset: usize,
) -> Result<(Vec<ApiGameroom>, i64), RestApiResponseError> {
    let db_limit = limit as i64;
    let db_offset = offset as i64;

    let gamerooms =
        helpers::list_gamerooms_with_paging(&*pool.get()?, "ACCECPTED", db_limit, db_offset)?
            .into_iter()
            .map(ApiGameroom::from)
            .collect();
    let gameroom_count = helpers::get_gameroom_count(&*pool.get()?, "ACCEPTED")?;

    Ok((gamerooms, gameroom_count))
}

fn fetch_gameroom_from_db(
    pool: web::Data<ConnectionPool>,
    circuit_id: &str,
) -> Result<ApiGameroom, RestApiResponseError> {
    if let Some(gameroom) =
        helpers::fetch_gameroom_with_status(&*pool.get()?, "ACCECPTED", circuit_id)?
    {
        return Ok(ApiGameroom::from(gameroom));
    }
    Err(RestApiResponseError::NotFound(format!(
        "Gameroom {}",
        circuit_id
    )))
}
