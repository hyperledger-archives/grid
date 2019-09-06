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

use actix_web::{error, web, Error, HttpResponse};
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

use crate::application_metadata::ApplicationMetadata;
use crate::rest_api::RestApiResponseError;

use super::{
    get_response_paging_info, ErrorResponse, SuccessResponse, DEFAULT_LIMIT, DEFAULT_OFFSET,
};

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

#[derive(Debug, Serialize)]
struct ApiGameroom {
    circuit_id: String,
    authorization_type: String,
    persistence: String,
    routes: String,
    circuit_management_type: String,
    alias: String,
    status: String,
}

impl ApiGameroom {
    fn from(db_gameroom: Gameroom) -> Self {
        Self {
            circuit_id: db_gameroom.circuit_id.to_string(),
            authorization_type: db_gameroom.authorization_type.to_string(),
            persistence: db_gameroom.persistence.to_string(),
            routes: db_gameroom.routes.to_string(),
            circuit_management_type: db_gameroom.circuit_management_type.to_string(),
            alias: db_gameroom.alias.to_string(),
            status: db_gameroom.status.to_string(),
        }
    }
}

pub fn propose_gameroom(
    pool: web::Data<ConnectionPool>,
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

    let scabbard_admin_keys = match serde_json::to_string(
        &create_gameroom
            .member
            .iter()
            .map(|member| member.metadata.public_key.clone())
            .collect::<Vec<_>>(),
    ) {
        Ok(s) => s,
        Err(err) => {
            debug!("Failed to serialize member public keys: {}", err);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::internal_error())
                .into_future();
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
                debug!("Failed to serialize peer services: {}", err);
                return HttpResponse::InternalServerError()
                    .json(ErrorResponse::internal_error())
                    .into_future();
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

    let application_metadata = match check_alias_uniqueness(pool, &create_gameroom.alias) {
        Ok(()) => match ApplicationMetadata::new(&create_gameroom.alias).to_bytes() {
            Ok(bytes) => bytes,
            Err(err) => {
                debug!("Failed to serialize application metadata: {}", err);
                return HttpResponse::InternalServerError()
                    .json(ErrorResponse::internal_error())
                    .into_future();
            }
        },
        Err(err) => {
            return HttpResponse::BadRequest()
                .json(ErrorResponse::bad_request(&err.to_string()))
                .into_future();
        }
    };

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

    let payload_bytes = match make_payload(create_request, node_info.identity.to_string()) {
        Ok(bytes) => bytes,
        Err(err) => {
            debug!("Failed to make circuit management payload: {}", err);
            return HttpResponse::InternalServerError()
                .json(ErrorResponse::internal_error())
                .into_future();
        }
    };

    HttpResponse::Ok()
        .json(SuccessResponse::new(json!({
            "payload_bytes": payload_bytes
        })))
        .into_future()
}

fn check_alias_uniqueness(
    pool: web::Data<ConnectionPool>,
    alias: &str,
) -> Result<(), RestApiResponseError> {
    if let Some(gameroom) = helpers::fetch_gameroom_by_alias(&*pool.get()?, alias)? {
        return Err(RestApiResponseError::BadRequest(format!(
            "Gameroom with alias {} already exists",
            gameroom.alias
        )));
    }
    Ok(())
}

fn make_payload(
    create_request: CreateCircuit,
    local_node: String,
) -> Result<Vec<u8>, RestApiResponseError> {
    let circuit_proto = create_request.into_proto()?;
    let circuit_bytes = circuit_proto.write_to_bytes()?;
    let hashed_bytes = hash(MessageDigest::sha512(), &circuit_bytes)?;

    let mut header = Header::new();
    header.set_action(Action::CIRCUIT_CREATE_REQUEST);
    header.set_payload_sha512(hashed_bytes.to_vec());
    header.set_requester_node_id(local_node);
    let header_bytes = header.write_to_bytes()?;

    let mut circuit_management_payload = CircuitManagementPayload::new();
    circuit_management_payload.set_header(header_bytes);
    circuit_management_payload.set_circuit_create_request(circuit_proto);
    let payload_bytes = circuit_management_payload.write_to_bytes()?;
    Ok(payload_bytes)
}

pub fn list_gamerooms(
    pool: web::Data<ConnectionPool>,
    query: web::Query<HashMap<String, String>>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let mut base_link = "api/gamerooms?".to_string();
    let offset: usize = query
        .get("offset")
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| DEFAULT_OFFSET.to_string())
        .parse()
        .unwrap_or_else(|_| DEFAULT_OFFSET);

    let limit: usize = query
        .get("limit")
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| DEFAULT_LIMIT.to_string())
        .parse()
        .unwrap_or_else(|_| DEFAULT_LIMIT);

    let status_optional = query.get("status").map(ToOwned::to_owned);

    if let Some(status) = status_optional.clone() {
        base_link.push_str(format!("status={}?", status).as_str());
    }

    Box::new(
        web::block(move || list_gamerooms_from_db(pool, status_optional, limit, offset)).then(
            move |res| match res {
                Ok((gamerooms, query_count)) => {
                    let paging_info = get_response_paging_info(
                        limit,
                        offset,
                        "api/gamerooms?",
                        query_count as usize,
                    );
                    Ok(HttpResponse::Ok().json(SuccessResponse::list(gamerooms, paging_info)))
                }
                Err(err) => {
                    debug!("Internal Server Error: {}", err);
                    Ok(HttpResponse::InternalServerError().json(ErrorResponse::internal_error()))
                }
            },
        ),
    )
}

