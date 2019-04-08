// Copyright 2019 Bitwise IO, Inc.
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

use crate::database::{helpers as db, models::Agent, ConnectionPool};
use crate::rest_api::{error::RestApiResponseError, AppState};
use actix::{Actor, Context, Handler, Message, SyncContext};
use actix_web::{AsyncResponder, HttpMessage, HttpRequest, HttpResponse, Query, State};
use futures::future;
use futures::future::Future;
use sawtooth_sdk::messages::batch::{Batch, BatchList};
use sawtooth_sdk::messages::client_batch_submit::{
    ClientBatchStatus, ClientBatchStatusRequest, ClientBatchStatusResponse,
    ClientBatchStatusResponse_Status, ClientBatchSubmitRequest, ClientBatchSubmitResponse,
    ClientBatchSubmitResponse_Status,
};
use sawtooth_sdk::messages::validator::Message_MessageType;
use sawtooth_sdk::messaging::stream::MessageSender;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::time::Duration;
use url::Url;
use uuid::Uuid;

const DEFAULT_TIME_OUT: u32 = 300; // Max timeout 300 seconds == 5 minutes

pub struct DbExecutor {
    connection_pool: ConnectionPool,
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

impl DbExecutor {
    pub fn new(connection_pool: ConnectionPool) -> DbExecutor {
        DbExecutor { connection_pool }
    }
}

pub struct SawtoothMessageSender {
    sender: Box<dyn MessageSender>,
}

impl Actor for SawtoothMessageSender {
    type Context = Context<Self>;
}

impl SawtoothMessageSender {
    pub fn new(sender: Box<dyn MessageSender>) -> SawtoothMessageSender {
        SawtoothMessageSender { sender }
    }
}

struct SubmitBatches {
    batch_list: BatchList,
    response_url: Url,
}

impl Message for SubmitBatches {
    type Result = Result<BatchStatusLink, RestApiResponseError>;
}

struct BatchStatuses {
    batch_ids: Vec<String>,
    wait: Option<u32>,
}

impl Message for BatchStatuses {
    type Result = Result<Vec<BatchStatus>, RestApiResponseError>;
}

#[derive(Serialize, Deserialize, Debug)]
struct BatchStatus {
    id: String,
    invalid_transactions: Vec<HashMap<String, String>>,
    status: String,
}

impl BatchStatus {
    pub fn from_proto(proto: &ClientBatchStatus) -> BatchStatus {
        BatchStatus {
            id: proto.get_batch_id().to_string(),
            invalid_transactions: proto
                .get_invalid_transactions()
                .iter()
                .map(|txn| {
                    let mut invalid_transaction_info = HashMap::new();
                    invalid_transaction_info
                        .insert("id".to_string(), txn.get_transaction_id().to_string());
                    invalid_transaction_info
                        .insert("message".to_string(), txn.get_message().to_string());
                    invalid_transaction_info.insert(
                        "extended_data".to_string(),
                        base64::encode(txn.get_extended_data()),
                    );
                    invalid_transaction_info
                })
                .collect(),
            status: format!("{:?}", proto.get_status()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct BatchStatusResponse {
    data: Vec<BatchStatus>,
    link: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchStatusLink {
    pub link: String,
}

impl Handler<SubmitBatches> for SawtoothMessageSender {
    type Result = Result<BatchStatusLink, RestApiResponseError>;

    fn handle(&mut self, msg: SubmitBatches, _: &mut Context<Self>) -> Self::Result {
        let mut client_submit_request = ClientBatchSubmitRequest::new();
        client_submit_request.set_batches(protobuf::RepeatedField::from_vec(
            msg.batch_list.get_batches().to_vec(),
        ));

        let response_status: ClientBatchSubmitResponse = query_validator(
            &*self.sender,
            Message_MessageType::CLIENT_BATCH_SUBMIT_REQUEST,
            &client_submit_request,
        )?;

        match process_validator_response(response_status.get_status()) {
            Ok(_) => {
                let batch_query = msg
                    .batch_list
                    .get_batches()
                    .iter()
                    .map(Batch::get_header_signature)
                    .collect::<Vec<_>>()
                    .join(",");

                let mut response_url = msg.response_url.clone();
                response_url.set_query(Some(&format!("id={}", batch_query)));

                Ok(BatchStatusLink {
                    link: response_url.to_string(),
                })
            }
            Err(err) => Err(err),
        }
    }
}

impl Handler<BatchStatuses> for SawtoothMessageSender {
    type Result = Result<Vec<BatchStatus>, RestApiResponseError>;

    fn handle(&mut self, msg: BatchStatuses, _: &mut Context<Self>) -> Self::Result {
        let mut batch_status_request = ClientBatchStatusRequest::new();
        batch_status_request.set_batch_ids(protobuf::RepeatedField::from_vec(msg.batch_ids));
        match msg.wait {
            Some(wait_time) => {
                batch_status_request.set_wait(true);
                batch_status_request.set_timeout(wait_time);
            }
            None => {
                batch_status_request.set_wait(false);
            }
        }

        let response_status: ClientBatchStatusResponse = query_validator(
            &*self.sender,
            Message_MessageType::CLIENT_BATCH_STATUS_REQUEST,
            &batch_status_request,
        )?;

        process_batch_status_response(response_status)
    }
}

pub fn submit_batches(
    (req, state): (HttpRequest<AppState>, State<AppState>),
) -> impl Future<Item = HttpResponse, Error = RestApiResponseError> {
    req.body().from_err().and_then(
        move |body| -> Box<Future<Item = HttpResponse, Error = RestApiResponseError>> {
            let batch_list: BatchList = match protobuf::parse_from_bytes(&*body) {
                Ok(batch_list) => batch_list,
                Err(err) => {
                    return Box::new(future::err(RestApiResponseError::BadRequest(format!(
                        "Protobuf message was badly formatted. {}",
                        err.to_string()
                    ))));
                }
            };
            let response_url = match req.url_for_static("batch_statuses") {
                Ok(url) => url,
                Err(err) => return Box::new(future::err(err.into())),
            };

            let res = state
                .sawtooth_connection
                .send(SubmitBatches {
                    batch_list,
                    response_url,
                })
                .from_err()
                .and_then(|res| match res {
                    Ok(link) => Ok(HttpResponse::Ok().json(link)),
                    Err(err) => Err(err),
                });
            Box::new(res)
        },
    )
}

#[derive(Deserialize, Debug)]
struct Params {
    id: Vec<String>,
}

pub fn get_batch_statuses(
    (state, query, req): (
        State<AppState>,
        Query<HashMap<String, String>>,
        HttpRequest<AppState>,
    ),
) -> Box<Future<Item = HttpResponse, Error = RestApiResponseError>> {
    let batch_ids = match query.get("id") {
        Some(ids) => ids.split(',').map(|id| id.to_string()).collect(),
        None => {
            return future::err(RestApiResponseError::BadRequest(
                "Request for statuses missing id query.".to_string(),
            ))
            .responder();
        }
    };

    // Max wait time allowed is 95% of network's configured timeout
    let max_wait_time = (DEFAULT_TIME_OUT * 95) / 100;

    let wait = match query.get("wait") {
        Some(wait_time) => {
            if wait_time == "false" {
                None
            } else {
                match wait_time.parse::<u32>() {
                    Ok(wait_time) => {
                        if wait_time > max_wait_time {
                            Some(max_wait_time)
                        } else {
                            Some(wait_time)
                        }
                    }
                    Err(_) => {
                        return future::err(RestApiResponseError::BadRequest(format!(
                            "Query wait has invalid value {}. \
                             It should set to false or a a time in seconds to wait for the commit",
                            wait_time
                        )))
                        .responder();
                    }
                }
            }
        }

        None => Some(max_wait_time),
    };

    let response_url = match req.url_for_static("batch_statuses") {
        Ok(url) => format!("{}?{}", url, req.query_string()),
        Err(err) => return Box::new(future::err(err.into())),
    };

    state
        .sawtooth_connection
        .send(BatchStatuses { batch_ids, wait })
        .from_err()
        .and_then(|res| match res {
            Ok(batch_statuses) => Ok(HttpResponse::Ok().json(BatchStatusResponse {
                data: batch_statuses,
                link: response_url,
            })),
            Err(err) => Err(err),
        })
        .responder()
}

fn query_validator<T: protobuf::Message, C: protobuf::Message>(
    sender: &dyn MessageSender,
    message_type: Message_MessageType,
    message: &C,
) -> Result<T, RestApiResponseError> {
    let content = protobuf::Message::write_to_bytes(message).map_err(|err| {
        RestApiResponseError::RequestHandlerError(format!(
            "Failed to serialize batch submit request. {}",
            err.to_string()
        ))
    })?;

    let correlation_id = Uuid::new_v4().to_string();

    let mut response_future = sender
        .send(message_type, &correlation_id, &content)
        .map_err(|err| {
            RestApiResponseError::SawtoothConnectionError(format!(
                "Failed to send message to validator. {}",
                err.to_string()
            ))
        })?;

    protobuf::parse_from_bytes(
        response_future
            .get_timeout(Duration::new(DEFAULT_TIME_OUT.into(), 0))
            .map_err(|err| RestApiResponseError::RequestHandlerError(err.to_string()))?
            .get_content(),
    )
    .map_err(|err| {
        RestApiResponseError::RequestHandlerError(format!(
            "Failed to parse validator response from bytes. {}",
            err.to_string()
        ))
    })
}

fn process_validator_response(
    status: ClientBatchSubmitResponse_Status,
) -> Result<(), RestApiResponseError> {
    match status {
        ClientBatchSubmitResponse_Status::OK => Ok(()),
        ClientBatchSubmitResponse_Status::INVALID_BATCH => Err(RestApiResponseError::BadRequest(
            "The submitted BatchList was rejected by the validator. It was '
            'poorly formed, or has an invalid signature."
                .to_string(),
        )),
        _ => Err(RestApiResponseError::SawtoothValidatorResponseError(
            format!("Validator responded with error {:?}", status),
        )),
    }
}

fn process_batch_status_response(
    response: ClientBatchStatusResponse,
) -> Result<Vec<BatchStatus>, RestApiResponseError> {
    let status = response.get_status();
    match status {
        ClientBatchStatusResponse_Status::OK => Ok(response
            .get_batch_statuses()
            .iter()
            .map(BatchStatus::from_proto)
            .collect()),
        ClientBatchStatusResponse_Status::INVALID_ID => Err(RestApiResponseError::BadRequest(
            "Blockchain items are identified by 128 character hex-strings. A submitted \
             batch id was invalid"
                .to_string(),
        )),
        _ => Err(RestApiResponseError::SawtoothValidatorResponseError(
            format!("Validator responded with error {:?}", status),
        )),
    }
}
#[derive(Debug, Serialize)]
struct AgentSlice {
    public_key: String,
    org_id: String,
    active: bool,
    roles: Vec<String>,
    metadata: Vec<JsonValue>,
}

impl AgentSlice {
    pub fn from_agent(agent: &Agent) -> Self {
        Self {
            public_key: agent.public_key.clone(),
            org_id: agent.org_id.clone(),
            active: agent.active,
            roles: agent.roles.clone(),
            metadata: agent.metadata.clone(),
        }
    }
}

struct ListAgents;

impl Message for ListAgents {
    type Result = Result<Vec<AgentSlice>, RestApiResponseError>;
}

impl Handler<ListAgents> for DbExecutor {
    type Result = Result<Vec<AgentSlice>, RestApiResponseError>;

    fn handle(&mut self, _msg: ListAgents, _: &mut SyncContext<Self>) -> Self::Result {
        let fetched_agents = db::get_agents(&*self.connection_pool.get()?)?
            .iter()
            .map(|agent| AgentSlice::from_agent(agent))
            .collect();

        Ok(fetched_agents)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rest_api::AppState;
    use actix_web::{http, http::Method, test::TestServer, HttpMessage};
    use futures::future::Future;
    use sawtooth_sdk::messages::batch::{Batch, BatchList};
    use sawtooth_sdk::messages::client_batch_submit::{
        ClientBatchStatus, ClientBatchStatusRequest, ClientBatchStatusResponse,
        ClientBatchStatusResponse_Status, ClientBatchStatus_Status, ClientBatchSubmitResponse,
        ClientBatchSubmitResponse_Status,
    };
    use sawtooth_sdk::messages::validator::{Message, Message_MessageType};

    use sawtooth_sdk::messaging::stream::{MessageFuture, MessageSender, SendError};
    use std::sync::mpsc::channel;

    static BATCH_ID_1: &str = "batch_1";
    static BATCH_ID_2: &str = "batch_2";
    static BATCH_ID_3: &str = "batch_3";

    struct MockMessageSender {
        response_type: ResponseType,
    }

    #[derive(Clone, Copy, Debug)]
    enum ResponseType {
        ClientBatchStatusResponseOK,
        ClientBatchStatusResponseInvalidId,
        ClientBatchStatusResponseInternalError,
        ClientBatchSubmitResponseOK,
        ClientBatchSubmitResponseInvalidBatch,
        ClientBatchSubmitResponseInternalError,
    }

    impl MockMessageSender {
        fn new_boxed(response_type: ResponseType) -> Box<MockMessageSender> {
            Box::new(MockMessageSender { response_type })
        }
    }

    impl MessageSender for MockMessageSender {
        fn send(
            &self,
            destination: Message_MessageType,
            correlation_id: &str,
            contents: &[u8],
        ) -> Result<MessageFuture, SendError> {
            let mut mock_validator_response = Message::new();
            mock_validator_response.set_message_type(destination);
            mock_validator_response.set_correlation_id(correlation_id.to_string());
            match &self.response_type {
                ResponseType::ClientBatchStatusResponseOK => {
                    let request: ClientBatchStatusRequest =
                        protobuf::parse_from_bytes(contents).unwrap();
                    if request.get_batch_ids().len() <= 1 {
                        mock_validator_response.set_content(get_batch_statuses_response_one_id())
                    } else {
                        mock_validator_response
                            .set_content(get_batch_statuses_response_multiple_ids())
                    }
                }
                ResponseType::ClientBatchStatusResponseInvalidId => {
                    mock_validator_response.set_content(get_batch_statuses_response_invalid_id())
                }
                ResponseType::ClientBatchStatusResponseInternalError => mock_validator_response
                    .set_content(get_batch_statuses_response_validator_internal_error()),
                ResponseType::ClientBatchSubmitResponseOK => mock_validator_response.set_content(
                    get_submit_batches_response(ClientBatchSubmitResponse_Status::OK),
                ),
                ResponseType::ClientBatchSubmitResponseInvalidBatch => mock_validator_response
                    .set_content(get_submit_batches_response(
                        ClientBatchSubmitResponse_Status::INVALID_BATCH,
                    )),
                ResponseType::ClientBatchSubmitResponseInternalError => mock_validator_response
                    .set_content(get_submit_batches_response(
                        ClientBatchSubmitResponse_Status::INTERNAL_ERROR,
                    )),
            }

            let mock_resut = Ok(mock_validator_response);
            let (send, recv) = channel();
            send.send(mock_resut).unwrap();
            Ok(MessageFuture::new(recv))
        }

        fn reply(
            &self,
            _destination: Message_MessageType,
            _correlation_id: &str,
            _contents: &[u8],
        ) -> Result<(), SendError> {
            unimplemented!()
        }

        fn close(&mut self) {
            unimplemented!()
        }
    }

    fn create_test_server(response_type: ResponseType) -> TestServer {
        TestServer::build_with_state(move || {
            let mock_connection_addr =
                SawtoothMessageSender::create(move |_ctx: &mut Context<SawtoothMessageSender>| {
                    SawtoothMessageSender::new(MockMessageSender::new_boxed(response_type))
                });
            AppState {
                sawtooth_connection: mock_connection_addr,
            }
        })
        .start(|app| {
            app.resource("/batch_statuses", |r| {
                r.name("batch_statuses");
                r.method(Method::GET).with_async(get_batch_statuses)
            })
            .resource("/batches", |r| {
                r.method(Method::POST).with_async(submit_batches)
            });
        })
    }

    ///
    /// Verifies a GET /batch_statuses with one id works properly.
    ///
    ///    The TestServer will receive a request with :
    ///        - a batch_ids property of BATCH_ID
    ///    It will receive a Protobuf response with status OK:
    ///        - containing batch statuses of {batch_id: BATCH_ID_1,  status: COMMITED}
    ///    It should send back a JSON response with:
    ///        - a link property that ends in '/batch_statuses?id=BATCH_ID_1'
    ///        - a data property matching the batch statuses received
    ///
    #[test]
    fn test_get_batch_status_one_id() {
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);

        let request = srv
            .client(
                http::Method::GET,
                &format!("/batch_statuses?id={}", BATCH_ID_1),
            )
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(response.status(), http::StatusCode::OK);

        let deserialized: BatchStatusResponse =
            serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();

        assert_eq!(deserialized.data.len(), 1);
        assert_eq!(deserialized.data[0].id, BATCH_ID_1);
        assert_eq!(deserialized.data[0].status, "COMMITTED");
        assert_eq!(deserialized.data[0].invalid_transactions.len(), 0);
        assert!(deserialized
            .link
            .contains(&format!("/batch_statuses?id={}", BATCH_ID_1)));
    }

    ///
    /// Verifies a GET /batch_statuses with multiple ids works properly.
    ///
    ///    The TestServer will receive a request with :
    ///        - a batch_ids property of BATCH_ID_1, BATCH_ID_2, BATCH_ID_3
    ///    It will receive a Protobuf response with status OK:
    ///        - containing batch statuses of {batch_id: BATCH_ID_1,  status: COMMITED}
    ///                                       {batch_id: BATCH_ID_2,  status: COMMITED}
    ///                                       {batch_id: BATCH_ID_3,  status: COMMITED}
    ///    It should send back a JSON response with:
    ///        - a link property that ends in
    ///             `/batch_statuses?id={BATCH_ID_1},{BATCH_ID_2},{BATCH_ID_3}`
    ///        - a data property matching the batch statuses received
    ///
    #[test]
    fn test_get_batch_status_multiple_ids() {
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);

        let request = srv
            .client(
                http::Method::GET,
                &format!(
                    "/batch_statuses?id={},{},{}",
                    BATCH_ID_1, BATCH_ID_2, BATCH_ID_3
                ),
            )
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(response.status(), http::StatusCode::OK);

        let deserialized: BatchStatusResponse =
            serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();

        assert_eq!(deserialized.data.len(), 3);
        assert_eq!(deserialized.data[0].id, BATCH_ID_1);
        assert_eq!(deserialized.data[0].status, "COMMITTED");
        assert_eq!(deserialized.data[0].invalid_transactions.len(), 0);

        assert_eq!(deserialized.data[1].id, BATCH_ID_2);
        assert_eq!(deserialized.data[1].status, "COMMITTED");
        assert_eq!(deserialized.data[1].invalid_transactions.len(), 0);

        assert_eq!(deserialized.data[2].id, BATCH_ID_3);
        assert_eq!(deserialized.data[2].status, "COMMITTED");
        assert_eq!(deserialized.data[2].invalid_transactions.len(), 0);
        assert!(deserialized.link.contains(&format!(
            "/batch_statuses?id={},{},{}",
            BATCH_ID_1, BATCH_ID_2, BATCH_ID_3
        )));
    }

    ///
    /// Verifies a GET /batch_statuses with one invalid id works properly.
    ///
    ///    The TestServer will receive a request with :
    ///        - a batch_ids property of BATCH_ID
    ///    It will receive a Protobuf response with status INVALID_ID:
    ///    It should send back a response with status BadRequest:
    ///        - with an error message explaining the error
    ///
    #[test]
    fn test_get_batch_status_invalid_id() {
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseInvalidId);

        let request = srv
            .client(
                http::Method::GET,
                &format!("/batch_statuses?id={}", BATCH_ID_1),
            )
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();
        assert_eq!(response.status(), http::StatusCode::BAD_REQUEST);
    }

    ///
    /// Verifies a GET /batch_statuses returns InternalError when validator responds with error.
    ///
    ///    The TestServer will receive a request with :
    ///        - a batch_ids property of BATCH_ID
    ///    It will receive a Protobuf response with status InternalError
    ///    It should send back a response with status InternalError
    ///
    #[test]
    fn test_get_batch_status_internal_error() {
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseInternalError);

        let request = srv
            .client(
                http::Method::GET,
                &format!("/batch_statuses?id={}", BATCH_ID_1),
            )
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();
        assert_eq!(response.status(), http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    ///
    /// Verifies a GET /batch_statuses returns an error when the wait param is set with an invalid
    /// value.
    ///
    ///    The TestServer will receive a request with :
    ///        - a batch_ids property of BATCH_ID
    ///        - wait param set to "not_a_number"
    ///    It should send back a response with status BadRequest
    ///
    #[test]
    fn test_get_batch_status_wait_error() {
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);

        let request = srv
            .client(
                http::Method::GET,
                &format!("/batch_statuses?id={}&wait=not_a_number", BATCH_ID_1),
            )
            .finish()
            .unwrap();

        let response = srv.execute(request.send()).unwrap();
        assert_eq!(response.status(), http::StatusCode::BAD_REQUEST);
    }

    ///
    /// Verifies a POST /batches with an OK response.
    ///
    ///    The TestServer will receive a request with :
    ///        - an serialized batch list
    ///    It will receive a Protobuf response with status OK
    ///    It should send back a JSON response with:
    ///        - a link property that ends in '/batch_statuses?id=BATCH_ID_1'
    ///
    #[test]
    fn test_post_batches_ok() {
        let mut srv = create_test_server(ResponseType::ClientBatchSubmitResponseOK);

        let request = srv
            .client(http::Method::POST, "/batches")
            .body(get_batch_list())
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(response.status(), http::StatusCode::OK);

        let deserialized: BatchStatusLink =
            serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();

        assert!(deserialized
            .link
            .contains(&format!("/batch_statuses?id={}", BATCH_ID_1)));
    }

    ///
    /// Verifies a POST /batches with an INVALID_BATCH response.
    ///
    ///    The TestServer will receive a request with :
    ///        - an serialized batch list
    ///    It will receive a Protobuf response with status INVALID_BATCH
    ///    It should send back a response with BadRequest status
    ///
    #[test]
    fn test_post_batches_invalid_batch() {
        let mut srv = create_test_server(ResponseType::ClientBatchSubmitResponseInvalidBatch);

        let request = srv
            .client(http::Method::POST, "/batches")
            .body(get_batch_list())
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(response.status(), http::StatusCode::BAD_REQUEST);
    }

    ///
    /// Verifies a POST /batches responds with InternalError when the validator returns an error.
    ///
    ///    The TestServer will receive a request with :
    ///        - an serialized batch list
    ///    It will receive a Protobuf response with status INTERNAL_ERROR
    ///    It should send back a response with InternalError status
    ///
    #[test]
    fn test_post_batches_internal_error() {
        let mut srv = create_test_server(ResponseType::ClientBatchSubmitResponseInternalError);

        let request = srv
            .client(http::Method::POST, "/batches")
            .body(get_batch_list())
            .unwrap();

        let response = srv.execute(request.send()).unwrap();

        assert_eq!(response.status(), http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    fn get_batch_statuses_response_one_id() -> Vec<u8> {
        let mut batch_status_response = ClientBatchStatusResponse::new();
        batch_status_response.set_status(ClientBatchStatusResponse_Status::OK);
        let mut batch_status = ClientBatchStatus::new();
        batch_status.set_batch_id(BATCH_ID_1.to_string());
        batch_status.set_status(ClientBatchStatus_Status::COMMITTED);
        batch_status_response
            .set_batch_statuses(protobuf::RepeatedField::from_vec(vec![batch_status]));
        protobuf::Message::write_to_bytes(&batch_status_response)
            .expect("Failed to write batch statuses to bytes")
    }

    fn get_batch_statuses_response_multiple_ids() -> Vec<u8> {
        let mut batch_status_response = ClientBatchStatusResponse::new();
        batch_status_response.set_status(ClientBatchStatusResponse_Status::OK);
        let mut batch_status_1 = ClientBatchStatus::new();
        batch_status_1.set_batch_id(BATCH_ID_1.to_string());
        batch_status_1.set_status(ClientBatchStatus_Status::COMMITTED);
        let mut batch_status_2 = ClientBatchStatus::new();
        batch_status_2.set_batch_id(BATCH_ID_2.to_string());
        batch_status_2.set_status(ClientBatchStatus_Status::COMMITTED);
        let mut batch_status_3 = ClientBatchStatus::new();
        batch_status_3.set_batch_id(BATCH_ID_3.to_string());
        batch_status_3.set_status(ClientBatchStatus_Status::COMMITTED);
        batch_status_response.set_batch_statuses(protobuf::RepeatedField::from_vec(vec![
            batch_status_1,
            batch_status_2,
            batch_status_3,
        ]));
        protobuf::Message::write_to_bytes(&batch_status_response)
            .expect("Failed to write batch statuses to bytes")
    }

    fn get_batch_statuses_response_invalid_id() -> Vec<u8> {
        let mut batch_status_response = ClientBatchStatusResponse::new();
        batch_status_response.set_status(ClientBatchStatusResponse_Status::INVALID_ID);
        protobuf::Message::write_to_bytes(&batch_status_response)
            .expect("Failed to write batch statuses to bytes")
    }

    fn get_batch_statuses_response_validator_internal_error() -> Vec<u8> {
        let mut batch_status_response = ClientBatchStatusResponse::new();
        batch_status_response.set_status(ClientBatchStatusResponse_Status::INTERNAL_ERROR);
        protobuf::Message::write_to_bytes(&batch_status_response)
            .expect("Failed to write batch statuses to bytes")
    }

    fn get_submit_batches_response(status: ClientBatchSubmitResponse_Status) -> Vec<u8> {
        let mut batch_status_response = ClientBatchSubmitResponse::new();
        batch_status_response.set_status(status);
        protobuf::Message::write_to_bytes(&batch_status_response)
            .expect("Failed to write batch statuses to bytes")
    }

    fn get_batch_list() -> Vec<u8> {
        let mut batch_list = BatchList::new();
        let mut batch = Batch::new();
        batch.set_header_signature(BATCH_ID_1.to_string());
        batch_list.set_batches(protobuf::RepeatedField::from_vec(vec![batch]));
        protobuf::Message::write_to_bytes(&batch_list)
            .expect("Failed to write batch statuses to bytes")
    }

}
