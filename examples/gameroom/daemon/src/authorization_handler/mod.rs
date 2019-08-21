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
    mpsc::{self, Receiver, Sender, TryRecvError},
    Arc,
};
use std::thread;
use std::time::{Duration, SystemTime};

use awc::ws::{CloseCode, CloseReason, Codec, Frame, Message};
use diesel::connection::Connection;
use futures::{
    future::{self, Either, FutureResult},
    sink::Sink,
    stream::{SplitSink, Stream},
    Future,
};
use hyper::upgrade::Upgraded;
use hyper::{header, Body, Client, Request, StatusCode};
use tokio::{
    codec::{Decoder, Framed},
    runtime::Runtime,
};

use gameroom_database::{
    helpers,
    models::{
        Gameroom, GameroomProposal, NewGameroomMember, NewGameroomProposal, NewGameroomService,
        NewProposalVoteRecord,
    },
    ConnectionPool,
};
use libsplinter::admin::messages::{
    AdminServiceEvent, CircuitProposal, CircuitProposalVote, CreateCircuit, SplinterNode,
    SplinterService,
};

// number of consecutive invalid messages the client will accept before trying to reconnect
static INVALID_MESSAGE_THRESHOLD: u32 = 10;

// wait time in seconds before the client attempts to reconnect
static RECONNECT_WAIT_TIME: u64 = 10;

pub struct AppAuthHandlerShutdownHandle {
    do_shutdown: Box<dyn Fn() -> Result<(), AppAuthHandlerError> + Send>,
}

impl AppAuthHandlerShutdownHandle {
    pub fn shutdown(&self) -> Result<(), AppAuthHandlerError> {
        (*self.do_shutdown)()
    }
}

pub struct ThreadJoinHandle(Vec<thread::JoinHandle<Result<(), AppAuthHandlerError>>>);

impl ThreadJoinHandle {
    pub fn join(self) {
        self.0.into_iter().for_each(|join_handle| {
            let _ = join_handle.join();
        });
    }
}

pub fn run(
    splinterd_url: &str,
    db_conn: ConnectionPool,
) -> Result<(AppAuthHandlerShutdownHandle, ThreadJoinHandle), AppAuthHandlerError> {
    let url = splinterd_url.to_string();
    let shutdown_signaler = Arc::new(AtomicBool::new(true));

    // channel to send request future to client thread
    let (tx_future, rx_future) = mpsc::channel();

    //  channel to send sink to connection manager thread
    let (tx_closing, rx_closing) = mpsc::channel();

    //  channel to send closing message to connection manager thread
    let (tx_msg_closing, rx_msg_closing) = mpsc::channel::<Message>();

    // Flag to signal the thread managing the websocket connection that it should attempt to
    // reconnect once the connection is dropped.
    let reconnect = Arc::new(AtomicBool::new(false));

    let running = shutdown_signaler.clone();

    // Thread that will receive request futures and execute them
    let join_handle_client = thread::Builder::new()
        .name("GameroomdAppAuthHandlerClient".into())
        .spawn(move || {
            let result = loop {
                let request_future = match try_recv(&rx_future, running.clone()) {
                    Ok(future) => {
                        match future {
                            Some(future) => future,
                            None => break Ok(()), // no request future to receive
                        }
                    }
                    Err(err) => break Err(err),
                };

                let mut runtime = match Runtime::new() {
                    Ok(rt) => rt,
                    Err(err) => break Err(err.into()),
                };
                if let Err(err) = runtime.block_on(request_future) {
                    break Err(err);
                };
                if !running.load(Ordering::SeqCst) {
                    debug!("Exiting request loop");
                    break Ok(());
                }
            };

            // if loop exits with an error, signal that AppAuthHandler should exit
            if result.is_err() {
                running.store(false, Ordering::SeqCst);
            };
            result
        })?;

    let request_future = prepare_request(
        &url,
        &tx_closing,
        &tx_msg_closing,
        &db_conn,
        shutdown_signaler.clone(),
        reconnect.clone(),
    );

    // Send initial connection request
    tx_future.send(request_future).map_err(|err| {
        AppAuthHandlerError::StartUpError(format!("Unable to send connect request {}", err))
    })?;

    let running = shutdown_signaler.clone();
    let closing_msg_sender = tx_msg_closing.clone();

    // Thread that will listen to shutdown requests and forward them to the server
    // this thread is also responsible for managing reconnection attempts
    let join_handle_connection = thread::Builder::new()
        .name("GameroomDAppAuthHandlerConnectionManager".into())
        .spawn(move || {
            let result = loop {
                let sink = match try_recv(&rx_closing, running.clone()) {
                    Ok(sink) => {
                        match sink {
                            Some(sink) => sink,
                            None => break Ok(()), // no sink to receive
                        }
                    }
                    Err(err) => break Err(err),
                };

                let msg = match try_recv(&rx_msg_closing, running.clone()) {
                    Ok(msg) => {
                        match msg {
                            Some(msg) => msg,
                            None => break Ok(()), // no msg to receive
                        }
                    }
                    Err(err) => break Err(err),
                };

                if let Err(err) = sink.send(msg).wait() {
                    break Err(AppAuthHandlerError::ShutdownError(format!(
                        "Unable to send close message to server {}",
                        err
                    )));
                };

                if !reconnect.load(Ordering::SeqCst) || !running.load(Ordering::SeqCst) {
                    debug!("Exiting messaging loop");
                    break Ok(());
                }

                debug!(
                    "The client will try to reconnect in {} seconds",
                    RECONNECT_WAIT_TIME
                );

                thread::sleep(Duration::from_secs(RECONNECT_WAIT_TIME));

                if !running.load(Ordering::SeqCst) {
                    debug!("Exiting messaging loop");
                    break Ok(());
                }

                debug!("Sending reconnect request");
                let request_future = prepare_request(
                    &url,
                    &tx_closing,
                    &closing_msg_sender,
                    &db_conn,
                    running.clone(),
                    reconnect.clone(),
                );

                if let Err(err) = tx_future.send(request_future) {
                    break Err(AppAuthHandlerError::StartUpError(format!(
                        "Unable to send reconnect request message to {}",
                        err
                    )));
                };

                // reset reconnect flag
                reconnect.store(false, Ordering::SeqCst);
            };

            // if loop exits with an error, signal that AppAuthHandler should exit
            if result.is_err() {
                running.store(false, Ordering::SeqCst);
            };

            result
        })?;

    let do_shutdown = Box::new(move || {
        debug!("Shutting down application authentication handler");
        shutdown_signaler.store(false, Ordering::SeqCst);

        // Send shutdown message to listening thread
        tx_msg_closing
            .send(Message::Close(Some(CloseReason {
                code: CloseCode::Normal,
                description: Some("The client received shutdown signal".to_string()),
            })))
            .map_err(|err| {
                AppAuthHandlerError::ShutdownError(format!(
                    "Unable to send websocket close message {}",
                    err
                ))
            })?;

        Ok(())
    });

    Ok((
        AppAuthHandlerShutdownHandle { do_shutdown },
        ThreadJoinHandle(vec![join_handle_client, join_handle_connection]),
    ))
}

