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

use actix_web::{client::Client, http::StatusCode, web, Error, HttpResponse};
use futures::Future;
use libsplinter::admin::messages::{
    AuthorizationType, CreateCircuit, PersistenceType, RouteType, SplinterNode, SplinterService,
};

use libsplinter::node_registry::Node;

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
}

pub fn propose_gameroom(
    client: web::Data<Client>,
    splinterd_url: web::Data<String>,
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

    let create_request = CreateCircuit {
        circuit_id: create_gameroom.alias.to_string(),
        roster: members
            .iter()
            .map(|node| SplinterService {
                service_id: format!("gameroom_{}", node.node_id),
                service_type: "scabbard".to_string(),
                allowed_nodes: vec![node.node_id.to_string()],
            })
            .collect::<Vec<SplinterService>>(),
        members,
        authorization_type: AuthorizationType::Trust,
        persistence: PersistenceType::Any,
        routes: RouteType::Any,
        circuit_management_type: "Gameroom".to_string(),
        application_metadata: vec![],
    };

    debug!("Create circuit message {:?}", create_request);

    client
        .post(format!("{}/admin/circuit", splinterd_url.get_ref()))
        .send_json(&create_request)
        .map_err(Error::from)
        .and_then(|resp| match resp.status() {
            StatusCode::ACCEPTED => Ok(HttpResponse::Accepted().finish()),
            _ => Ok(HttpResponse::InternalServerError().finish()),
        })
}
