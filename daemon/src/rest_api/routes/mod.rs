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

mod agents;
mod batches;
mod organizations;
mod records;
mod schemas;

pub use agents::*;
pub use batches::*;
pub use organizations::*;
pub use records::*;
pub use schemas::*;

use crate::database::ConnectionPool;

use actix::{Actor, Context, SyncContext};
use sawtooth_sdk::messaging::stream::MessageSender;

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

#[cfg(all(feature = "test-api", test))]
mod test {
    use super::*;
    use crate::database;
    use crate::database::{
        helpers::MAX_BLOCK_NUM,
        models::{
            NewAgent, NewAssociatedAgent, NewGridPropertyDefinition, NewGridSchema,
            NewOrganization, NewProposal, NewRecord,
        },
        schema::{associated_agent, grid_property_definition, grid_schema, proposal, record},
    };
    use crate::rest_api::{
        routes::{AgentSlice, BatchStatusResponse, OrganizationSlice},
        AppState,
    };

    use actix::SyncArbiter;
    use actix_web::{http, http::Method, test::TestServer, HttpMessage};
    use diesel::dsl::insert_into;
    use diesel::pg::PgConnection;
    use diesel::RunQueryDsl;
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

    static DATABASE_URL: &str = "postgres://grid_test:grid_test@test_server:5432/grid_test";

    static KEY1: &str = "111111111111111111111111111111111111111111111111111111111111111111";
    static KEY2: &str = "222222222222222222222222222222222222222222222222222222222222222222";
    static KEY3: &str = "333333333333333333333333333333333333333333333333333333333333333333";

    static ORG_NAME_1: &str = "my_org";
    static ORG_NAME_2: &str = "other_org";

    static ADDRESS_1: &str = "my_address";
    static ADDRESS_2: &str = "my_address_2";
    static UPDATED_ADDRESS_2: &str = "my_updated_address";

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

    fn get_connection_pool() -> ConnectionPool {
        database::create_connection_pool(&DATABASE_URL).expect("Unable to unwrap connection pool")
    }