pub fn submit_vote(url: &str, vote: &CircuitProposalVote) -> Result<(), AppAuthHandlerError> {
    let serialized = serde_json::to_vec(vote)?;
    let body_stream = futures::stream::once::<_, std::io::Error>(Ok(serialized));
    let req = Request::builder()
        .uri(format!("{}/admin/vote", url))
        .method("POST")
        .body(Body::wrap_stream(body_stream))
        .map_err(|err| AppAuthHandlerError::RequestError(format!("{}", err)))?;

    let mut runtime = tokio::runtime::Runtime::new()?;
    let client = Client::new();
    runtime.block_on(client.request(req).then(|response| match response {
        Ok(res) => {
            let status = res.status();
            let body = res
                .into_body()
                .concat2()
                .wait()
                .map_err(|err| {
                    AppAuthHandlerError::SubmitVoteError(format!(
                        "The client encountered an error {}",
                        err
                    ))
                })?
                .to_vec();

            match status {
                StatusCode::ACCEPTED => Ok(()),
                _ => Err(AppAuthHandlerError::SubmitVoteError(format!(
                    "The server returned an error. Status: {}, {}",
                    status,
                    String::from_utf8(body)?
                ))),
            }
        }
        Err(err) => Err(AppAuthHandlerError::SubmitVoteError(format!(
            "The client encountered an error {}",
            err
        ))),
    }))
}

fn make_request(url: &str) -> Result<Request<Body>, AppAuthHandlerError> {
    Request::builder()
        .uri(format!("{}/ws/admin/register/gameroom", url))
        .header(header::UPGRADE, "websocket")
        .header(header::CONNECTION, "Upgrade")
        .header(header::SEC_WEBSOCKET_VERSION, "13")
        .header(header::SEC_WEBSOCKET_KEY, "13")
        .body(Body::empty())
        .map_err(|err| AppAuthHandlerError::RequestError(format!("{}", err)))
}

fn prepare_request(
    url: &str,
    tx_closing: &Sender<SplitSink<Framed<Upgraded, Codec>>>,
    closing_sender: &Sender<Message>,
    db_conn: &ConnectionPool,
    running: Arc<AtomicBool>,
    reconnect: Arc<AtomicBool>,
) -> Box<dyn Future<Item = (), Error = AppAuthHandlerError> + Send> {
    let tx_closing = tx_closing.clone();
    let closing_sender = closing_sender.clone();
    let db_conn = db_conn.clone();
    let request = match make_request(url) {
        Ok(req) => req,
        Err(err) => {
            let error: Box<FutureResult<_, _>> = Box::new(err.into());
            return error;
        }
    };

    Box::new(
        Client::new()
            .request(request)
            .and_then(|res| {
                if res.status() != StatusCode::SWITCHING_PROTOCOLS {
                    error!("The server didn't upgrade: {}", res.status());
                }
                res.into_body().on_upgrade()
            })
            .map_err(|e| {
                error!("The client returned an error: {}", e);
                AppAuthHandlerError::ClientError(format!("{}", e))
            })
            .and_then(move |upgraded| {
                let codec = Codec::new().client_mode();
                let framed = codec.framed(upgraded);
                let (sink, stream) = framed.split();

                if let Err(err) = tx_closing.send(sink) {
                    return Either::A(future::err(AppAuthHandlerError::StartUpError(format!(
                        "Unable to send send join handler addr {}",
                        err
                    ))));
                };

                let mut invalid_message_count = 0;
                // Read stream until shutdown signal is received
                Either::B(
                    stream
                        .map_err(|e| {
                            error!("The client returned an error: {}", e);
                            AppAuthHandlerError::ClientError(format!("{}", e))
                        })
                        .take_while(move |message| {
                            match message {
                                Frame::Text(msg) => {
                                    let msg_bytes = match msg {
                                        Some(bytes) => &bytes[..],
                                        None => &[],
                                    };

                                    match parse_message_bytes(msg_bytes) {
                                        Ok(admin_event) => {
                                            // reset invalid message count
                                            invalid_message_count = 0;
                                            if let Err(err) =
                                                process_admin_event(admin_event, &db_conn)
                                            {
                                                return err.into();
                                            }
                                        }
                                        Err(_) => {
                                            invalid_message_count += 1;
                                            if invalid_message_count > INVALID_MESSAGE_THRESHOLD {
                                                return handle_invalid_messages(
                                                    closing_sender.clone(),
                                                    reconnect.clone(),
                                                );
                                            }
                                        }
                                    }
                                }
                                Frame::Ping(msg) => {
                                    info!("Received Ping {}", msg);
                                    invalid_message_count = 0;
                                }
                                Frame::Close(msg) => {
                                    info!("Received close message {:?}", msg);
                                    invalid_message_count = 0;
                                    running.store(false, Ordering::SeqCst);
                                }
                                _ => {
                                    error!("Received invalid message: {:?}", message);
                                    invalid_message_count += 1;
                                    if invalid_message_count > INVALID_MESSAGE_THRESHOLD {
                                        return handle_invalid_messages(
                                            closing_sender.clone(),
                                            reconnect.clone(),
                                        );
                                    }
                                }
                            };

                            future::ok(running.load(Ordering::SeqCst))
                        })
                        // Transform stream into a future
                        .for_each(|_| future::ok(()))
                        .map_err(|e| {
                            error!("The client returned an error: {}", e);
                            AppAuthHandlerError::ClientError(format!("{}", e))
                        }),
                )
            }),
    )
}

