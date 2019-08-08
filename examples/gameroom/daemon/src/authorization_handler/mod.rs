/*
 * Copyright 2019 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

mod error;
pub use error::AppAuthHandlerError;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::SystemTime;

use awc::ws::{Codec, Frame};
use diesel::connection::Connection;
use futures::{future, stream::Stream};
use hyper::{
    header,
    rt::{self, Future},
    Body, Client, Request, StatusCode,
};
use tokio::codec::Decoder;
use uuid::Uuid;

use gameroom_database::{
    helpers,
    models::{CircuitProposal, NewCircuitMember, NewCircuitService},
    ConnectionPool,
};
use libsplinter::admin::messages::{
    AdminServiceEvent, CircuitProposal as MsgCircuitProposal, SplinterNode, SplinterService,
};

pub struct AppAuthHandlerShutdownHandle {
    do_shutdown: Box<dyn Fn() -> Result<(), AppAuthHandlerError> + Send>,
}

impl AppAuthHandlerShutdownHandle {
    pub fn shutdown(&self) -> Result<(), AppAuthHandlerError> {
        (*self.do_shutdown)()
    }
}

pub fn run(
    splinterd_url: &str,
    db_conn: ConnectionPool,
) -> Result<
    (
        AppAuthHandlerShutdownHandle,
        thread::JoinHandle<Result<(), AppAuthHandlerError>>,
    ),
    AppAuthHandlerError,
> {
    let splinterd_url = splinterd_url.to_owned();
    let client = Client::new();
    let shutdown_signaler = Arc::new(AtomicBool::new(true));
    let running = shutdown_signaler.clone();
    let join_handle = thread::Builder::new()
        .name("GameroomDAppAuthHandler".into())
        .spawn(move || {
            let req = Request::builder()
                .uri(format!("{}/ws/admin/register/gameroom", splinterd_url))
                .header(header::UPGRADE, "websocket")
                .header(header::CONNECTION, "Upgrade")
                .header(header::SEC_WEBSOCKET_VERSION, "13")
                .header(header::SEC_WEBSOCKET_KEY, "13")
                .body(Body::empty())
                .map_err(|err| AppAuthHandlerError::RequestError(format!("{}", err)))?;

            rt::run(
                client
                    .request(req)
                    .and_then(|res| {
                        if res.status() != StatusCode::SWITCHING_PROTOCOLS {
                            error!("The server didn't upgrade: {}", res.status());
                        }
                        res.into_body().on_upgrade()
                    })
                    .map_err(|e| error!("The client returned an error: {}", e))
                    .and_then(move |upgraded| {
                        let codec = Codec::new().client_mode();
                        let framed = codec.framed(upgraded);

                        // Read stream until shutdown signal is received
                        framed
                            .take_while(move |message| {
                                match message {
                                    Frame::Text(msg) => {
                                        if let Some(bytes) = msg {
                                            if let Err(err) =
                                                process_text_message(&bytes[..], &db_conn)
                                            {
                                                error!(
                                                    "An error occurred while processing a message:
                                                    {}",
                                                    err
                                                );
                                            }
                                        } else {
                                            error!("Received empty message");
                                        }
                                    }
                                    Frame::Ping(msg) => info!("Received Ping {}", msg),
                                    _ => error!("Received unknown message: {:?}", message),
                                };

                                future::ok(running.load(Ordering::SeqCst))
                            })
                            // Transform stream into a future
                            .for_each(|_| future::ok(()))
                            .map_err(|e| error!("The client returned an error: {}", e))
                    }),
            );

            Ok(())
        })?;

    let do_shutdown = Box::new(move || {
        debug!("Shutting down application authentication handler");
        shutdown_signaler.store(false, Ordering::SeqCst);
        Ok(())
    });

    Ok((AppAuthHandlerShutdownHandle { do_shutdown }, join_handle))
}

fn process_text_message(msg: &[u8], pool: &ConnectionPool) -> Result<(), AppAuthHandlerError> {
    let admin_event: AdminServiceEvent = serde_json::from_slice(msg)?;
    match admin_event {
        AdminServiceEvent::ProposalSubmitted(msg_proposal) => {
            let time = SystemTime::now();
            let proposal_id = Uuid::new_v4().to_string();
            let proposal = parse_proposal(&msg_proposal, &proposal_id, time);
            let services = parse_splinter_services(&proposal_id, &msg_proposal.circuit.roster);
            let nodes = parse_splinter_nodes(&proposal_id, &msg_proposal.circuit.members);
            let conn = &*pool.get()?;

            // insert proposal information in database tables in a single transaction
            conn.transaction::<_, _, _>(|| {
                helpers::insert_circuit_proposal(conn, proposal)?;
                helpers::insert_circuit_service(conn, &services)?;
                helpers::insert_circuit_member(conn, &nodes)?;

                debug!("Inserted new proposal into database");
                Ok(())
            })
        }
        _ => {
            error!("Unknown message type {:?}", admin_event);
            Ok(())
        }
    }
}

fn parse_proposal(
    proposal: &MsgCircuitProposal,
    id: &str,
    timestamp: SystemTime,
) -> CircuitProposal {
    CircuitProposal {
        id: id.to_string(),
        proposal_type: format!("{:?}", proposal.proposal_type),
        circuit_id: proposal.circuit_id.clone(),
        circuit_hash: proposal.circuit_hash.clone(),
        requester: proposal.requester.clone(),
        authorization_type: format!("{:?}", proposal.circuit.authorization_type),
        persistence: format!("{:?}", proposal.circuit.persistence),
        routes: format!("{:?}", proposal.circuit.routes),
        circuit_management_type: proposal.circuit.circuit_management_type.clone(),
        application_metadata: proposal.circuit.application_metadata.clone(),
        status: "Pending".to_string(),
        created_time: timestamp,
        updated_time: timestamp,
    }
}

fn parse_splinter_services(
    proposal_id: &str,
    splinter_services: &[SplinterService],
) -> Vec<NewCircuitService> {
    splinter_services
        .iter()
        .map(|service| NewCircuitService {
            proposal_id: proposal_id.to_string(),
            service_id: service.service_id.to_string(),
            service_type: service.service_type.to_string(),
            allowed_nodes: service.allowed_nodes.clone(),
        })
        .collect()
}

fn parse_splinter_nodes(
    proposal_id: &str,
    splinter_nodes: &[SplinterNode],
) -> Vec<NewCircuitMember> {
    splinter_nodes
        .iter()
        .map(|node| NewCircuitMember {
            proposal_id: proposal_id.to_string(),
            node_id: node.node_id.to_string(),
            endpoint: node.endpoint.to_string(),
        })
        .collect()
}

#[cfg(all(feature = "test-authorization-handler", test))]
mod test {
    use super::*;
    use diesel::{prelude::*, RunQueryDsl};
    use gameroom_database::models::{CircuitMember, CircuitService};

    use libsplinter::admin::messages::{
        AuthorizationType, CreateCircuit, PersistenceType, ProposalType, RouteType,
    };

    static DATABASE_URL: &str = "postgres://gameroom_test:gameroom_test@db-test:5432/gameroom_test";

    #[test]
    /// Tests if when receiving an admin message to CreateProposal the circuit_proposal
    /// table is updated as expected
    fn test_process_proposal_submitted_message_update_proposal_table() {
        let pool: ConnectionPool = gameroom_database::create_connection_pool(DATABASE_URL)
            .expect("Failed to get database connection pool");

        clear_circuit_proposals_table(&pool);
        let message = get_submit_proposal_msg("my_circuit");
        let serialized = serde_json::to_vec(&message).expect("Failed to serialize message");

        process_text_message(&serialized, &pool).expect("Error processing message");

        let proposals = query_proposals_table(&pool);

        assert_eq!(proposals.len(), 1);

        let proposal = &proposals[0];
        let expected_proposal = get_circuit_proposal("", "my_circuit", SystemTime::now());

        assert_eq!(proposal.proposal_type, expected_proposal.proposal_type);
        assert_eq!(proposal.circuit_id, expected_proposal.circuit_id);
        assert_eq!(proposal.circuit_hash, expected_proposal.circuit_hash);
        assert_eq!(proposal.requester, expected_proposal.requester);
        assert_eq!(
            proposal.authorization_type,
            expected_proposal.authorization_type
        );
        assert_eq!(proposal.persistence, expected_proposal.persistence);
        assert_eq!(proposal.routes, expected_proposal.routes);
        assert_eq!(
            proposal.circuit_management_type,
            expected_proposal.circuit_management_type
        );
        assert_eq!(
            proposal.application_metadata,
            expected_proposal.application_metadata
        );
        assert_eq!(proposal.status, expected_proposal.status);
    }

    #[test]
    /// Tests if when receiving an admin message to CreateProposal the proposal_circuit_member
    /// table is updated as expected
    fn test_process_proposal_submitted_message_update_member_table() {
        let pool: ConnectionPool = gameroom_database::create_connection_pool(DATABASE_URL)
            .expect("Failed to get database connection pool");

        clear_circuit_proposals_table(&pool);
        let message = get_submit_proposal_msg("my_circuit");
        let serialized = serde_json::to_vec(&message).expect("Failed to serialize message");

        process_text_message(&serialized, &pool).expect("Error processing message");

        let members = query_circuit_members_table(&pool);

        assert_eq!(members.len(), 1);

        let node = &members[0];
        let expected_node = get_new_circuit_member("");

        assert_eq!(node.node_id, expected_node.node_id);
        assert_eq!(node.endpoint, expected_node.endpoint);
    }

    #[test]
    /// Tests if when receiving an admin message to CreateProposal the proposal_circuit_service
    /// table is updated as expected
    fn test_process_proposal_submitted_message_update_service_table() {
        let pool: ConnectionPool = gameroom_database::create_connection_pool(DATABASE_URL)
            .expect("Failed to get database connection pool");

        clear_circuit_proposals_table(&pool);
        let message = get_submit_proposal_msg("my_circuit");
        let serialized = serde_json::to_vec(&message).expect("Failed to serialize message");

        process_text_message(&serialized, &pool).expect("Error processing message");

        let services = query_circuit_service_table(&pool);

        assert_eq!(services.len(), 1);

        let service = &services[0];
        let expected_service = get_new_circuit_service("");

        assert_eq!(service.service_id, expected_service.service_id);
        assert_eq!(service.service_type, expected_service.service_type);
        assert_eq!(service.allowed_nodes, expected_service.allowed_nodes);
    }

    #[test]
    /// Tests if the admin message CreateProposal to a database CircuitProposal is successful
    fn test_parse_proposal() {
        let time = SystemTime::now();
        let proposal = parse_proposal(&get_msg_proposal("my_circuit"), "my_proposal", time.clone());

        assert_eq!(
            proposal,
            get_circuit_proposal("my_proposal", "my_circuit", time.clone())
        )
    }

    #[test]
    /// Tests if the admin message SplinterService to a database NewCircuitService is successful
    fn test_parse_circuit_service() {
        let services = parse_splinter_services(
            "my_proposal",
            &get_msg_proposal("my_circuit").circuit.roster,
        );

        assert_eq!(services, vec![get_new_circuit_service("my_proposal")])
    }

    #[test]
    /// Tests if the admin message SplinterNode to a database NewCircuitMember is successful
    fn test_parse_circuit_member() {
        let members = parse_splinter_nodes(
            "my_proposal",
            &get_msg_proposal("my_circuit").circuit.members,
        );

        assert_eq!(members, vec![get_new_circuit_member("my_proposal")])
    }

    fn get_create_circuit_msg(circuit_id: &str) -> CreateCircuit {
        CreateCircuit {
            circuit_id: circuit_id.to_string(),
            roster: vec![SplinterService {
                service_id: "scabbard_123".to_string(),
                service_type: "scabbard".to_string(),
                allowed_nodes: vec!["acme_corp".to_string()],
            }],
            members: vec![SplinterNode {
                node_id: "Node-123".to_string(),
                endpoint: "127.0.0.1:8282".to_string(),
            }],
            authorization_type: AuthorizationType::Trust,
            persistence: PersistenceType::Any,
            routes: RouteType::Any,
            circuit_management_type: "gameroom".to_string(),
            application_metadata: vec![],
        }
    }

    fn get_msg_proposal(circuit_id: &str) -> MsgCircuitProposal {
        MsgCircuitProposal {
            proposal_type: ProposalType::Create,
            circuit_id: circuit_id.to_string(),
            circuit_hash: "8e066d41911817a42ab098eda35a2a2b11e93c753bc5ecc3ffb3e99ed99ada0d"
                .to_string(),
            circuit: get_create_circuit_msg(circuit_id),
            votes: vec![],
            requester: "acme_corp".to_string(),
        }
    }

    fn get_submit_proposal_msg(circuit_id: &str) -> AdminServiceEvent {
        AdminServiceEvent::ProposalSubmitted(get_msg_proposal(circuit_id))
    }

    fn get_circuit_proposal(
        proposal_id: &str,
        circuit_id: &str,
        timestamp: SystemTime,
    ) -> CircuitProposal {
        CircuitProposal {
            id: proposal_id.to_string(),
            proposal_type: "Create".to_string(),
            circuit_id: circuit_id.to_string(),
            circuit_hash: "8e066d41911817a42ab098eda35a2a2b11e93c753bc5ecc3ffb3e99ed99ada0d"
                .to_string(),
            requester: "acme_corp".to_string(),
            authorization_type: "Trust".to_string(),
            persistence: "Any".to_string(),
            routes: "Any".to_string(),
            circuit_management_type: "gameroom".to_string(),
            application_metadata: vec![],
            status: "Pending".to_string(),
            created_time: timestamp,
            updated_time: timestamp,
        }
    }

    fn get_new_circuit_service(proposal_id: &str) -> NewCircuitService {
        NewCircuitService {
            proposal_id: proposal_id.to_string(),
            service_id: "scabbard_123".to_string(),
            service_type: "scabbard".to_string(),
            allowed_nodes: vec!["acme_corp".to_string()],
        }
    }

    fn get_new_circuit_member(proposal_id: &str) -> NewCircuitMember {
        NewCircuitMember {
            proposal_id: proposal_id.to_string(),
            node_id: "Node-123".to_string(),
            endpoint: "127.0.0.1:8282".to_string(),
        }
    }

    fn query_circuit_members_table(pool: &ConnectionPool) -> Vec<CircuitMember> {
        use gameroom_database::schema::proposal_circuit_member;

        let conn = &*pool.get().expect("Error getting db connection");
        proposal_circuit_member::table
            .select(proposal_circuit_member::all_columns)
            .load::<CircuitMember>(conn)
            .expect("Error fetching circuit members")
    }

    fn query_circuit_service_table(pool: &ConnectionPool) -> Vec<CircuitService> {
        use gameroom_database::schema::proposal_circuit_service;

        let conn = &*pool.get().expect("Error getting db connection");
        proposal_circuit_service::table
            .select(proposal_circuit_service::all_columns)
            .load::<CircuitService>(conn)
            .expect("Error fetching circuit members")
    }

    fn query_proposals_table(pool: &ConnectionPool) -> Vec<CircuitProposal> {
        use gameroom_database::schema::circuit_proposal;

        let conn = &*pool.get().expect("Error getting db connection");
        circuit_proposal::table
            .select(circuit_proposal::all_columns)
            .load::<CircuitProposal>(conn)
            .expect("Error fetching proposals")
    }

    fn clear_circuit_proposals_table(pool: &ConnectionPool) {
        use gameroom_database::schema::circuit_proposal::dsl::*;

        let conn = &*pool.get().expect("Error getting db connection");
        diesel::delete(circuit_proposal)
            .execute(conn)
            .expect("Error cleaning proposals table");
    }

}