    fn create_test_server(response_type: ResponseType) -> TestServer {
        TestServer::build_with_state(move || {
            let mock_connection_addr =
                SawtoothMessageSender::create(move |_ctx: &mut Context<SawtoothMessageSender>| {
                    SawtoothMessageSender::new(MockMessageSender::new_boxed(response_type))
                });
            let db_executor_addr =
                SyncArbiter::start(1, move || DbExecutor::new(get_connection_pool()));
            AppState {
                sawtooth_connection: mock_connection_addr,
                database_connection: db_executor_addr,
            }
        })
        .start(|app| {
            app.resource("/batch_statuses", |r| {
                r.name("batch_statuses");
                r.method(Method::GET).with_async(get_batch_statuses)
            })
            .resource("/batches", |r| {
                r.method(Method::POST).with_async(submit_batches)
            })
            .resource("/agent", |r| r.method(Method::GET).with_async(list_agents))
            .resource("/agent/{public_key}", |r| {
                r.method(Method::GET).with_async(fetch_agent)
            })
            .resource("/organization", |r| {
                r.method(Method::GET).with_async(list_organizations)
            })
            .resource("/organization/{id}", |r| {
                r.method(Method::GET).with_async(fetch_organization)
            })
            .resource("/schema", |r| {
                r.method(Method::GET).with_async(list_grid_schemas)
            })
            .resource("/schema/{name}", |r| {
                r.method(Method::GET).with_async(fetch_grid_schema)
            })
            .resource("/record", |r| {
                r.method(Method::GET).with_async(list_records)
            })
            .resource("/record/{record_id}", |r| {
                r.method(Method::GET).with_async(fetch_record)
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

    ///
    /// Verifies a GET /agent responds with an Ok response
    ///     with an empty Agents table
    ///
    ///     The TestServer will receive a request with no parameters
    ///     It will receive a response with status Ok
    ///     It should send back a response with:
    ///         - body containing a list of Agents
    #[test]
    fn test_list_agents() {
        database::run_migrations(&DATABASE_URL).unwrap();
        let test_pool = get_connection_pool();
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);
        // Clears the agents table in the test database
        clear_agents_table(&test_pool.get().unwrap());
        let request = srv.client(http::Method::GET, "/agent").finish().unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert!(response.status().is_success());
        let body: Vec<AgentSlice> =
            serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();
        assert!(body.is_empty());

        // Adds a single Agent to the test database
        populate_agent_table(&test_pool.get().unwrap(), &get_agent());

        // Making another request to the database
        let request = srv.client(http::Method::GET, "/agent").finish().unwrap();
        let response = srv.execute(request.send()).unwrap();

        assert!(response.status().is_success());
        let body: Vec<AgentSlice> =
            serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();
        assert_eq!(body.len(), 1);
        let agent = body.first().unwrap();
        assert_eq!(agent.public_key, KEY1.to_string());
        assert_eq!(agent.org_id, KEY2.to_string());
    }

    ///
    /// Verifies a GET /organization responds with an Ok response
    ///     with an empty organization table
    ///
    #[test]
    fn test_list_organizations_empty() {
        database::run_migrations(&DATABASE_URL).unwrap();
        let test_pool = get_connection_pool();
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);
        // Clears the organization table in the test database
        clear_organization_table(&test_pool.get().unwrap());
        let request = srv
            .client(http::Method::GET, "/organization")
            .finish()
            .unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert!(response.status().is_success());
        let body: Vec<OrganizationSlice> =
            serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();
        assert!(body.is_empty());
    }

    ///
    /// Verifies a GET /organization responds with an Ok response
    ///     with a list containing one organization
    ///
    #[test]
    fn test_list_organizations() {
        database::run_migrations(&DATABASE_URL).unwrap();
        let test_pool = get_connection_pool();
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);

        // Adds an organization to the test database
        populate_organization_table(&test_pool.get().unwrap(), get_organization());

        // Making another request to the database
        let request = srv
            .client(http::Method::GET, "/organization")
            .finish()
            .unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert!(response.status().is_success());
        let body: Vec<OrganizationSlice> =
            serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();
        assert_eq!(body.len(), 1);
        let org = body.first().unwrap();
        assert_eq!(org.name, ORG_NAME_1.to_string());
        assert_eq!(org.org_id, KEY2.to_string());
        assert_eq!(org.address, ADDRESS_1.to_string());
    }

    ///
    /// Verifies a GET /organization responds with an Ok response
    /// with a list containing one organization, when there's two records for the same
    /// organization_id. The rest-api should return a list with a single organization with the
    /// record that contains the most recent information for that organization
    /// (end_block_num == MAX_BLOCK_NUM)
    ///
    #[test]
    fn test_list_organizations_updated() {
        database::run_migrations(&DATABASE_URL).unwrap();
        let test_pool = get_connection_pool();
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);

        // Adds two instances of organization with the same org_id to the test database
        populate_organization_table(&test_pool.get().unwrap(), get_updated_organization());

        // Making another request to the database
        let request = srv
            .client(http::Method::GET, "/organization")
            .finish()
            .unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert!(response.status().is_success());
        let body: Vec<OrganizationSlice> =
            serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();
        assert_eq!(body.len(), 1);
        let org = body.first().unwrap();
        assert_eq!(org.name, ORG_NAME_2.to_string());
        assert_eq!(org.org_id, KEY3.to_string());
        // Checks is returned the organization with the most recent information
        assert_eq!(org.address, UPDATED_ADDRESS_2.to_string());
    }

    ///
    /// Verifies a GET /organization/{id} responds with NotFound response
    /// when there is no organization with the specified id.
    ///
    #[test]
    fn test_fetch_organization_not_found() {
        database::run_migrations(&DATABASE_URL).unwrap();
        let test_pool = get_connection_pool();
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);
        // Clears the organization table in the test database
        clear_organization_table(&test_pool.get().unwrap());
        let request = srv
            .client(http::Method::GET, "/organization/not_a_valid_id")
            .finish()
            .unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    ///
    /// Verifies a GET /organization/{id} responds with Ok response
    /// when there is an organization with the specified id.
    ///
    #[test]
    fn test_fetch_organization_ok() {
        database::run_migrations(&DATABASE_URL).unwrap();
        let test_pool = get_connection_pool();
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);

        // Adds an organization to the test database
        populate_organization_table(&test_pool.get().unwrap(), get_organization());

        // Making another request to the database
        let request = srv
            .client(http::Method::GET, &format!("/organization/{}", KEY2))
            .finish()
            .unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert!(response.status().is_success());
        let org: OrganizationSlice =
            serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();
        assert_eq!(org.name, ORG_NAME_1.to_string());
        assert_eq!(org.org_id, KEY2.to_string());
        assert_eq!(org.address, ADDRESS_1.to_string());
    }