fn try_recv<T>(
    receiver: &Receiver<T>,
    running: Arc<AtomicBool>,
) -> Result<Option<T>, AppAuthHandlerError> {
    loop {
        if !running.load(Ordering::SeqCst) {
            debug!("Exiting loop");
            break Ok(None);
        }

        thread::sleep(Duration::from_secs(1));
        match receiver.try_recv() {
            Ok(sink) => break Ok(Some(sink)),
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {
                break Err(AppAuthHandlerError::ShutdownError(
                    "Unable to receive sink".to_string(),
                ))
            }
        }
    }
}

fn handle_invalid_messages(
    sender: Sender<Message>,
    reconnect: Arc<AtomicBool>,
) -> FutureResult<bool, AppAuthHandlerError> {
    warn!("Received too many invalid messages from Splinterd websocket server. Disconnecting.");
    // signal to thread that it should try to reconnect
    reconnect.store(true, Ordering::SeqCst);
    match sender.send(Message::Close(Some(CloseReason {
        code: CloseCode::Unsupported,
        description: Some("Received too many invalid messages".to_string()),
    }))) {
        Ok(()) => future::ok(true),
        Err(err) => AppAuthHandlerError::ShutdownError(format!(
            "Unable to send websocket close message {}",
            err
        ))
        .into(),
    }
}

fn parse_message_bytes(bytes: &[u8]) -> Result<AdminServiceEvent, AppAuthHandlerError> {
    if bytes.is_empty() {
        error!("Received empty message");
        return Err(AppAuthHandlerError::InvalidMessageError(
            "Received empty message".to_string(),
        ));
    };
    let admin_event: AdminServiceEvent = serde_json::from_slice(bytes)?;
    Ok(admin_event)
}

fn process_admin_event(
    admin_event: AdminServiceEvent,
    pool: &ConnectionPool,
) -> Result<(), AppAuthHandlerError> {
    match admin_event {
        AdminServiceEvent::ProposalSubmitted(msg_proposal) => {
            let time = SystemTime::now();

            let proposal = parse_proposal(&msg_proposal, time);

            let gameroom = parse_gameroom(&msg_proposal.circuit, time);

            let services = parse_splinter_services(
                &msg_proposal.circuit_id,
                &msg_proposal.circuit.roster,
                time,
            );

            let nodes = parse_splinter_nodes(
                &msg_proposal.circuit_id,
                &msg_proposal.circuit.members,
                time,
            );

            let conn = &*pool.get()?;

            // insert proposal information in database tables in a single transaction
            conn.transaction::<_, _, _>(|| {
                let notification = helpers::create_new_notification(
                    "gameroom_proposal",
                    &proposal.requester,
                    &proposal.circuit_id,
                );
                helpers::insert_gameroom_notification(conn, &[notification])?;

                helpers::insert_gameroom(conn, gameroom)?;
                helpers::insert_gameroom_proposal(conn, proposal)?;
                helpers::insert_gameroom_services(conn, &services)?;
                helpers::insert_gameroom_members(conn, &nodes)?;

                debug!("Inserted new proposal into database");
                Ok(())
            })
        }
        AdminServiceEvent::ProposalVote(msg_vote) => {
            let proposal =
                get_pending_proposal_with_circuit_id(&pool, &msg_vote.ballot.circuit_id)?;
            let time = SystemTime::now();
            let vote = NewProposalVoteRecord {
                proposal_id: proposal.id,
                voter_public_key: String::from_utf8(msg_vote.signer_public_key)?,
                vote: format!("{:?}", msg_vote.ballot.vote),
                created_time: time,
            };
            let conn = &*pool.get()?;

            // insert vote and update proposal in a single database transaction
            conn.transaction::<_, _, _>(|| {
                let notification = helpers::create_new_notification(
                    "proposal_vote_record",
                    &vote.voter_public_key,
                    &vote.proposal_id.to_string(),
                );
                helpers::insert_gameroom_notification(conn, &[notification])?;
                helpers::update_gameroom_proposal_status(conn, proposal.id, &time, "Pending")?;
                helpers::insert_proposal_vote_record(conn, &[vote])?;

                debug!("Inserted new vote into database");
                Ok(())
            })
        }
        AdminServiceEvent::ProposalAccepted(msg_proposal) => {
            let proposal = get_pending_proposal_with_circuit_id(&pool, &msg_proposal.circuit_id)?;
            let time = SystemTime::now();
            let conn = &*pool.get()?;
            helpers::update_gameroom_proposal_status(conn, proposal.id, &time, "Accepted")?;
            debug!("Updated proposal to status 'Accepted'");
            Ok(())
        }
        AdminServiceEvent::ProposalRejected(msg_proposal) => {
            let proposal = get_pending_proposal_with_circuit_id(&pool, &msg_proposal.circuit_id)?;
            let time = SystemTime::now();
            let conn = &*pool.get()?;
            helpers::update_gameroom_proposal_status(conn, proposal.id, &time, "Rejected")?;
            helpers::update_gameroom_status(conn, &proposal.circuit_id, &time, "Rejected")?;
            debug!("Updated proposal to status 'Rejected'");
            Ok(())
        }
    }
}