fn list_gamerooms_from_db(
    pool: web::Data<ConnectionPool>,
    status_optional: Option<String>,
    limit: usize,
    offset: usize,
) -> Result<(Vec<ApiGameroom>, i64), RestApiResponseError> {
    let db_limit = limit as i64;
    let db_offset = offset as i64;
    if let Some(status) = status_optional {
        let gamerooms = helpers::list_gamerooms_with_paging_and_status(
            &*pool.get()?,
            &status,
            db_limit,
            db_offset,
        )?
        .into_iter()
        .map(ApiGameroom::from)
        .collect();
        Ok((gamerooms, helpers::get_gameroom_count(&*pool.get()?)?))
    } else {
        let gamerooms = helpers::list_gamerooms_with_paging(&*pool.get()?, db_limit, db_offset)?
            .into_iter()
            .map(ApiGameroom::from)
            .collect();
        Ok((gamerooms, helpers::get_gameroom_count(&*pool.get()?)?))
    }
}

pub fn fetch_gameroom(
    pool: web::Data<ConnectionPool>,
    circuit_id: web::Path<String>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(
        web::block(move || fetch_gameroom_from_db(pool, &circuit_id)).then(|res| match res {
            Ok(gameroom) => Ok(HttpResponse::Ok().json(gameroom)),
            Err(err) => match err {
                error::BlockingError::Error(err) => {
                    match err {
                        RestApiResponseError::NotFound(err) => Ok(HttpResponse::NotFound()
                            .json(ErrorResponse::not_found(&err.to_string()))),
                        _ => Ok(HttpResponse::BadRequest()
                            .json(ErrorResponse::bad_request(&err.to_string()))),
                    }
                }
                error::BlockingError::Canceled => {
                    debug!("Internal Server Error: {}", err);
                    Ok(HttpResponse::InternalServerError().json(ErrorResponse::internal_error()))
                }
            },
        }),
    )
}

fn fetch_gameroom_from_db(
    pool: web::Data<ConnectionPool>,
    circuit_id: &str,
) -> Result<ApiGameroom, RestApiResponseError> {
    if let Some(gameroom) = helpers::fetch_gameroom(&*pool.get()?, circuit_id)? {
        return Ok(ApiGameroom::from(gameroom));
    }
    Err(RestApiResponseError::NotFound(format!(
        "Gameroom with id {} not found",
        circuit_id
    )))
}