    ///
    /// Verifies a GET /organization/{id} responds with an Ok response
    /// with a single organization, when there's two records for the same
    /// organization_id. The rest-api should return a single organization with the
    /// record that contains the most recent information for that organization
    /// (end_block_num == MAX_BLOCK_NUM)
    ///
    #[test]
    fn test_fetch_organization_updated_ok() {
        database::run_migrations(&DATABASE_URL).unwrap();
        let test_pool = get_connection_pool();
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);

        // Adds an organization to the test database
        populate_organization_table(&test_pool.get().unwrap(), get_updated_organization());

        // Making another request to the database
        let request = srv
            .client(http::Method::GET, &format!("/organization/{}", KEY3))
            .finish()
            .unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert!(response.status().is_success());
        let org: OrganizationSlice =
            serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();
        assert_eq!(org.name, ORG_NAME_2.to_string());
        assert_eq!(org.org_id, KEY3.to_string());
        // Checks is returned the organization with the most recent information
        assert_eq!(org.address, UPDATED_ADDRESS_2.to_string());
    }

    ///
    /// Verifies a GET /agent/{public_key} responds with an Ok response
    ///     with an Agent with the specified public key.
    ///
    #[test]
    fn test_fetch_agent_ok() {
        database::run_migrations(&DATABASE_URL).unwrap();
        let test_pool = get_connection_pool();
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);

        //Adds an agent to the test database
        populate_agent_table(&test_pool.get().unwrap(), &get_agent());

        let request = srv
            .client(http::Method::GET, &format!("/agent/{}", KEY1))
            .finish()
            .unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert!(response.status().is_success());
        let agent: AgentSlice = serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();
        assert_eq!(agent.public_key, KEY1.to_string());
        assert_eq!(agent.org_id, KEY2.to_string());
    }

    ///
    /// Verifies a GET /agent/{public_key} responds with a Not Found response
    ///     when the public key is not assigned to any Agent.
    ///
    #[test]
    fn test_fetch_agent_not_found() {
        database::run_migrations(&DATABASE_URL).unwrap();
        let test_pool = get_connection_pool();
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);
        // Clear the agents table in the test database
        clear_agents_table(&test_pool.get().unwrap());
        let request = srv
            .client(http::Method::GET, "/agent/unknown_public_key")
            .finish()
            .unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    /// Verifies a GET /schema responds with an OK response with a
    ///     list_grid_schemas request.
    ///
    ///     The TestServer will receive a request with no parameters,
    ///         then will respond with an Ok status and a list of Grid Schemas.
    #[test]
    fn test_list_schemas() {
        database::run_migrations(&DATABASE_URL).unwrap();
        let test_pool = get_connection_pool();
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);
        // Clears the grid schema table in the test database
        clear_grid_schema_table(&test_pool.get().unwrap());
        let request = srv.client(http::Method::GET, "/schema").finish().unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert!(response.status().is_success());
        let empty_body: Vec<GridSchemaSlice> =
            serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();
        assert!(empty_body.is_empty());

        populate_grid_schema_table(&test_pool.get().unwrap(), &get_grid_schema());
        let request = srv.client(http::Method::GET, "/schema").finish().unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert!(response.status().is_success());
        let body: Vec<GridSchemaSlice> =
            serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();
        assert_eq!(body.len(), 1);

        let test_schema = body.first().unwrap();
        assert_eq!(test_schema.name, "Test Grid Schema".to_string());
        assert_eq!(test_schema.owner, "phillips001".to_string());
        assert_eq!(test_schema.properties.len(), 2);
    }

    ///
    /// Verifies a GET /schema/{name} responds with an OK response
    ///     and the Grid Schema with the specified name
    ///
    #[test]
    fn test_fetch_schema_ok() {
        database::run_migrations(&DATABASE_URL).unwrap();
        let test_pool = get_connection_pool();
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);
        populate_grid_schema_table(&test_pool.get().unwrap(), &get_grid_schema());
        let request = srv
            .client(
                http::Method::GET,
                &format!("/schema/{}", "Test Grid Schema".to_string()),
            )
            .finish()
            .unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert!(response.status().is_success());
        let test_schema: GridSchemaSlice =
            serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();
        assert_eq!(test_schema.name, "Test Grid Schema".to_string());
        assert_eq!(test_schema.owner, "phillips001".to_string());
        assert_eq!(test_schema.properties.len(), 2);
    }

    ///
    /// Verifies a GET /schema/{name} responds with a Not Found error
    ///     when there is no Grid Schema with the specified name
    ///
    #[test]
    fn test_fetch_schema_not_found() {
        database::run_migrations(&DATABASE_URL).unwrap();
        let test_pool = get_connection_pool();
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);
        clear_grid_schema_table(&test_pool.get().unwrap());
        let request = srv
            .client(http::Method::GET, "/schema/not_in_database")
            .finish()
            .unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    ///
    /// Verifies a GET /record responds with an Ok response
    ///     with a list containing one record
    ///
    #[test]
    fn test_list_records() {
        database::run_migrations(&DATABASE_URL).unwrap();
        let test_pool = get_connection_pool();
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);
        populate_agent_table(&test_pool.get().unwrap(), &get_agents_with_roles());
        populate_associated_agent_table(&test_pool.get().unwrap(), &get_associated_agents());
        populate_proposal_table(&test_pool.get().unwrap(), &get_proposal());
        populate_record_table(&test_pool.get().unwrap(), &get_record());
        let request = srv
            .client(http::Method::GET, &format!("/record"))
            .finish()
            .unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert!(response.status().is_success());
        let body: Vec<RecordSlice> =
            serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();
        assert_eq!(body.len(), 1);
        let test_record = body.first().unwrap();
        assert_eq!(test_record.record_id, "Test Record".to_string());
        assert_eq!(test_record.schema, "Test Grid Schema".to_string());
        assert_eq!(test_record.owner, KEY1.to_string());
        assert_eq!(test_record.custodian, KEY2.to_string());
        assert_eq!(test_record.r#final, false);

        assert_eq!(test_record.owner_updates[0].agent_id, KEY1.to_string());
        assert_eq!(test_record.owner_updates[0].timestamp, 1);

        assert_eq!(test_record.custodian_updates[0].agent_id, KEY2.to_string());
        assert_eq!(test_record.custodian_updates[0].timestamp, 1);

        assert_eq!(test_record.proposals[0].timestamp, 1);
        assert_eq!(test_record.proposals[0].role, "OWNER");
        assert_eq!(
            test_record.proposals[0].properties,
            vec!["location".to_string()]
        );
        assert_eq!(test_record.proposals[0].status, "OPEN");
        assert_eq!(test_record.proposals[0].terms, "Proposal Terms".to_string());
    }

    ///
    /// Verifies a GET /record responds with an Ok response
    /// with a list containing one record, when there's two records for the same
    /// record_id. The rest-api should return a list with a single record with the
    /// record that contains the most recent information for that record
    /// (end_block_num == MAX_BLOCK_NUM)
    ///
    #[test]
    fn test_list_records_updated() {
        database::run_migrations(&DATABASE_URL).unwrap();
        let test_pool = get_connection_pool();
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);

        // Adds two instances of record with the same org_id to the test database
        populate_record_table(&test_pool.get().unwrap(), &get_updated_record());

        // Making another request to the database
        let request = srv.client(http::Method::GET, "/record").finish().unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert!(response.status().is_success());
        let body: Vec<RecordSlice> =
            serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();
        assert_eq!(body.len(), 1);
        let test_record = body.first().unwrap();
        assert_eq!(test_record.record_id, "Test Record".to_string());
        assert_eq!(test_record.schema, "Test Grid Schema".to_string());
        assert_eq!(test_record.r#final, true);
    }

    ///
    /// Verifies a GET /record responds with an Ok response
    /// with a list containing two records, when there's two records with differing
    /// record_ids, one of which has been updated.
    ///
    #[test]
    fn test_list_records_multiple() {
        database::run_migrations(&DATABASE_URL).unwrap();
        let test_pool = get_connection_pool();
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);

        // Adds two instances of record with the same org_id to the test database
        populate_record_table(&test_pool.get().unwrap(), &get_multuple_records());

        // Making another request to the database
        let request = srv.client(http::Method::GET, "/record").finish().unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert!(response.status().is_success());
        let body: Vec<RecordSlice> =
            serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();
        print!("{:?}", body);
        assert_eq!(body.len(), 2);
    }

    ///
    /// Verifies a GET /record/{record_id} responds with an OK response
    ///     and the Record with the specified record ID.
    ///
    #[test]
    fn test_fetch_record_ok() {
        database::run_migrations(&DATABASE_URL).unwrap();
        let test_pool = get_connection_pool();
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);
        populate_agent_table(&test_pool.get().unwrap(), &get_agents_with_roles());
        populate_associated_agent_table(&test_pool.get().unwrap(), &get_associated_agents());
        populate_proposal_table(&test_pool.get().unwrap(), &get_proposal());
        populate_record_table(&test_pool.get().unwrap(), &get_record());
        let request = srv
            .client(
                http::Method::GET,
                &format!("/record/{}", "Test Record".to_string()),
            )
            .finish()
            .unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert!(response.status().is_success());
        let test_record: RecordSlice =
            serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();
        assert_eq!(test_record.record_id, "Test Record".to_string());
        assert_eq!(test_record.schema, "Test Grid Schema".to_string());
        assert_eq!(test_record.owner, KEY1.to_string());
        assert_eq!(test_record.custodian, KEY2.to_string());
        assert_eq!(test_record.r#final, false);

        assert_eq!(test_record.owner_updates[0].agent_id, KEY1.to_string());
        assert_eq!(test_record.owner_updates[0].timestamp, 1);

        assert_eq!(test_record.custodian_updates[0].agent_id, KEY2.to_string());
        assert_eq!(test_record.custodian_updates[0].timestamp, 1);

        assert_eq!(test_record.proposals[0].timestamp, 1);
        assert_eq!(test_record.proposals[0].role, "OWNER");
        assert_eq!(
            test_record.proposals[0].properties,
            vec!["location".to_string()]
        );
        assert_eq!(test_record.proposals[0].status, "OPEN");
        assert_eq!(test_record.proposals[0].terms, "Proposal Terms".to_string());
    }

    ///
    /// Verifies a GET /record/{record_id} responds with an OK response
    ///     and the Record with the specified record ID after the Record's
    ///     owners, custodians, and proposals have been updated.
    ///
    #[test]
    fn test_fetch_record_updated_ok() {
        database::run_migrations(&DATABASE_URL).unwrap();
        let test_pool = get_connection_pool();
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);
        populate_record_table(&test_pool.get().unwrap(), &get_updated_record());
        populate_associated_agent_table(
            &test_pool.get().unwrap(),
            &get_associated_agents_updated(),
        );
        populate_proposal_table(&test_pool.get().unwrap(), &get_updated_proposal());
        let request = srv
            .client(
                http::Method::GET,
                &format!("/record/{}", "Test Record".to_string()),
            )
            .finish()
            .unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert!(response.status().is_success());
        let test_record: RecordSlice =
            serde_json::from_slice(&*response.body().wait().unwrap()).unwrap();

        assert_eq!(test_record.record_id, "Test Record".to_string());
        assert_eq!(test_record.schema, "Test Grid Schema".to_string());
        assert_eq!(test_record.owner, KEY2.to_string());
        assert_eq!(test_record.custodian, KEY1.to_string());
        assert_eq!(test_record.r#final, true);

        assert_eq!(test_record.owner_updates[0].agent_id, KEY1.to_string());
        assert_eq!(test_record.owner_updates[0].timestamp, 1);

        assert_eq!(test_record.custodian_updates[0].agent_id, KEY2.to_string());
        assert_eq!(test_record.custodian_updates[0].timestamp, 1);

        assert_eq!(test_record.owner_updates[1].agent_id, KEY2.to_string());
        assert_eq!(test_record.owner_updates[1].timestamp, 2);

        assert_eq!(test_record.custodian_updates[1].agent_id, KEY1.to_string());
        assert_eq!(test_record.custodian_updates[1].timestamp, 2);

        assert_eq!(test_record.proposals[0].status, "CANCELED");
    }

    ///
    /// Verifies a GET /record/{record_id} responds with a Not Found error
    ///     when there is no Record with the specified record_id.
    ///
    #[test]
    fn test_fetch_record_not_found() {
        database::run_migrations(&DATABASE_URL).unwrap();
        let test_pool = get_connection_pool();
        let mut srv = create_test_server(ResponseType::ClientBatchStatusResponseOK);
        clear_record_table(&test_pool.get().unwrap());
        let request = srv
            .client(http::Method::GET, "/record/not_in_database")
            .finish()
            .unwrap();
        let response = srv.execute(request.send()).unwrap();
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
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

    fn get_agent() -> Vec<NewAgent> {
        vec![NewAgent {
            public_key: KEY1.to_string(),
            org_id: KEY2.to_string(),
            active: true,
            roles: vec![],
            metadata: vec![],
            start_block_num: 0,
            end_block_num: MAX_BLOCK_NUM,
        }]
    }

    fn get_agents_with_roles() -> Vec<NewAgent> {
        vec![
            NewAgent {
                public_key: KEY1.to_string(),
                org_id: KEY3.to_string(),
                active: true,
                roles: vec!["OWNER".to_string()],
                metadata: vec![],
                start_block_num: 0,
                end_block_num: MAX_BLOCK_NUM,
            },
            NewAgent {
                public_key: KEY2.to_string(),
                org_id: KEY3.to_string(),
                active: true,
                roles: vec!["CUSTODIAN".to_string()],
                metadata: vec![],
                start_block_num: 0,
                end_block_num: MAX_BLOCK_NUM,
            },
        ]
    }

    fn populate_agent_table(conn: &PgConnection, agents: &[NewAgent]) {
        clear_agents_table(conn);
        database::helpers::insert_agents(conn, agents).unwrap();
    }

    fn clear_agents_table(conn: &PgConnection) {
        use crate::database::schema::agent::dsl::*;
        diesel::delete(agent).execute(conn).unwrap();
    }

    fn get_organization() -> Vec<NewOrganization> {
        vec![NewOrganization {
            org_id: KEY2.to_string(),
            name: ORG_NAME_1.to_string(),
            address: ADDRESS_1.to_string(),
            metadata: vec![],
            start_block_num: 1,
            end_block_num: database::helpers::MAX_BLOCK_NUM,
        }]
    }

    fn get_updated_organization() -> Vec<NewOrganization> {
        vec![
            NewOrganization {
                org_id: KEY3.to_string(),
                name: ORG_NAME_2.to_string(),
                address: ADDRESS_2.to_string(),
                metadata: vec![],
                start_block_num: 2,
                end_block_num: 4,
            },
            NewOrganization {
                org_id: KEY3.to_string(),
                name: ORG_NAME_2.to_string(),
                address: UPDATED_ADDRESS_2.to_string(),
                metadata: vec![],
                start_block_num: 4,
                end_block_num: database::helpers::MAX_BLOCK_NUM,
            },
        ]
    }

    fn populate_organization_table(conn: &PgConnection, organizations: Vec<NewOrganization>) {
        clear_organization_table(conn);
        database::helpers::insert_organizations(conn, &organizations).unwrap();
    }

    fn clear_organization_table(conn: &PgConnection) {
        use crate::database::schema::organization::dsl::*;
        diesel::delete(organization).execute(conn).unwrap();
    }

    fn get_grid_schema() -> Vec<NewGridSchema> {
        vec![NewGridSchema {
            start_block_num: 0,
            end_block_num: MAX_BLOCK_NUM,
            name: "Test Grid Schema".to_string(),
            description: "Example test grid schema".to_string(),
            owner: "phillips001".to_string(),
        }]
    }

    fn get_associated_agents() -> Vec<NewAssociatedAgent> {
        vec![
            NewAssociatedAgent {
                start_block_num: 0,
                end_block_num: MAX_BLOCK_NUM,
                agent_id: KEY1.to_string(),
                timestamp: 1,
                record_id: "Test Record".to_string(),
                role: "OWNER".to_string(),
            },
            NewAssociatedAgent {
                start_block_num: 0,
                end_block_num: MAX_BLOCK_NUM,
                agent_id: KEY2.to_string(),
                timestamp: 1,
                record_id: "Test Record".to_string(),
                role: "CUSTODIAN".to_string(),
            },
        ]
    }

    fn get_associated_agents_updated() -> Vec<NewAssociatedAgent> {
        vec![
            NewAssociatedAgent {
                start_block_num: 0,
                end_block_num: MAX_BLOCK_NUM,
                agent_id: KEY1.to_string(),
                timestamp: 1,
                record_id: "Test Record".to_string(),
                role: "OWNER".to_string(),
            },
            NewAssociatedAgent {
                start_block_num: 0,
                end_block_num: MAX_BLOCK_NUM,
                agent_id: KEY2.to_string(),
                timestamp: 1,
                record_id: "Test Record".to_string(),
                role: "CUSTODIAN".to_string(),
            },
            NewAssociatedAgent {
                start_block_num: 1,
                end_block_num: MAX_BLOCK_NUM,
                agent_id: KEY2.to_string(),
                timestamp: 2,
                record_id: "Test Record".to_string(),
                role: "OWNER".to_string(),
            },
            NewAssociatedAgent {
                start_block_num: 1,
                end_block_num: MAX_BLOCK_NUM,
                agent_id: KEY1.to_string(),
                timestamp: 2,
                record_id: "Test Record".to_string(),
                role: "CUSTODIAN".to_string(),
            },
        ]
    }

    fn get_proposal() -> Vec<NewProposal> {
        vec![NewProposal {
            start_block_num: 0,
            end_block_num: MAX_BLOCK_NUM,
            record_id: "Test Record".to_string(),
            timestamp: 1,
            issuing_agent: KEY1.to_string(),
            receiving_agent: KEY2.to_string(),
            properties: vec!["location".to_string()],
            role: "OWNER".to_string(),
            status: "OPEN".to_string(),
            terms: "Proposal Terms".to_string(),
        }]
    }

    fn get_updated_proposal() -> Vec<NewProposal> {
        vec![
            NewProposal {
                start_block_num: 0,
                end_block_num: 1,
                record_id: "Test Record".to_string(),
                timestamp: 1,
                issuing_agent: KEY1.to_string(),
                receiving_agent: KEY2.to_string(),
                properties: vec!["location".to_string()],
                role: "OWNER".to_string(),
                status: "OPEN".to_string(),
                terms: "Proposal Terms".to_string(),
            },
            NewProposal {
                start_block_num: 1,
                end_block_num: MAX_BLOCK_NUM,
                record_id: "Test Record".to_string(),
                timestamp: 1,
                issuing_agent: KEY1.to_string(),
                receiving_agent: KEY2.to_string(),
                properties: vec!["location".to_string()],
                role: "OWNER".to_string(),
                status: "CANCELED".to_string(),
                terms: "Proposal Terms".to_string(),
            },
        ]
    }

    fn get_record() -> Vec<NewRecord> {
        vec![NewRecord {
            start_block_num: 0,
            end_block_num: MAX_BLOCK_NUM,
            record_id: "Test Record".to_string(),
            schema: "Test Grid Schema".to_string(),
            final_: false,
            owners: vec![KEY1.to_string()],
            custodians: vec![KEY2.to_string()],
        }]
    }

    fn get_updated_record() -> Vec<NewRecord> {
        vec![
            NewRecord {
                start_block_num: 0,
                end_block_num: 1,
                record_id: "Test Record".to_string(),
                schema: "Test Grid Schema".to_string(),
                final_: false,
                owners: vec![KEY1.to_string()],
                custodians: vec![KEY2.to_string()],
            },
            NewRecord {
                start_block_num: 1,
                end_block_num: MAX_BLOCK_NUM,
                record_id: "Test Record".to_string(),
                schema: "Test Grid Schema".to_string(),
                final_: true,
                owners: vec![KEY2.to_string(), KEY1.to_string()],
                custodians: vec![KEY1.to_string(), KEY2.to_string()],
            },
        ]
    }

    fn get_multuple_records() -> Vec<NewRecord> {
        vec![
            NewRecord {
                start_block_num: 0,
                end_block_num: 1,
                record_id: "Test Record".to_string(),
                schema: "Test Grid Schema".to_string(),
                final_: false,
                owners: vec![KEY1.to_string()],
                custodians: vec![KEY2.to_string()],
            },
            NewRecord {
                start_block_num: 1,
                end_block_num: MAX_BLOCK_NUM,
                record_id: "Test Record".to_string(),
                schema: "Test Grid Schema".to_string(),
                final_: true,
                owners: vec![KEY2.to_string(), KEY1.to_string()],
                custodians: vec![KEY1.to_string(), KEY2.to_string()],
            },
            NewRecord {
                start_block_num: 0,
                end_block_num: MAX_BLOCK_NUM,
                record_id: "Test Record 2".to_string(),
                schema: "Test Grid Schema".to_string(),
                final_: false,
                owners: vec![KEY1.to_string()],
                custodians: vec![KEY2.to_string()],
            },
        ]
    }

    fn populate_grid_schema_table(conn: &PgConnection, schemas: &[NewGridSchema]) {
        clear_grid_schema_table(conn);
        populate_property_definition_table(conn, &get_property_definition());
        insert_into(grid_schema::table)
            .values(schemas)
            .execute(conn)
            .map(|_| ())
            .unwrap();
    }

    fn clear_grid_schema_table(conn: &PgConnection) {
        use crate::database::schema::grid_schema::dsl::*;
        diesel::delete(grid_schema).execute(conn).unwrap();
    }

    fn get_property_definition() -> Vec<NewGridPropertyDefinition> {
        vec![
            NewGridPropertyDefinition {
                start_block_num: 0,
                end_block_num: MAX_BLOCK_NUM,
                name: "Definition Name".to_string(),
                schema_name: "Test Grid Schema".to_string(),
                data_type: "Lightbulb".to_string(),
                required: false,
                description: "Definition Description".to_string(),
                number_exponent: 0,
                enum_options: vec![],
                struct_properties: vec![],
            },
            NewGridPropertyDefinition {
                start_block_num: 0,
                end_block_num: MAX_BLOCK_NUM,
                name: "Other Definition Name".to_string(),
                schema_name: "Test Grid Schema".to_string(),
                data_type: "New Lightbulb".to_string(),
                required: false,
                description: "Definition Description".to_string(),
                number_exponent: 0,
                enum_options: vec![],
                struct_properties: vec![],
            },
        ]
    }

    fn populate_property_definition_table(
        conn: &PgConnection,
        definitions: &[NewGridPropertyDefinition],
    ) {
        clear_property_definition_table(conn);
        insert_into(grid_property_definition::table)
            .values(definitions)
            .execute(conn)
            .map(|_| ())
            .unwrap();
    }

    fn clear_property_definition_table(conn: &PgConnection) {
        use crate::database::schema::grid_property_definition::dsl::*;
        diesel::delete(grid_property_definition)
            .execute(conn)
            .unwrap();
    }

    fn populate_associated_agent_table(
        conn: &PgConnection,
        associated_agents: &[NewAssociatedAgent],
    ) {
        clear_associated_agent_table(conn);
        insert_into(associated_agent::table)
            .values(associated_agents)
            .execute(conn)
            .map(|_| ())
            .unwrap();
    }

    fn clear_associated_agent_table(conn: &PgConnection) {
        use crate::database::schema::associated_agent::dsl::*;
        diesel::delete(associated_agent).execute(conn).unwrap();
    }

    fn populate_proposal_table(conn: &PgConnection, proposals: &[NewProposal]) {
        clear_proposal_table(conn);
        insert_into(proposal::table)
            .values(proposals)
            .execute(conn)
            .map(|_| ())
            .unwrap();
    }

    fn clear_proposal_table(conn: &PgConnection) {
        use crate::database::schema::proposal::dsl::*;
        diesel::delete(proposal).execute(conn).unwrap();
    }

    fn populate_record_table(conn: &PgConnection, records: &[NewRecord]) {
        clear_record_table(conn);
        insert_into(record::table)
            .values(records)
            .execute(conn)
            .map(|_| ())
            .unwrap();
    }

    fn clear_record_table(conn: &PgConnection) {
        use crate::database::schema::record::dsl::*;
        diesel::delete(record).execute(conn).unwrap();
    }
}