fn parse_proposal(proposal: &CircuitProposal, timestamp: SystemTime) -> NewGameroomProposal {
    NewGameroomProposal {
        proposal_type: format!("{:?}", proposal.proposal_type),
        circuit_id: proposal.circuit_id.clone(),
        circuit_hash: proposal.circuit_hash.to_string(),
        requester: proposal.requester.clone(),
        status: "Pending".to_string(),
        created_time: timestamp,
        updated_time: timestamp,
    }
}

fn parse_gameroom(circuit: &CreateCircuit, timestamp: SystemTime) -> Gameroom {
    Gameroom {
        circuit_id: circuit.circuit_id.clone(),
        authorization_type: format!("{:?}", circuit.authorization_type),
        persistence: format!("{:?}", circuit.persistence),
        routes: format!("{:?}", circuit.routes),
        circuit_management_type: circuit.circuit_management_type.clone(),
        application_metadata: circuit.application_metadata.clone(),
        status: "Pending".to_string(),
        created_time: timestamp,
        updated_time: timestamp,
    }
}

fn parse_splinter_services(
    circuit_id: &str,
    splinter_services: &[SplinterService],
    timestamp: SystemTime,
) -> Vec<NewGameroomService> {
    splinter_services
        .iter()
        .map(|service| NewGameroomService {
            circuit_id: circuit_id.to_string(),
            service_id: service.service_id.to_string(),
            service_type: service.service_type.to_string(),
            allowed_nodes: service.allowed_nodes.clone(),
            status: "Pending".to_string(),
            created_time: timestamp,
            updated_time: timestamp,
        })
        .collect()
}

fn parse_splinter_nodes(
    circuit_id: &str,
    splinter_nodes: &[SplinterNode],
    timestamp: SystemTime,
) -> Vec<NewGameroomMember> {
    splinter_nodes
        .iter()
        .map(|node| NewGameroomMember {
            circuit_id: circuit_id.to_string(),
            node_id: node.node_id.to_string(),
            endpoint: node.endpoint.to_string(),
            status: "Pending".to_string(),
            created_time: timestamp,
            updated_time: timestamp,
        })
        .collect()
}

fn get_pending_proposal_with_circuit_id(
    pool: &ConnectionPool,
    circuit_id: &str,
) -> Result<GameroomProposal, AppAuthHandlerError> {
    helpers::fetch_gameroom_proposal_with_status(&*pool.get()?, &circuit_id, "Pending")?.ok_or_else(
        || {
            AppAuthHandlerError::DatabaseError(format!(
                "Could not find open proposal for circuit: {}",
                circuit_id
            ))
        },
    )
}

#[cfg(all(feature = "test-authorization-handler", test))]
mod test {
    use super::*;
    use diesel::{dsl::insert_into, prelude::*, RunQueryDsl};
    use gameroom_database::models::{
        GameroomMember, GameroomNotification, GameroomService, NewGameroomNotification,
        ProposalVoteRecord,
    };

    use libsplinter::admin::messages::{
        AuthorizationType, Ballot, CircuitProposalVote, CreateCircuit, PersistenceType,
        ProposalType, RouteType, Vote,
    };

    static DATABASE_URL: &str = "postgres://gameroom_test:gameroom_test@db-test:5432/gameroom_test";

    #[test]
    /// Tests if when receiving an admin message to CreateProposal the gameroom_proposal
    /// table is updated as expected
    fn test_process_proposal_submitted_message_update_proposal_table() {
        let pool: ConnectionPool = gameroom_database::create_connection_pool(DATABASE_URL)
            .expect("Failed to get database connection pool");

        clear_gameroom_table(&pool);
        clear_gameroom_notification_table(&pool);

        let message = get_submit_proposal_msg("my_circuit");
        process_admin_event(message, &pool).expect("Error processing message");

        let proposals = query_proposals_table(&pool);

        assert_eq!(proposals.len(), 1);

        let proposal = &proposals[0];
        let expected_proposal = get_gameroom_proposal("my_circuit", SystemTime::now());

        assert_eq!(proposal.proposal_type, expected_proposal.proposal_type);
        assert_eq!(proposal.circuit_id, expected_proposal.circuit_id);
        assert_eq!(proposal.circuit_hash, expected_proposal.circuit_hash);
        assert_eq!(proposal.requester, expected_proposal.requester);
        assert_eq!(proposal.status, expected_proposal.status);
    }

    #[test]
    /// Tests if when receiving an admin message to CreateProposal the gameroom
    /// table is updated as expected
    fn test_process_proposal_submitted_message_update_gameroom_table() {
        let pool: ConnectionPool = gameroom_database::create_connection_pool(DATABASE_URL)
            .expect("Failed to get database connection pool");

        clear_gameroom_table(&pool);
        clear_gameroom_notification_table(&pool);

        let message = get_submit_proposal_msg("my_circuit");
        process_admin_event(message, &pool).expect("Error processing message");

        let gamerooms = query_gameroom_table(&pool);

        assert_eq!(gamerooms.len(), 1);

        let gameroom = &gamerooms[0];
        let expected_gameroom = get_gameroom("my_circuit", SystemTime::now());

        assert_eq!(gameroom.circuit_id, expected_gameroom.circuit_id);
        assert_eq!(
            gameroom.authorization_type,
            expected_gameroom.authorization_type
        );
        assert_eq!(gameroom.persistence, expected_gameroom.persistence);
        assert_eq!(gameroom.routes, expected_gameroom.routes);
        assert_eq!(
            gameroom.circuit_management_type,
            expected_gameroom.circuit_management_type
        );
        assert_eq!(
            gameroom.application_metadata,
            expected_gameroom.application_metadata
        );
        assert_eq!(gameroom.status, expected_gameroom.status);
    }

    #[test]
    /// Tests if when receiving an admin message to CreateProposal the gameroom_member
    /// table is updated as expected
    fn test_process_proposal_submitted_message_update_member_table() {
        let pool: ConnectionPool = gameroom_database::create_connection_pool(DATABASE_URL)
            .expect("Failed to get database connection pool");

        clear_gameroom_table(&pool);
        clear_gameroom_notification_table(&pool);

        let message = get_submit_proposal_msg("my_circuit");
        process_admin_event(message, &pool).expect("Error processing message");

        let members = query_gameroom_members_table(&pool);

        assert_eq!(members.len(), 1);

        let node = &members[0];
        let expected_node = get_new_gameroom_member("my_circuit", SystemTime::now());

        assert_eq!(node.node_id, expected_node.node_id);
        assert_eq!(node.endpoint, expected_node.endpoint);
    }

    #[test]
    /// Tests if when receiving an admin message to CreateProposal the gameroom_service
    /// table is updated as expected
    fn test_process_proposal_submitted_message_update_service_table() {
        let pool: ConnectionPool = gameroom_database::create_connection_pool(DATABASE_URL)
            .expect("Failed to get database connection pool");

        clear_gameroom_table(&pool);
        clear_gameroom_notification_table(&pool);

        let message = get_submit_proposal_msg("my_circuit");
        process_admin_event(message, &pool).expect("Error processing message");

        let services = query_gameroom_service_table(&pool);

        assert_eq!(services.len(), 1);

        let service = &services[0];
        let expected_service = get_new_gameroom_service("my_circuit", SystemTime::now());

        assert_eq!(service.service_id, expected_service.service_id);
        assert_eq!(service.service_type, expected_service.service_type);
        assert_eq!(service.allowed_nodes, expected_service.allowed_nodes);
    }

    #[test]
    /// Tests if when receiving an admin message to CreateProposal the gameroom_notification
    /// table is updated as expected
    fn test_process_proposal_submitted_message_update_notification_table() {
        let pool: ConnectionPool = gameroom_database::create_connection_pool(DATABASE_URL)
            .expect("Failed to get database connection pool");

        clear_gameroom_table(&pool);
        clear_gameroom_notification_table(&pool);

        let message = get_submit_proposal_msg("my_circuit");
        process_admin_event(message, &pool).expect("Error processing message");

        let notifications = query_gameroom_notification_table(&pool);

        assert_eq!(notifications.len(), 1);

        let notification = &notifications[0];
        let expected_notification =
            get_new_gameroom_notification_proposal("my_circuit", SystemTime::now());

        assert_eq!(
            notification.notification_type,
            expected_notification.notification_type
        );
        assert_eq!(notification.requester, expected_notification.requester);
        assert_eq!(notification.target, expected_notification.target);
        assert_eq!(notification.read, expected_notification.read);
    }

    #[test]
    /// Tests if when receiving an admin message ProposalAccepted the gameroom_proposal
    /// table is updated as expected
    fn test_process_proposal_accepted_message_ok() {
        let pool: ConnectionPool = gameroom_database::create_connection_pool(DATABASE_URL)
            .expect("Failed to get database connection pool");

        clear_gameroom_table(&pool);
        clear_gameroom_notification_table(&pool);

        let created_time = SystemTime::now();

        // insert gameroom into database
        insert_gameroom_table(&pool, get_gameroom("my_circuit", created_time.clone()));

        // insert pending proposal into database
        insert_proposals_table(
            &pool,
            get_gameroom_proposal("my_circuit", created_time.clone()),
        );

        let accept_message = get_accept_proposal_msg("my_circuit");

        // accept proposal
        process_admin_event(accept_message, &pool).expect("Error processing message");

        let proposals = query_proposals_table(&pool);

        assert_eq!(proposals.len(), 1);

        let proposal = &proposals[0];

        // Check proposal updated_time changed
        assert!(proposal.updated_time > created_time);
        // Check status was changed to accepted
        assert_eq!(proposal.status, "Accepted");
    }

    #[test]
    /// Tests if when receiving an admin message ProposalAccepted an error is returned
    /// if a pending proposal for that circuit is not found
    fn test_process_proposal_accepted_message_err() {
        let pool: ConnectionPool = gameroom_database::create_connection_pool(DATABASE_URL)
            .expect("Failed to get database connection pool");

        clear_gameroom_table(&pool);
        clear_gameroom_notification_table(&pool);

        let accept_message = get_accept_proposal_msg("my_circuit");

        // accept proposal
        match process_admin_event(accept_message, &pool) {
            Ok(()) => panic!("Pending proposal for circuit is missing, error should be returned"),
            Err(AppAuthHandlerError::DatabaseError(msg)) => {
                assert!(msg.contains("Could not find open proposal for circuit: my_circuit"));
            }
            Err(err) => panic!("Should have gotten DatabaseError error but got {}", err),
        }
    }

    #[test]
    /// Tests if when receiving an admin message ProposalRejected the gameroom_proposal and
    /// gameroom tables are updated as expected
    fn test_process_proposal_rejected_message_ok() {
        let pool: ConnectionPool = gameroom_database::create_connection_pool(DATABASE_URL)
            .expect("Failed to get database connection pool");

        clear_gameroom_table(&pool);
        clear_gameroom_notification_table(&pool);

        let created_time = SystemTime::now();

        // insert gameroom into database
        insert_gameroom_table(&pool, get_gameroom("my_circuit", created_time.clone()));

        // insert pending proposal into database
        insert_proposals_table(
            &pool,
            get_gameroom_proposal("my_circuit", created_time.clone()),
        );

        let rejected_message = get_reject_proposal_msg("my_circuit");

        // reject proposal
        process_admin_event(rejected_message, &pool).expect("Error processing message");

        let proposals = query_proposals_table(&pool);

        assert_eq!(proposals.len(), 1);

        let proposal = &proposals[0];

        // Check proposal updated_time changed
        assert!(proposal.updated_time > created_time);
        // Check status was changed to rejected
        assert_eq!(proposal.status, "Rejected");

        let gamerooms = query_gameroom_table(&pool);

        assert_eq!(gamerooms.len(), 1);

        let gameroom = &gamerooms[0];

        // Check gameroom updated_time changed
        assert!(gameroom.updated_time > created_time);
        // Check status was changed to rejected
        assert_eq!(gameroom.status, "Rejected");
    }

    #[test]
    /// Tests if when receiving an admin message ProposalRejected an error is returned
    /// if a pending proposal for that circuit is not found
    fn test_process_proposal_rejected_message_err() {
        let pool: ConnectionPool = gameroom_database::create_connection_pool(DATABASE_URL)
            .expect("Failed to get database connection pool");

        clear_gameroom_table(&pool);
        clear_gameroom_notification_table(&pool);

        let rejected_message = get_reject_proposal_msg("my_circuit");

        // reject proposal
        match process_admin_event(rejected_message, &pool) {
            Ok(()) => panic!("Pending proposal for circuit is missing, error should be returned"),
            Err(AppAuthHandlerError::DatabaseError(msg)) => {
                assert!(msg.contains("Could not find open proposal for circuit: my_circuit"));
            }
            Err(err) => panic!("Should have gotten DatabaseError error but got {}", err),
        }
    }

    #[test]
    /// Tests if when receiving an admin message ProposalVote the gameroom_proposal and
    /// proposal_vote_record tables are updated as expected
    fn test_process_proposal_vote_message_ok() {
        let pool: ConnectionPool = gameroom_database::create_connection_pool(DATABASE_URL)
            .expect("Failed to get database connection pool");

        clear_gameroom_table(&pool);
        clear_gameroom_notification_table(&pool);

        let created_time = SystemTime::now();

        // insert gameroom into database
        insert_gameroom_table(&pool, get_gameroom("my_circuit", created_time.clone()));

        // insert pending proposal into database
        insert_proposals_table(
            &pool,
            get_gameroom_proposal("my_circuit", created_time.clone()),
        );

        let vote_message = get_vote_proposal_msg("my_circuit");

        // vote proposal
        process_admin_event(vote_message, &pool).expect("Error processing message");

        let proposals = query_proposals_table(&pool);

        assert_eq!(proposals.len(), 1);

        let proposal = &proposals[0];

        // Check proposal updated_time changed
        assert!(proposal.updated_time > created_time);

        let votes = query_votes_table(&pool);
        assert_eq!(votes.len(), 1);

        let vote = &votes[0];
        let expected_vote = get_new_vote_record(proposal.id, SystemTime::now());
        assert_eq!(vote.voter_public_key, expected_vote.voter_public_key);
        assert_eq!(vote.vote, expected_vote.vote);
        assert_eq!(vote.created_time, proposal.updated_time);
    }

    #[test]
    /// Tests if when receiving an admin message to ProposalVote the gameroom_notification
    /// table is updated as expected
    fn test_process_proposal_vote_message_update_notification_table() {
        let pool: ConnectionPool = gameroom_database::create_connection_pool(DATABASE_URL)
            .expect("Failed to get database connection pool");

        clear_gameroom_table(&pool);
        clear_gameroom_notification_table(&pool);

        let created_time = SystemTime::now();

        // insert gameroom into database
        insert_gameroom_table(&pool, get_gameroom("my_circuit", created_time.clone()));

        // insert pending proposal into database
        insert_proposals_table(
            &pool,
            get_gameroom_proposal("my_circuit", created_time.clone()),
        );

        let vote_message = get_vote_proposal_msg("my_circuit");

        // vote proposal
        process_admin_event(vote_message, &pool).expect("Error processing message");

        let notifications = query_gameroom_notification_table(&pool);

        assert_eq!(notifications.len(), 1);

        let votes = query_votes_table(&pool);
        assert_eq!(votes.len(), 1);

        let vote = &votes[0];

        let notification = &notifications[0];
        let expected_notification =
            get_new_gameroom_notification_vote(vote.proposal_id, SystemTime::now());

        assert_eq!(
            notification.notification_type,
            expected_notification.notification_type
        );
        assert_eq!(notification.requester, expected_notification.requester);
        assert_eq!(notification.target, expected_notification.target);
        assert_eq!(notification.read, expected_notification.read);
    }

    #[test]
    /// Tests if when receiving an admin message ProposalVote an error is returned
    /// if a pending proposal for that circuit is not found
    fn test_process_proposal_vote_message_err() {
        let pool: ConnectionPool = gameroom_database::create_connection_pool(DATABASE_URL)
            .expect("Failed to get database connection pool");

        clear_gameroom_table(&pool);
        clear_gameroom_notification_table(&pool);

        let vote_message = get_vote_proposal_msg("my_circuit");

        // vote proposal
        match process_admin_event(vote_message, &pool) {
            Ok(()) => panic!("Pending proposal for circuit is missing, error should be returned"),
            Err(AppAuthHandlerError::DatabaseError(msg)) => {
                assert!(msg.contains("Could not find open proposal for circuit: my_circuit"));
            }
            Err(err) => panic!("Should have gotten DatabaseError error but got {}", err),
        }
    }

    #[test]
    /// Tests if the admin message CreateProposal to a database GameroomProposal is successful
    fn test_parse_proposal() {
        let time = SystemTime::now();
        let proposal = parse_proposal(&get_msg_proposal("my_circuit"), time.clone());

        assert_eq!(proposal, get_gameroom_proposal("my_circuit", time.clone()))
    }

    #[test]
    /// Tests if the admin message CreateCircuit to a database Gameroom is successful
    fn test_parse_gameroom() {
        let time = SystemTime::now();
        let gameroom = parse_gameroom(&get_create_circuit_msg("my_circuit"), time);

        assert_eq!(gameroom, get_gameroom("my_circuit", time.clone()))
    }

    #[test]
    /// Tests if the admin message SplinterService to a database NewGameroomService is successful
    fn test_parse_gameroom_service() {
        let time = SystemTime::now();
        let services = parse_splinter_services(
            "my_circuit",
            &get_msg_proposal("my_circuit").circuit.roster,
            time,
        );

        assert_eq!(services, vec![get_new_gameroom_service("my_circuit", time)]);
    }

    #[test]
    /// Tests if the admin message SplinterNode to a database NewGameroomMember is successful
    fn test_parse_gameroom_member() {
        let time = SystemTime::now();
        let members = parse_splinter_nodes(
            "my_circuit",
            &get_msg_proposal("my_circuit").circuit.members,
            time,
        );

        assert_eq!(members, vec![get_new_gameroom_member("my_circuit", time)]);
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

    fn get_msg_proposal(circuit_id: &str) -> CircuitProposal {
        CircuitProposal {
            proposal_type: ProposalType::Create,
            circuit_id: circuit_id.to_string(),
            circuit_hash: "8e066d41911817a42ab098eda35a2a2b11e93c753bc5ecc3ffb3e99ed99ada0d"
                .to_string(),
            circuit: get_create_circuit_msg(circuit_id),
            votes: vec![],
            requester: "acme_corp".to_string(),
        }
    }

    fn get_ballot(circuit_id: &str) -> Ballot {
        Ballot {
            circuit_id: circuit_id.to_string(),
            circuit_hash: "8e066d41911817a42ab098eda35a2a2b11e93c753bc5ecc3ffb3e99ed99ada0d"
                .to_string(),
            vote: Vote::Accept,
        }
    }

    fn get_msg_circuit_proposal_vote(circuit_id: &str) -> CircuitProposalVote {
        CircuitProposalVote {
            ballot: get_ballot(circuit_id),
            ballot_signature: vec![65, 65, 65, 65, 66, 51, 78],
            signer_public_key: vec![73, 119, 65, 65, 65, 81],
        }
    }

    fn get_reject_proposal_msg(circuit_id: &str) -> AdminServiceEvent {
        AdminServiceEvent::ProposalRejected(get_msg_proposal(circuit_id))
    }

    fn get_accept_proposal_msg(circuit_id: &str) -> AdminServiceEvent {
        AdminServiceEvent::ProposalAccepted(get_msg_proposal(circuit_id))
    }

    fn get_vote_proposal_msg(circuit_id: &str) -> AdminServiceEvent {
        AdminServiceEvent::ProposalVote(get_msg_circuit_proposal_vote(circuit_id))
    }

    fn get_submit_proposal_msg(circuit_id: &str) -> AdminServiceEvent {
        AdminServiceEvent::ProposalSubmitted(get_msg_proposal(circuit_id))
    }

    fn get_gameroom_proposal(circuit_id: &str, timestamp: SystemTime) -> NewGameroomProposal {
        NewGameroomProposal {
            proposal_type: "Create".to_string(),
            circuit_id: circuit_id.to_string(),
            circuit_hash: "8e066d41911817a42ab098eda35a2a2b11e93c753bc5ecc3ffb3e99ed99ada0d"
                .to_string(),
            requester: "acme_corp".to_string(),
            status: "Pending".to_string(),
            created_time: timestamp,
            updated_time: timestamp,
        }
    }

    fn get_gameroom(circuit_id: &str, timestamp: SystemTime) -> Gameroom {
        Gameroom {
            circuit_id: circuit_id.to_string(),
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

    fn get_new_vote_record(proposal_id: i64, timestamp: SystemTime) -> NewProposalVoteRecord {
        NewProposalVoteRecord {
            proposal_id,
            voter_public_key: "IwAAAQ".to_string(),
            vote: "Accept".to_string(),
            created_time: timestamp,
        }
    }

    fn get_new_gameroom_service(circuit_id: &str, timestamp: SystemTime) -> NewGameroomService {
        NewGameroomService {
            circuit_id: circuit_id.to_string(),
            service_id: "scabbard_123".to_string(),
            service_type: "scabbard".to_string(),
            allowed_nodes: vec!["acme_corp".to_string()],
            status: "Pending".to_string(),
            created_time: timestamp,
            updated_time: timestamp,
        }
    }

    fn get_new_gameroom_member(circuit_id: &str, timestamp: SystemTime) -> NewGameroomMember {
        NewGameroomMember {
            circuit_id: circuit_id.to_string(),
            node_id: "Node-123".to_string(),
            endpoint: "127.0.0.1:8282".to_string(),
            status: "Pending".to_string(),
            created_time: timestamp,
            updated_time: timestamp,
        }
    }

    fn get_new_gameroom_notification_proposal(
        circuit_id: &str,
        timestamp: SystemTime,
    ) -> NewGameroomNotification {
        NewGameroomNotification {
            notification_type: "gameroom_proposal".to_string(),
            requester: "acme_corp".to_string(),
            target: circuit_id.to_string(),
            created_time: timestamp,
            read: false,
        }
    }

    fn get_new_gameroom_notification_vote(
        proposal_id: i64,
        timestamp: SystemTime,
    ) -> NewGameroomNotification {
        NewGameroomNotification {
            notification_type: "proposal_vote_record".to_string(),
            requester: "IwAAAQ".to_string(),
            target: proposal_id.to_string(),
            created_time: timestamp,
            read: false,
        }
    }

    fn query_votes_table(pool: &ConnectionPool) -> Vec<ProposalVoteRecord> {
        use gameroom_database::schema::proposal_vote_record;

        let conn = &*pool.get().expect("Error getting db connection");
        proposal_vote_record::table
            .select(proposal_vote_record::all_columns)
            .load::<ProposalVoteRecord>(conn)
            .expect("Error fetching vote records")
    }

    fn query_gameroom_members_table(pool: &ConnectionPool) -> Vec<GameroomMember> {
        use gameroom_database::schema::gameroom_member;

        let conn = &*pool.get().expect("Error getting db connection");
        gameroom_member::table
            .select(gameroom_member::all_columns)
            .load::<GameroomMember>(conn)
            .expect("Error fetching circuit members")
    }

    fn query_gameroom_service_table(pool: &ConnectionPool) -> Vec<GameroomService> {
        use gameroom_database::schema::gameroom_service;

        let conn = &*pool.get().expect("Error getting db connection");
        gameroom_service::table
            .select(gameroom_service::all_columns)
            .load::<GameroomService>(conn)
            .expect("Error fetching circuit members")
    }

    fn query_proposals_table(pool: &ConnectionPool) -> Vec<GameroomProposal> {
        use gameroom_database::schema::gameroom_proposal;

        let conn = &*pool.get().expect("Error getting db connection");
        gameroom_proposal::table
            .select(gameroom_proposal::all_columns)
            .load::<GameroomProposal>(conn)
            .expect("Error fetching proposals")
    }

    fn query_gameroom_table(pool: &ConnectionPool) -> Vec<Gameroom> {
        use gameroom_database::schema::gameroom;

        let conn = &*pool.get().expect("Error getting db connection");
        gameroom::table
            .select(gameroom::all_columns)
            .load::<Gameroom>(conn)
            .expect("Error fetching proposals")
    }

    fn query_gameroom_notification_table(pool: &ConnectionPool) -> Vec<GameroomNotification> {
        use gameroom_database::schema::gameroom_notification;

        let conn = &*pool.get().expect("Error getting db connection");
        gameroom_notification::table
            .select(gameroom_notification::all_columns)
            .load::<GameroomNotification>(conn)
            .expect("Error fetching proposals")
    }

    fn insert_proposals_table(pool: &ConnectionPool, proposal: NewGameroomProposal) {
        use gameroom_database::schema::gameroom_proposal;

        let conn = &*pool.get().expect("Error getting db connection");
        insert_into(gameroom_proposal::table)
            .values(&vec![proposal])
            .execute(conn)
            .map(|_| ())
            .expect("Failed to insert proposal in table")
    }

    fn insert_gameroom_table(pool: &ConnectionPool, gameroom: Gameroom) {
        use gameroom_database::schema::gameroom;

        let conn = &*pool.get().expect("Error getting db connection");
        insert_into(gameroom::table)
            .values(&vec![gameroom])
            .execute(conn)
            .map(|_| ())
            .expect("Failed to insert proposal in table")
    }

    fn clear_gameroom_table(pool: &ConnectionPool) {
        use gameroom_database::schema::gameroom::dsl::*;

        let conn = &*pool.get().expect("Error getting db connection");
        diesel::delete(gameroom)
            .execute(conn)
            .expect("Error cleaning gameroom table");
    }

    fn clear_gameroom_notification_table(pool: &ConnectionPool) {
        use gameroom_database::schema::gameroom_notification::dsl::*;

        let conn = &*pool.get().expect("Error getting db connection");
        diesel::delete(gameroom_notification)
            .execute(conn)
            .expect("Error cleaning gameroom_notification table");
    }

}
