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

use std::sync::Arc;

use grid_sdk::{
    agents::{AgentStore, DieselAgentStore},
    locations::{DieselLocationStore, LocationStore},
    organizations::{DieselOrganizationStore, OrganizationStore},
    products::{DieselProductStore, ProductStore},
    schemas::{DieselSchemaStore, SchemaStore},
    track_and_trace::{DieselTrackAndTraceStore, TrackAndTraceStore},
};

#[cfg(feature = "pike")]
mod agents;
mod batches;
#[cfg(feature = "location")]
mod locations;
#[cfg(feature = "pike")]
mod organizations;
#[cfg(feature = "product")]
mod products;
#[cfg(feature = "track-and-trace")]
mod records;
#[cfg(feature = "schema")]
mod schemas;

#[cfg(feature = "pike")]
pub use agents::*;
pub use batches::*;
#[cfg(feature = "location")]
pub use locations::*;
#[cfg(feature = "pike")]
pub use organizations::*;
#[cfg(feature = "product")]
pub use products::*;
#[cfg(feature = "track-and-trace")]
pub use records::*;
#[cfg(feature = "schema")]
pub use schemas::*;

use crate::database::ConnectionPool;

use actix::{Actor, SyncContext};

#[derive(Clone)]
pub struct DbExecutor {
    agent_store: Arc<dyn AgentStore>,
    location_store: Arc<dyn LocationStore>,
    organization_store: Arc<dyn OrganizationStore>,
    product_store: Arc<dyn ProductStore>,
    schema_store: Arc<dyn SchemaStore>,
    tnt_store: Arc<dyn TrackAndTraceStore>,
}

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

impl DbExecutor {
    pub fn from_pg_pool(connection_pool: ConnectionPool<diesel::pg::PgConnection>) -> DbExecutor {
        let agent_store = Arc::new(DieselAgentStore::new(connection_pool.pool.clone()));
        let location_store = Arc::new(DieselLocationStore::new(connection_pool.pool.clone()));
        let organization_store =
            Arc::new(DieselOrganizationStore::new(connection_pool.pool.clone()));
        let product_store = Arc::new(DieselProductStore::new(connection_pool.pool.clone()));
        let schema_store = Arc::new(DieselSchemaStore::new(connection_pool.pool.clone()));
        let tnt_store = Arc::new(DieselTrackAndTraceStore::new(connection_pool.pool));

        Self {
            agent_store,
            location_store,
            organization_store,
            product_store,
            schema_store,
            tnt_store,
        }
    }

    pub fn from_sqlite_pool(
        connection_pool: ConnectionPool<diesel::sqlite::SqliteConnection>,
    ) -> DbExecutor {
        let agent_store = Arc::new(DieselAgentStore::new(connection_pool.pool.clone()));
        let location_store = Arc::new(DieselLocationStore::new(connection_pool.pool.clone()));
        let organization_store =
            Arc::new(DieselOrganizationStore::new(connection_pool.pool.clone()));
        let product_store = Arc::new(DieselProductStore::new(connection_pool.pool.clone()));
        let schema_store = Arc::new(DieselSchemaStore::new(connection_pool.pool.clone()));
        let tnt_store = Arc::new(DieselTrackAndTraceStore::new(connection_pool.pool));

        Self {
            agent_store,
            location_store,
            organization_store,
            product_store,
            schema_store,
            tnt_store,
        }
    }
}

#[cfg(all(test, feature = "stable"))]
mod test {
    use super::*;
    use crate::config::Endpoint;
    use crate::database;
    use crate::rest_api::{
        error::RestApiResponseError,
        routes::{AgentSlice, OrganizationSlice},
        AppState,
    };
    use crate::sawtooth::batch_submitter::{
        process_batch_status_response, process_validator_response, query_validator,
    };
    use crate::submitter::*;

    use actix_web::{
        http,
        test::{start, TestServer},
        web, App,
    };
    use diesel::Connection;
    #[cfg(feature = "test-postgres")]
    use diesel::PgConnection;
    #[cfg(not(feature = "test-postgres"))]
    use diesel::SqliteConnection;
    use futures::prelude::*;
    #[cfg(feature = "test-postgres")]
    use grid_sdk::migrations::{clear_postgres_database, run_postgres_migrations};
    #[cfg(not(feature = "test-postgres"))]
    use grid_sdk::migrations::{clear_sqlite_database, run_sqlite_migrations};
    #[cfg(feature = "track-and-trace")]
    use grid_sdk::track_and_trace::store::{
        diesel::DieselTrackAndTraceStore, AssociatedAgent, LatLongValue, Property, Proposal,
        Record, ReportedValue, Reporter,
    };
    use grid_sdk::{
        agents::store::{diesel::DieselAgentStore, Agent},
        locations::store::{diesel::DieselLocationStore, Location, LocationAttribute},
        organizations::store::{diesel::DieselOrganizationStore, Organization},
        products::store::{diesel::DieselProductStore, Product, PropertyValue},
        schemas::store::{diesel::DieselSchemaStore, PropertyDefinition, Schema},
    };
    use sawtooth_sdk::messages::batch::{Batch, BatchList};
    use sawtooth_sdk::messages::client_batch_submit::{
        ClientBatchStatus, ClientBatchStatusRequest, ClientBatchStatusResponse,
        ClientBatchStatusResponse_Status, ClientBatchStatus_Status, ClientBatchSubmitRequest,
        ClientBatchSubmitResponse, ClientBatchSubmitResponse_Status,
    };
    use sawtooth_sdk::messages::validator::{Message, Message_MessageType};

    use sawtooth_sdk::messaging::stream::{MessageFuture, MessageSender, SendError};
    use std::pin::Pin;
    use std::sync::mpsc::channel;

    #[cfg(feature = "test-postgres")]
    static DATABASE_URL: &str = "postgres://grid_test:grid_test@test_server:5432/grid_test";
    #[cfg(not(feature = "test-postgres"))]
    static DATABASE_URL: &str = "test_db";

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

    static TEST_SERVICE_ID: &str = "test_service";

    #[derive(Clone)]
    enum Backend {
        Splinter,
        Sawtooth,
    }

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
        fn new(response_type: ResponseType) -> Self {
            MockMessageSender { response_type }
        }
    }

    macro_rules! try_fut {
        ($try_expr:expr) => {
            match $try_expr {
                Ok(res) => res,
                Err(err) => return futures::future::err(err).boxed(),
            }
        };
    }

    struct MockBatchSubmitter {
        sender: MockMessageSender,
    }

    impl BatchSubmitter for MockBatchSubmitter {
        fn submit_batches(
            &self,
            msg: SubmitBatches,
        ) -> Pin<Box<dyn Future<Output = Result<BatchStatusLink, RestApiResponseError>> + Send>>
        {
            let mut client_submit_request = ClientBatchSubmitRequest::new();
            client_submit_request.set_batches(protobuf::RepeatedField::from_vec(
                msg.batch_list.get_batches().to_vec(),
            ));

            let response_status: ClientBatchSubmitResponse = try_fut!(query_validator(
                &self.sender,
                Message_MessageType::CLIENT_BATCH_SUBMIT_REQUEST,
                &client_submit_request,
            ));

            future::ready(
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
                },
            )
            .boxed()
        }

        fn batch_status(
            &self,
            msg: BatchStatuses,
        ) -> Pin<Box<dyn Future<Output = Result<Vec<BatchStatus>, RestApiResponseError>> + Send>>
        {
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

            let response_status: ClientBatchStatusResponse = try_fut!(query_validator(
                &self.sender,
                Message_MessageType::CLIENT_BATCH_STATUS_REQUEST,
                &batch_status_request,
            ));

            future::ready(process_batch_status_response(response_status)).boxed()
        }

        fn clone_box(&self) -> Box<dyn BatchSubmitter> {
            unimplemented!()
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

    #[cfg(feature = "test-postgres")]
    fn get_connection_pool() -> ConnectionPool<diesel::pg::PgConnection> {
        database::ConnectionPool::new(&DATABASE_URL).expect("Unable to unwrap connection pool")
    }

    #[cfg(not(feature = "test-postgres"))]
    fn get_connection_pool() -> ConnectionPool<diesel::sqlite::SqliteConnection> {
        database::ConnectionPool::new(&DATABASE_URL).expect("Unable to unwrap connection pool")
    }

    fn create_test_server(backend: Backend, response_type: ResponseType) -> TestServer {
        start(move || {
            let state = {
                let mock_sender = MockMessageSender::new(response_type);
                let mock_batch_submitter = Box::new(MockBatchSubmitter {
                    sender: mock_sender,
                });
                #[cfg(feature = "test-postgres")]
                let db_executor = DbExecutor::from_pg_pool(get_connection_pool());
                #[cfg(not(feature = "test-postgres"))]
                let db_executor = DbExecutor::from_sqlite_pool(get_connection_pool());
                AppState::new(mock_batch_submitter, db_executor)
            };
            let endpoint_backend = match backend {
                Backend::Splinter => "splinter:",
                Backend::Sawtooth => "sawtooth:",
            };

            #[allow(unused_mut)]
            let mut app = App::new()
                .data(state)
                .app_data(Endpoint::from(
                    format!("{}tcp://localhost:9090", endpoint_backend).as_str(),
                ))
                .service(web::resource("/batches").route(web::post().to(submit_batches)))
                .service(
                    web::resource("/batch_statuses")
                        .name("batch_statuses")
                        .route(web::get().to(get_batch_statuses)),
                )
                .service(
                    web::scope("/agent")
                        .service(web::resource("").route(web::get().to(list_agents)))
                        .service(web::resource("/{public_key}").route(web::get().to(fetch_agent))),
                )
                .service(
                    web::scope("/organization")
                        .service(web::resource("").route(web::get().to(list_organizations)))
                        .service(web::resource("/{id}").route(web::get().to(fetch_organization))),
                )
                .service(
                    web::scope("/product")
                        .service(web::resource("").route(web::get().to(list_products)))
                        .service(web::resource("/{id}").route(web::get().to(fetch_product))),
                )
                .service(
                    web::scope("/location")
                        .service(web::resource("").route(web::get().to(list_locations)))
                        .service(web::resource("/{id}").route(web::get().to(fetch_location))),
                )
                .service(
                    web::scope("/schema")
                        .service(web::resource("").route(web::get().to(list_grid_schemas)))
                        .service(web::resource("/{name}").route(web::get().to(fetch_grid_schema))),
                );

            #[cfg(feature = "track-and-trace")]
            {
                app = app.service(
                    web::scope("/record")
                        .service(web::resource("").route(web::get().to(list_records)))
                        .service(
                            web::scope("/{record_id}")
                                .service(web::resource("").route(web::get().to(fetch_record)))
                                .service(
                                    web::resource("/property/{property_name}")
                                        .route(web::get().to(fetch_record_property)),
                                ),
                        ),
                );
            }

            app
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
    #[actix_rt::test]
    async fn test_get_batch_status_one_id() {
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);

        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/batch_statuses?id={}", BATCH_ID_1)),
            )
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), http::StatusCode::OK);

        let deserialized: BatchStatusResponse =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();

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
    #[actix_rt::test]
    async fn test_get_batch_status_multiple_ids() {
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);

        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!(
                    "/batch_statuses?id={},{},{}",
                    BATCH_ID_1, BATCH_ID_2, BATCH_ID_3
                )),
            )
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), http::StatusCode::OK);

        let deserialized: BatchStatusResponse =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();

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
    #[actix_rt::test]
    async fn test_get_batch_status_invalid_id() {
        let srv = create_test_server(
            Backend::Sawtooth,
            ResponseType::ClientBatchStatusResponseInvalidId,
        );

        let response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/batch_statuses?id={}", BATCH_ID_1)),
            )
            .send()
            .await
            .unwrap();

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
    #[actix_rt::test]
    async fn test_get_batch_status_internal_error() {
        let srv = create_test_server(
            Backend::Sawtooth,
            ResponseType::ClientBatchStatusResponseInternalError,
        );

        let response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/batch_statuses?id={}", BATCH_ID_1)),
            )
            .send()
            .await
            .unwrap();

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
    #[actix_rt::test]
    async fn test_get_batch_status_wait_error() {
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);

        let response = srv
            .request(
                http::Method::GET,
                srv.url(&format!(
                    "/batch_statuses?id={}&wait=not_a_number",
                    BATCH_ID_1
                )),
            )
            .send()
            .await
            .unwrap();

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
    #[actix_rt::test]
    async fn test_post_batches_ok() {
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchSubmitResponseOK);

        let mut response = srv
            .request(http::Method::POST, srv.url("/batches"))
            .send_body(get_batch_list())
            .await
            .unwrap();

        assert_eq!(response.status(), http::StatusCode::OK);

        let deserialized: BatchStatusLink =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();

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
    #[actix_rt::test]
    async fn test_post_batches_invalid_batch() {
        let srv = create_test_server(
            Backend::Sawtooth,
            ResponseType::ClientBatchSubmitResponseInvalidBatch,
        );

        let response = srv
            .request(http::Method::POST, srv.url("/batches"))
            .send_body(get_batch_list())
            .await
            .unwrap();

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
    #[actix_rt::test]
    async fn test_post_batches_internal_error() {
        let srv = create_test_server(
            Backend::Sawtooth,
            ResponseType::ClientBatchSubmitResponseInternalError,
        );

        let response = srv
            .request(http::Method::POST, srv.url("/batches"))
            .send_body(get_batch_list())
            .await
            .unwrap();

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
    #[actix_rt::test]
    async fn test_list_agents() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);
        // Clears the agents table in the test database
        clear_database();
        let mut response = srv
            .request(http::Method::GET, srv.url("/agent"))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Vec<AgentSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert!(body.is_empty());

        // Adds a single Agent to the test database
        populate_agent_table(get_agent(None));

        // Making another request to the database
        let mut response = srv
            .request(http::Method::GET, srv.url("/agent"))
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());
        let body: Vec<AgentSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(body.len(), 1);
        let agent = body.first().unwrap();
        assert_eq!(agent.public_key, KEY1.to_string());
        assert_eq!(agent.org_id, KEY2.to_string());
    }

    ///
    /// Verifies a GET /agent?service_id=test_service responds with an Ok response
    ///     with an empty Agents table and a single Agent with the matching service_id.
    ///
    ///     The TestServer will receive a request with service_id equal to 'test'
    ///     It will receive a response with status Ok
    ///     It should send back a response with:
    ///         - body containing a list of Agents
    #[actix_rt::test]
    async fn test_list_agents_with_service_id() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);
        // Clears the agents table in the test database
        clear_database();
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/agent?service_id={}", TEST_SERVICE_ID)),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Vec<AgentSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert!(body.is_empty());

        // Adds a single Agent to the test database
        populate_agent_table(get_agent(Some(TEST_SERVICE_ID.to_string())));

        // Making another request to the database
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/agent?service_id={}", TEST_SERVICE_ID)),
            )
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());
        let body: Vec<AgentSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(body.len(), 1);
        let agent = body.first().unwrap();
        assert_eq!(agent.public_key, KEY1.to_string());
        assert_eq!(agent.org_id, KEY2.to_string());
        assert_eq!(agent.service_id, Some(TEST_SERVICE_ID.to_string()));
    }

    /////
    ///// Verifies a GET /organization responds with an Ok response
    /////     with an empty organization table
    /////
    #[actix_rt::test]
    async fn test_list_organizations_empty() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);
        // Clears the organization table in the test database
        clear_database();
        let mut response = srv
            .request(http::Method::GET, srv.url("/organization"))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Vec<OrganizationSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert!(body.is_empty());
    }

    ///
    /// Verifies a GET /organization responds with an Ok response
    ///     with a list containing one organization
    ///
    #[actix_rt::test]
    async fn test_list_organizations() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);

        // Adds an organization to the test database
        clear_database();
        populate_organization_table(get_organization(None));

        // Making another request to the database
        let mut response = srv
            .request(http::Method::GET, srv.url("/organization"))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Vec<OrganizationSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(body.len(), 1);
        let org = body.first().unwrap();
        assert_eq!(org.name, ORG_NAME_1.to_string());
        assert_eq!(org.org_id, KEY2.to_string());
        assert_eq!(org.address, ADDRESS_1.to_string());
    }

    ///
    /// Verifies a GET /organization?service_id=test_service responds with an Ok response
    ///     with a list containing one organization with a matching service_id
    ///
    #[actix_rt::test]
    async fn test_list_organizations_with_service_id() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);

        // Adds an organization to the test database
        clear_database();
        populate_organization_table(get_organization(Some(TEST_SERVICE_ID.to_string())));

        // Making another request to the database
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/organization?service_id={}", TEST_SERVICE_ID)),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Vec<OrganizationSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(body.len(), 1);
        let org = body.first().unwrap();
        assert_eq!(org.name, ORG_NAME_1.to_string());
        assert_eq!(org.org_id, KEY2.to_string());
        assert_eq!(org.address, ADDRESS_1.to_string());
        assert_eq!(org.service_id, Some(TEST_SERVICE_ID.to_string()));
    }

    ///
    /// Verifies a GET /organization responds with an Ok response
    /// with a list containing one organization, when there's two records for the same
    /// organization_id. The rest-api should return a list with a single organization with the
    /// record that contains the most recent information for that organization
    /// (end_commit_num == MAX_COMMIT_NUM)
    ///
    #[actix_rt::test]
    async fn test_list_organizations_updated() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);

        // Adds two instances of organization with the same org_id to the test database
        clear_database();
        populate_organization_table(get_updated_organization());

        // Making another request to the database
        let mut response = srv
            .request(http::Method::GET, srv.url("/organization"))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Vec<OrganizationSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
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
    #[actix_rt::test]
    async fn test_fetch_organization_not_found() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);
        // Clears the organization table in the test database
        clear_database();
        let response = srv
            .request(http::Method::GET, srv.url("/organization/not_a_valid_id"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    ///
    /// Verifies a GET /organization/{id}?service_id=test_service responds with NotFound response
    /// when there is no organization with the specified id and service_id.
    ///
    #[actix_rt::test]
    async fn test_fetch_organization_with_service_id_not_found() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);
        //Adds an organization to the test database
        clear_database();
        populate_organization_table(get_organization(None));
        let response = srv
            .request(
                http::Method::GET,
                srv.url(&format!(
                    "/organization/not_a_valid_id?service_id={}",
                    TEST_SERVICE_ID
                )),
            )
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    ///
    /// Verifies a GET /organization/{id} responds with Ok response
    /// when there is an organization with the specified id.
    ///
    #[actix_rt::test]
    async fn test_fetch_organization_ok() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);

        // Adds an organization to the test database
        clear_database();
        populate_organization_table(get_organization(None));

        // Making another request to the database
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/organization/{}", KEY2)),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let org: OrganizationSlice =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(org.name, ORG_NAME_1.to_string());
        assert_eq!(org.org_id, KEY2.to_string());
        assert_eq!(org.address, ADDRESS_1.to_string());
    }

    ///
    /// Verifies a GET /organization/{id} responds with an Ok response
    /// with a single organization, when there's two records for the same
    /// organization_id. The rest-api should return a single organization with the
    /// record that contains the most recent information for that organization
    /// (end_commit_num == MAX_COMMIT_NUM)
    ///
    #[actix_rt::test]
    async fn test_fetch_organization_updated_ok() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);

        // Adds an organization to the test database
        clear_database();
        populate_organization_table(get_updated_organization());

        // Making another request to the database
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/organization/{}", KEY3)),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let org: OrganizationSlice =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(org.name, ORG_NAME_2.to_string());
        assert_eq!(org.org_id, KEY3.to_string());
        // Checks is returned the organization with the most recent information
        assert_eq!(org.address, UPDATED_ADDRESS_2.to_string());
    }

    ///
    /// Verifies a GET /organization/{id}?service_id=test_service responds with Ok response
    /// when there is an organization with the specified id and matching service_id.
    ///
    #[actix_rt::test]
    async fn test_fetch_organization_with_service_id_ok() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);

        // Adds an organization to the test database
        clear_database();
        populate_organization_table(get_organization(Some(TEST_SERVICE_ID.to_string())));

        // Making another request to the database
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!(
                    "/organization/{}?service_id={}",
                    KEY2, TEST_SERVICE_ID
                )),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let org: OrganizationSlice =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(org.name, ORG_NAME_1.to_string());
        assert_eq!(org.org_id, KEY2.to_string());
        assert_eq!(org.address, ADDRESS_1.to_string());
        assert_eq!(org.service_id, Some(TEST_SERVICE_ID.to_string()));
    }

    ///
    /// Verifies a GET /agent/{public_key} responds with an Ok response
    ///     with an Agent with the specified public key.
    ///
    #[actix_rt::test]
    async fn test_fetch_agent_ok() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);

        //Adds an agent to the test database
        clear_database();
        populate_agent_table(get_agent(None));

        let mut response = srv
            .request(http::Method::GET, srv.url(&format!("/agent/{}", KEY1)))
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());
        let agent: AgentSlice = serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(agent.public_key, KEY1.to_string());
        assert_eq!(agent.org_id, KEY2.to_string());
    }

    ///
    /// Verifies a GET /agent/{public_key}?service_id=test_service responds with an Ok response
    ///     with an Agent with the specified public key and service_id.
    ///
    #[actix_rt::test]
    async fn test_fetch_agent_with_service_id_ok() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);

        //Adds an agent to the test database
        clear_database();
        populate_agent_table(get_agent(Some(TEST_SERVICE_ID.to_string())));

        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/agent/{}?service_id={}", KEY1, TEST_SERVICE_ID)),
            )
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());
        let agent: AgentSlice = serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(agent.public_key, KEY1.to_string());
        assert_eq!(agent.org_id, KEY2.to_string());
        assert_eq!(agent.service_id, Some(TEST_SERVICE_ID.to_string()));
    }

    ///
    /// Verifies a GET /agent/{public_key} responds with a Not Found response
    ///     when the public key is not assigned to any Agent.
    ///
    #[actix_rt::test]
    async fn test_fetch_agent_not_found() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);
        // Clear the agents table in the test database
        clear_database();
        let response = srv
            .request(http::Method::GET, srv.url("/agent/unknown_public_key"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    ///
    /// Verifies a GET /agent/{public_key}?service_id=test_service responds with a Not Found response
    ///     when the public key is not assigned to any Agent with the service_id.
    ///
    #[actix_rt::test]
    async fn test_fetch_agent_with_service_id_not_found() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);
        //Adds an agent to the test database
        clear_database();
        populate_agent_table(get_agent(None));

        let response = srv
            .request(
                http::Method::GET,
                srv.url(&format!(
                    "/agent/unknown_public_key?service_id={}",
                    TEST_SERVICE_ID
                )),
            )
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    /// Verifies a GET /schema responds with an OK response with a
    ///     list_grid_schemas request.
    ///
    ///     The TestServer will receive a request with no parameters,
    ///         then will respond with an Ok status and a list of Grid Schemas.
    #[actix_rt::test]
    async fn test_list_schemas() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);
        // Clears the grid schema table in the test database
        clear_database();
        let mut response = srv
            .request(http::Method::GET, srv.url("/schema"))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let empty_body: Vec<GridSchemaSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert!(empty_body.is_empty());

        populate_grid_schema_table(get_grid_schema(None));
        let mut response = srv
            .request(http::Method::GET, srv.url("/schema"))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Vec<GridSchemaSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(body.len(), 1);

        let test_schema = body.first().unwrap();
        assert_eq!(test_schema.name, "TestGridSchema".to_string());
        assert_eq!(test_schema.owner, "phillips001".to_string());
        assert_eq!(test_schema.properties.len(), 2);
    }

    /// Verifies a GET /schema?service_id=test_service responds with an OK response with a
    ///     list_grid_schemas request.
    ///
    ///     The TestServer will receive a request with the service ID,
    ///         then will respond with an Ok status and a list of Grid Schemas.
    #[actix_rt::test]
    async fn test_list_schemas_with_service_id() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);
        // Clears the grid schema table in the test database
        clear_database();
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/schema?service_id={}", TEST_SERVICE_ID)),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let empty_body: Vec<GridSchemaSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert!(empty_body.is_empty());

        populate_grid_schema_table(get_grid_schema(Some(TEST_SERVICE_ID.to_string())));
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/schema?service_id={}", TEST_SERVICE_ID)),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Vec<GridSchemaSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(body.len(), 1);

        let test_schema = body.first().unwrap();
        assert_eq!(test_schema.name, "TestGridSchema".to_string());
        assert_eq!(test_schema.owner, "phillips001".to_string());
        assert_eq!(test_schema.properties.len(), 2);
        assert_eq!(test_schema.service_id, Some(TEST_SERVICE_ID.to_string()));
    }

    ///
    /// Verifies a GET /schema/{name} responds with an OK response
    ///     and the Grid Schema with the specified name
    ///
    #[actix_rt::test]
    async fn test_fetch_schema_ok() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);
        clear_database();
        populate_grid_schema_table(get_grid_schema(None));
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/schema/{}", "TestGridSchema".to_string())),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let test_schema: GridSchemaSlice =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(test_schema.name, "TestGridSchema".to_string());
        assert_eq!(test_schema.owner, "phillips001".to_string());
        assert_eq!(test_schema.properties.len(), 2);
    }

    ///
    /// Verifies a GET /schema/{name}?service_id=test_service responds with an OK response
    ///     and the Grid Schema with the specified name and service_id.
    ///
    #[actix_rt::test]
    async fn test_fetch_schema_with_service_id_ok() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);
        clear_database();
        populate_grid_schema_table(get_grid_schema(Some(TEST_SERVICE_ID.to_string())));
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!(
                    "/schema/{}?service_id={}",
                    "TestGridSchema", TEST_SERVICE_ID
                )),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let test_schema: GridSchemaSlice =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(test_schema.name, "TestGridSchema".to_string());
        assert_eq!(test_schema.owner, "phillips001".to_string());
        assert_eq!(test_schema.properties.len(), 2);
        assert_eq!(test_schema.service_id, Some(TEST_SERVICE_ID.to_string()));
    }

    ///
    /// Verifies a GET /schema/{name} responds with a Not Found error
    ///     when there is no Grid Schema with the specified name
    ///
    #[actix_rt::test]
    async fn test_fetch_schema_not_found() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);
        clear_database();
        let response = srv
            .request(http::Method::GET, srv.url("/schema/not_in_database"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    ///
    /// Verifies a GET /schema/{name}?service_id=test_service responds with a Not Found error
    ///     when there is no Grid Schema with the specified name and service id.
    ///
    #[actix_rt::test]
    async fn test_fetch_schema_with_service_id_not_found() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);
        clear_database();
        populate_grid_schema_table(get_grid_schema(None));
        let response = srv
            .request(
                http::Method::GET,
                srv.url(&format!(
                    "/schema/not_in_database?service_id={}",
                    TEST_SERVICE_ID
                )),
            )
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    /// Verifies a GET /product responds with an OK response with a
    ///     list_products request.
    ///
    ///     The TestServer will receive a request with no parameters,
    ///         then will respond with an Ok status and a list of Products.
    #[actix_rt::test]
    async fn test_list_products() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);
        // Clears the product table in the test database
        clear_database();
        let mut response = srv
            .request(http::Method::GET, srv.url("/product"))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let empty_body: Vec<ProductSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert!(empty_body.is_empty());

        populate_product_table(get_product(None));
        let mut response = srv
            .request(http::Method::GET, srv.url("/product"))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Vec<ProductSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(body.len(), 1);

        let test_product = body.first().unwrap();
        assert_eq!(test_product.product_id, "041205707820".to_string());
        assert_eq!(test_product.product_address, "test_address".to_string());
        assert_eq!(test_product.product_namespace, "Grid Product".to_string());
        assert_eq!(test_product.owner, "phillips001".to_string());
        assert_eq!(test_product.properties.len(), 2);
    }

    /// Verifies a GET /location responds with an OK response with a
    ///     list_locations request.
    ///
    ///     The TestServer will receive a request with no parameters,
    ///         then will respond with an Ok status and a list of Locations.
    #[actix_rt::test]
    async fn test_list_locations() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);

        clear_database();

        let mut response = srv
            .request(http::Method::GET, srv.url("location"))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let empty_body: Vec<LocationSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert!(empty_body.is_empty());

        populate_location_table(get_location(None));

        let mut response = srv
            .request(http::Method::GET, srv.url("/location"))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Vec<LocationSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(body.len(), 1);

        let test_location = body.first().unwrap();
        assert_eq!(
            test_location.location_namespace,
            "Grid Location".to_string()
        );
        assert_eq!(test_location.owner, "phillips001".to_string());
        assert_eq!(test_location.properties.len(), 2);
    }

    /// Verifies a GET /product?service_id=test_service responds with an OK response with a
    ///     list_products request.
    ///
    ///     The TestServer will receive a request with a service_id,
    ///         then will respond with an Ok status and a list of Products.
    #[actix_rt::test]
    async fn test_list_products_with_service_id() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);
        // Clears the product table in the test database
        clear_database();
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/product?service_id={}", TEST_SERVICE_ID)),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let empty_body: Vec<ProductSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert!(empty_body.is_empty());

        populate_product_table(get_product(Some(TEST_SERVICE_ID.to_string())));
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/product?service_id={}", TEST_SERVICE_ID)),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Vec<ProductSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(body.len(), 1);

        let test_product = body.first().unwrap();
        assert_eq!(test_product.product_id, "041205707820".to_string());
        assert_eq!(test_product.product_address, "test_address".to_string());
        assert_eq!(test_product.product_namespace, "Grid Product".to_string());
        assert_eq!(test_product.owner, "phillips001".to_string());
        assert_eq!(test_product.properties.len(), 2);
        assert_eq!(test_product.service_id, Some(TEST_SERVICE_ID.to_string()));
    }

    /// Verifies a GET /location?service_id=test_service responds with an OK response with a
    ///     list_locations request.
    ///
    ///     The TestServer will receive a request with a service_id,
    ///         then will respond with an Ok status and a list of Products.
    #[actix_rt::test]
    async fn test_list_locations_with_service_id() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);
        // Clears the location table in the test database
        clear_database();
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/location?service_id={}", TEST_SERVICE_ID)),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let empty_body: Vec<LocationSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert!(empty_body.is_empty());

        populate_location_table(get_location(Some(TEST_SERVICE_ID.to_string())));
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/location?service_id={}", TEST_SERVICE_ID)),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Vec<LocationSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(body.len(), 1);

        let test_location = body.first().unwrap();
        assert_eq!(test_location.location_id, "0653114000000".to_string());
        assert_eq!(
            test_location.location_namespace,
            "Grid Location".to_string()
        );
        assert_eq!(test_location.owner, "phillips001".to_string());
        assert_eq!(test_location.properties.len(), 2);
        assert_eq!(test_location.service_id, Some(TEST_SERVICE_ID.to_string()));
    }

    ///
    /// Verifies a GET /product/{id} responds with an OK response
    ///     and the Product with the specified id
    ///
    #[actix_rt::test]
    async fn test_fetch_product_ok() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);
        clear_database();
        populate_product_table(get_product(None));
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/product/{}", "041205707820".to_string())),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let test_product: ProductSlice =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(test_product.product_id, "041205707820".to_string());
        assert_eq!(test_product.product_address, "test_address".to_string());
        assert_eq!(test_product.product_namespace, "Grid Product".to_string());
        assert_eq!(test_product.owner, "phillips001".to_string());
        assert_eq!(test_product.properties.len(), 2);
    }

    ///
    /// Verifies a GET /location/{id} responds with an OK response
    ///     and the Location with the specified id
    ///
    #[actix_rt::test]
    async fn test_fetch_location_ok() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);

        clear_database();

        populate_location_table(get_location(None));

        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/location/{}", "0653114000000".to_string())),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let test_location: LocationSlice =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(test_location.location_id, "0653114000000".to_string());
        assert_eq!(
            test_location.location_namespace,
            "Grid Location".to_string()
        );
        assert_eq!(test_location.owner, "phillips001".to_string());
        assert_eq!(test_location.properties.len(), 2);
    }

    ///
    /// Verifies a GET /product/{id}?service_id=test_service responds with an OK response
    ///     and the Product with the specified id and service id.
    ///
    #[actix_rt::test]
    async fn test_fetch_product_with_service_id_ok() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);
        clear_database();
        populate_product_table(get_product(Some(TEST_SERVICE_ID.to_string())));
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!(
                    "/product/{}?service_id={}",
                    "041205707820", TEST_SERVICE_ID
                )),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let test_product: ProductSlice =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(test_product.product_id, "041205707820".to_string());
        assert_eq!(test_product.product_address, "test_address".to_string());
        assert_eq!(test_product.product_namespace, "Grid Product".to_string());
        assert_eq!(test_product.owner, "phillips001".to_string());
        assert_eq!(test_product.properties.len(), 2);
        assert_eq!(test_product.service_id, Some(TEST_SERVICE_ID.to_string()));
    }

    ///
    /// Verifies a GET /location/{id}?service_id=test_service responds with an OK response
    ///     and the Location with the specified id and service id.
    ///
    #[actix_rt::test]
    async fn test_fetch_location_with_service_id_ok() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);

        clear_database();

        populate_location_table(get_location(Some(TEST_SERVICE_ID.to_string())));
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!(
                    "/location/{}?service_id={}",
                    "0653114000000", TEST_SERVICE_ID
                )),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let test_location: LocationSlice =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(test_location.location_id, "0653114000000".to_string());
        assert_eq!(
            test_location.location_namespace,
            "Grid Location".to_string()
        );
        assert_eq!(test_location.owner, "phillips001".to_string());
        assert_eq!(test_location.properties.len(), 2);
        assert_eq!(test_location.service_id, Some(TEST_SERVICE_ID.to_string()));
    }

    ///
    /// Verifies a GET /product/{id} responds with a Not Found error
    ///     when there is no Product with the specified id
    ///
    #[actix_rt::test]
    async fn test_fetch_product_not_found() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);
        clear_database();
        let response = srv
            .request(http::Method::GET, srv.url("/product/not_in_database"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    ///
    /// Verifies a GET /location/{id} responds with a Not Found error
    ///     when there is no Product with the specified id
    ///
    #[actix_rt::test]
    async fn test_fetch_location_not_found() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);
        clear_database();
        let response = srv
            .request(http::Method::GET, srv.url("/location/not_in_database"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    ///
    /// Verifies a GET /product/{id}?service_id=test_service responds with a Not Found error
    ///     when there is no Product with the specified id and service id.
    ///
    #[actix_rt::test]
    async fn test_fetch_product_with_service_id_not_found() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);
        clear_database();
        populate_product_table(get_product(None));
        let response = srv
            .request(
                http::Method::GET,
                srv.url(&format!(
                    "/product/{}?service_id={}",
                    "041205707820", TEST_SERVICE_ID
                )),
            )
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    ///
    /// Verifies a GET /location/{id}?service_id=test_service responds with a Not Found error
    ///     when there is no Product with the specified id and service id.
    ///
    #[actix_rt::test]
    async fn test_fetch_location_with_service_id_not_found() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);
        clear_database();
        populate_location_table(get_location(None));
        let response = srv
            .request(
                http::Method::GET,
                srv.url(&format!(
                    "/location/{}?service_id={}",
                    "0653114000000", TEST_SERVICE_ID
                )),
            )
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    ///
    /// Verifies a GET /record responds with an Ok response
    ///     with a list containing one record
    ///
    #[cfg(feature = "track-and-trace")]
    #[actix_rt::test]
    async fn test_list_records() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);

        clear_database();

        populate_agent_table(get_agents_with_roles(None));
        populate_associated_agent_table(get_associated_agents(None));
        populate_proposal_table(get_proposal(None));
        populate_grid_schema_table(get_grid_schema_for_record(None));
        populate_record_table(get_record("TestRecord", None));
        populate_tnt_property_table(
            get_property_for_record(None),
            get_reported_value_for_property_record(None),
            get_reporter_for_property_record(None),
        );
        let mut response = srv
            .request(http::Method::GET, srv.url(&format!("/record")))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Vec<RecordSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(body.len(), 1);
        let test_record = body.first().unwrap();
        assert_eq!(test_record.record_id, "TestRecord".to_string());
        assert_eq!(test_record.schema, "TestGridSchema".to_string());
        assert_eq!(test_record.owner, KEY1.to_string());
        assert_eq!(test_record.custodian, KEY2.to_string());
        assert_eq!(test_record.properties.len(), 2);
        assert_eq!(test_record.properties[0].name, "TestProperty1");
        assert_eq!(test_record.properties[0].record_id, "TestRecord");
        assert_eq!(test_record.properties[0].data_type, "String");
        assert_eq!(test_record.properties[0].reporters, vec![KEY1.to_string()]);
        assert_eq!(test_record.properties[0].updates.len(), 1);
        assert_eq!(
            test_record.properties[0].updates[0].value,
            Value::String("value_1".to_string())
        );
        assert_eq!(test_record.properties[0].updates[0].timestamp, 5);
        validate_reporter(&test_record.properties[0].updates[0].reporter, KEY1, None);
        assert_eq!(
            test_record.properties[0]
                .value
                .clone()
                .expect("Property value not returned")
                .value,
            Value::String("value_1".to_string())
        );
        assert_eq!(
            test_record.properties[0]
                .value
                .clone()
                .expect("Property value not returned")
                .timestamp,
            5
        );
        validate_reporter(
            &test_record.properties[0]
                .value
                .clone()
                .expect("Property value not returned")
                .reporter,
            KEY1,
            None,
        );

        assert_eq!(test_record.properties[1].name, "TestProperty2");
        assert_eq!(test_record.properties[1].record_id, "TestRecord");
        assert_eq!(test_record.properties[1].data_type, "Boolean");
        assert_eq!(test_record.properties[1].reporters, vec![KEY2.to_string()]);
        assert_eq!(test_record.properties[1].updates.len(), 1);
        assert_eq!(
            test_record.properties[1].updates[0].value,
            Value::Bool(true)
        );
        assert_eq!(test_record.properties[1].updates[0].timestamp, 5);
        validate_reporter(&test_record.properties[1].updates[0].reporter, KEY2, None);
        assert_eq!(
            test_record.properties[1]
                .value
                .clone()
                .expect("Property value not returned")
                .value,
            Value::Bool(true)
        );
        assert_eq!(
            test_record.properties[1]
                .value
                .clone()
                .expect("Property value not returned")
                .timestamp,
            5
        );
        validate_reporter(
            &test_record.properties[1]
                .value
                .clone()
                .expect("Property value not returned")
                .reporter,
            KEY2,
            None,
        );

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
    /// Verifies a GET /record?service_id=test_service responds with an Ok response
    ///     with a list containing one record with the correct service id.
    ///
    #[actix_rt::test]
    #[cfg(feature = "track-and-trace")]
    async fn test_list_records_with_service_id() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);

        clear_database();

        populate_agent_table(get_agents_with_roles(Some(TEST_SERVICE_ID.to_string())));
        populate_associated_agent_table(get_associated_agents(Some(TEST_SERVICE_ID.to_string())));
        populate_proposal_table(get_proposal(Some(TEST_SERVICE_ID.to_string())));
        populate_grid_schema_table(get_grid_schema_for_record(None));
        populate_record_table(get_record("TestRecord", Some(TEST_SERVICE_ID.to_string())));
        populate_tnt_property_table(
            get_property_for_record(Some(TEST_SERVICE_ID.to_string())),
            get_reported_value_for_property_record(Some(TEST_SERVICE_ID.to_string())),
            get_reporter_for_property_record(Some(TEST_SERVICE_ID.to_string())),
        );
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/record?service_id={}", TEST_SERVICE_ID)),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Vec<RecordSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(body.len(), 1);
        let test_record = body.first().unwrap();
        assert_eq!(test_record.record_id, "TestRecord".to_string());
        assert_eq!(test_record.schema, "TestGridSchema".to_string());
        assert_eq!(test_record.owner, KEY1.to_string());
        assert_eq!(test_record.custodian, KEY2.to_string());
        assert_eq!(test_record.service_id, Some(TEST_SERVICE_ID.to_string()));
        assert_eq!(test_record.properties.len(), 2);
        assert_eq!(test_record.properties[0].name, "TestProperty1");
        assert_eq!(test_record.properties[0].record_id, "TestRecord");
        assert_eq!(test_record.properties[0].data_type, "String");
        assert_eq!(test_record.properties[0].reporters, vec![KEY1.to_string()]);
        assert_eq!(test_record.properties[0].updates.len(), 1);
        assert_eq!(
            test_record.properties[0].updates[0].value,
            Value::String("value_1".to_string())
        );
        assert_eq!(test_record.properties[0].updates[0].timestamp, 5);
        assert_eq!(
            test_record.properties[0].service_id,
            Some(TEST_SERVICE_ID.to_string())
        );
        validate_reporter(
            &test_record.properties[0].updates[0].reporter,
            KEY1,
            Some(TEST_SERVICE_ID.to_string()),
        );
        assert_eq!(
            test_record.properties[0]
                .value
                .clone()
                .expect("Property value not returned")
                .value,
            Value::String("value_1".to_string())
        );
        assert_eq!(
            test_record.properties[0]
                .value
                .clone()
                .expect("Property value not returned")
                .timestamp,
            5
        );
        assert_eq!(
            test_record.properties[0]
                .value
                .clone()
                .expect("Property value not returned")
                .service_id,
            Some(TEST_SERVICE_ID.to_string()),
        );
        validate_reporter(
            &test_record.properties[0]
                .value
                .clone()
                .expect("Property value not returned")
                .reporter,
            KEY1,
            Some(TEST_SERVICE_ID.to_string()),
        );

        assert_eq!(test_record.properties[1].name, "TestProperty2");
        assert_eq!(test_record.properties[1].record_id, "TestRecord");
        assert_eq!(test_record.properties[1].data_type, "Boolean");
        assert_eq!(test_record.properties[1].reporters, vec![KEY2.to_string()]);
        assert_eq!(test_record.properties[1].updates.len(), 1);
        assert_eq!(
            test_record.properties[1].updates[0].value,
            Value::Bool(true)
        );
        assert_eq!(test_record.properties[1].updates[0].timestamp, 5);
        assert_eq!(
            test_record.properties[1].updates[0].service_id,
            Some(TEST_SERVICE_ID.to_string())
        );
        validate_reporter(
            &test_record.properties[1].updates[0].reporter,
            KEY2,
            Some(TEST_SERVICE_ID.to_string()),
        );
        assert_eq!(
            test_record.properties[1]
                .value
                .clone()
                .expect("Property value not returned")
                .value,
            Value::Bool(true)
        );
        assert_eq!(
            test_record.properties[1]
                .value
                .clone()
                .expect("Property value not returned")
                .timestamp,
            5
        );
        assert_eq!(
            test_record.properties[1]
                .value
                .clone()
                .expect("Property value not returned")
                .service_id,
            Some(TEST_SERVICE_ID.to_string()),
        );
        validate_reporter(
            &test_record.properties[1]
                .value
                .clone()
                .expect("Property value not returned")
                .reporter,
            KEY2,
            Some(TEST_SERVICE_ID.to_string()),
        );

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
        assert_eq!(
            test_record.proposals[0].service_id,
            Some(TEST_SERVICE_ID.to_string())
        );
    }

    ///
    /// Verifies a GET /record responds with an Ok response
    /// with a list containing one record, when there's two records for the same
    /// record_id. The rest-api should return a list with a single record with the
    /// record that contains the most recent information for that record
    /// (end_commit_num == MAX_COMMIT_NUM)
    ///
    #[actix_rt::test]
    #[cfg(feature = "track-and-trace")]
    async fn test_list_records_updated() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);

        clear_database();
        // Adds two instances of record with the same org_id to the test database
        populate_record_table(get_updated_record());

        // Making another request to the database
        let mut response = srv
            .request(http::Method::GET, srv.url("/record"))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Vec<RecordSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(body.len(), 1);
        let test_record = body.first().unwrap();
        assert_eq!(test_record.record_id, "TestRecord".to_string());
        assert_eq!(test_record.schema, "TestGridSchema".to_string());
        assert_eq!(test_record.r#final, true);
    }

    ///
    /// Verifies a GET /record responds with an Ok response
    /// with a list containing two records, when there's two records with differing
    /// record_ids, one of which has been updated.
    ///
    #[actix_rt::test]
    #[cfg(feature = "track-and-trace")]
    async fn test_list_records_multiple() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);

        clear_database();
        // Adds two instances of record with the same org_id to the test database
        populate_record_table(get_multiple_records());
        populate_tnt_property_table(
            get_property_for_record(None),
            get_reported_value_for_property_record(None),
            get_reporter_for_property_record(None),
        );

        // Making another request to the database
        let mut response = srv
            .request(http::Method::GET, srv.url("/record"))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: Vec<RecordSlice> =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();

        assert_eq!(body.len(), 2);
        let record_1 = &body[0];
        assert_eq!(record_1.properties.len(), 2);

        let record_2 = &body[1];
        assert!(record_2.properties.is_empty());
    }

    ///
    /// Verifies a GET /record/{record_id} responds with an OK response
    ///     and the Record with the specified record ID.
    ///
    #[actix_rt::test]
    #[cfg(feature = "track-and-trace")]
    async fn test_fetch_record_ok() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);
        clear_database();

        populate_agent_table(get_agents_with_roles(None));
        populate_associated_agent_table(get_associated_agents(None));
        populate_proposal_table(get_proposal(None));
        populate_record_table(get_record("TestRecord", None));
        populate_grid_schema_table(get_grid_schema_for_record(None));
        populate_tnt_property_table(
            get_property_for_record(None),
            get_reported_value_for_property_record(None),
            get_reporter_for_property_record(None),
        );
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/record/{}", "TestRecord".to_string())),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let test_record: RecordSlice =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(test_record.record_id, "TestRecord".to_string());
        assert_eq!(test_record.schema, "TestGridSchema".to_string());
        assert_eq!(test_record.owner, KEY1.to_string());
        assert_eq!(test_record.custodian, KEY2.to_string());

        assert_eq!(test_record.properties.len(), 2);
        assert_eq!(test_record.properties[0].name, "TestProperty1");
        assert_eq!(test_record.properties[0].record_id, "TestRecord");
        assert_eq!(test_record.properties[0].data_type, "String");
        assert_eq!(test_record.properties[0].reporters, vec![KEY1.to_string()]);
        assert_eq!(test_record.properties[0].updates.len(), 1);
        assert_eq!(
            test_record.properties[0].updates[0].value,
            Value::String("value_1".to_string())
        );
        assert_eq!(test_record.properties[0].updates[0].timestamp, 5);
        validate_reporter(&test_record.properties[0].updates[0].reporter, KEY1, None);
        assert_eq!(
            test_record.properties[0]
                .value
                .clone()
                .expect("Property value not returned")
                .value,
            Value::String("value_1".to_string())
        );
        assert_eq!(
            test_record.properties[0]
                .value
                .clone()
                .expect("Property value not returned")
                .timestamp,
            5
        );
        validate_reporter(
            &test_record.properties[0]
                .value
                .clone()
                .expect("Property value not returned")
                .reporter,
            KEY1,
            None,
        );

        assert_eq!(test_record.properties[1].name, "TestProperty2");
        assert_eq!(test_record.properties[1].record_id, "TestRecord");
        assert_eq!(test_record.properties[1].data_type, "Boolean");
        assert_eq!(test_record.properties[1].reporters, vec![KEY2.to_string()]);
        assert_eq!(test_record.properties[1].updates.len(), 1);
        assert_eq!(
            test_record.properties[1].updates[0].value,
            Value::Bool(true)
        );
        assert_eq!(test_record.properties[1].updates[0].timestamp, 5);
        validate_reporter(&test_record.properties[1].updates[0].reporter, KEY2, None);
        assert_eq!(
            test_record.properties[1]
                .value
                .clone()
                .expect("Property value not returned")
                .value,
            Value::Bool(true)
        );
        assert_eq!(
            test_record.properties[1]
                .value
                .clone()
                .expect("Property value not returned")
                .timestamp,
            5
        );
        validate_reporter(
            &test_record.properties[1]
                .value
                .clone()
                .expect("Property value not returned")
                .reporter,
            KEY2,
            None,
        );

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
    /// Verifies a GET /record/{record_id}?service_id=test_service responds with an OK response
    ///     and the Record with the specified record ID.
    ///
    #[actix_rt::test]
    #[cfg(feature = "track-and-trace")]
    async fn test_fetch_record_with_service_id_ok() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);

        clear_database();

        populate_agent_table(get_agents_with_roles(Some(TEST_SERVICE_ID.to_string())));
        populate_associated_agent_table(get_associated_agents(Some(TEST_SERVICE_ID.to_string())));
        populate_proposal_table(get_proposal(Some(TEST_SERVICE_ID.to_string())));
        populate_record_table(get_record("TestRecord", Some(TEST_SERVICE_ID.to_string())));
        populate_grid_schema_table(get_grid_schema_for_record(None));
        populate_tnt_property_table(
            get_property_for_record(Some(TEST_SERVICE_ID.to_string())),
            get_reported_value_for_property_record(Some(TEST_SERVICE_ID.to_string())),
            get_reporter_for_property_record(Some(TEST_SERVICE_ID.to_string())),
        );

        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!(
                    "/record/{}?service_id={}",
                    "TestRecord", TEST_SERVICE_ID
                )),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let test_record: RecordSlice =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();
        assert_eq!(test_record.record_id, "TestRecord".to_string());
        assert_eq!(test_record.schema, "TestGridSchema".to_string());
        assert_eq!(test_record.owner, KEY1.to_string());
        assert_eq!(test_record.custodian, KEY2.to_string());
        assert_eq!(test_record.service_id, Some(TEST_SERVICE_ID.to_string()));

        assert_eq!(test_record.properties.len(), 2);
        assert_eq!(test_record.properties[0].name, "TestProperty1");
        assert_eq!(test_record.properties[0].record_id, "TestRecord");
        assert_eq!(test_record.properties[0].data_type, "String");
        assert_eq!(
            test_record.properties[0].service_id,
            Some(TEST_SERVICE_ID.to_string())
        );
        assert_eq!(test_record.properties[0].reporters, vec![KEY1.to_string()]);
        assert_eq!(test_record.properties[0].updates.len(), 1);
        assert_eq!(
            test_record.properties[0].updates[0].value,
            Value::String("value_1".to_string())
        );
        assert_eq!(test_record.properties[0].updates[0].timestamp, 5);
        validate_reporter(
            &test_record.properties[0].updates[0].reporter,
            KEY1,
            Some(TEST_SERVICE_ID.to_string()),
        );
        assert_eq!(
            test_record.properties[0]
                .value
                .clone()
                .expect("Property value not returned")
                .value,
            Value::String("value_1".to_string())
        );
        assert_eq!(
            test_record.properties[0]
                .value
                .clone()
                .expect("Property value not returned")
                .timestamp,
            5
        );
        validate_reporter(
            &test_record.properties[0]
                .value
                .clone()
                .expect("Property value not returned")
                .reporter,
            KEY1,
            Some(TEST_SERVICE_ID.to_string()),
        );

        assert_eq!(test_record.properties[1].name, "TestProperty2");
        assert_eq!(test_record.properties[1].record_id, "TestRecord");
        assert_eq!(test_record.properties[1].data_type, "Boolean");
        assert_eq!(test_record.properties[1].reporters, vec![KEY2.to_string()]);
        assert_eq!(test_record.properties[1].updates.len(), 1);
        assert_eq!(
            test_record.properties[1].updates[0].value,
            Value::Bool(true)
        );
        assert_eq!(test_record.properties[1].updates[0].timestamp, 5);
        validate_reporter(
            &test_record.properties[1].updates[0].reporter,
            KEY2,
            Some(TEST_SERVICE_ID.to_string()),
        );
        assert_eq!(
            test_record.properties[1]
                .value
                .clone()
                .expect("Property value not returned")
                .value,
            Value::Bool(true)
        );
        assert_eq!(
            test_record.properties[1]
                .value
                .clone()
                .expect("Property value not returned")
                .timestamp,
            5
        );
        assert_eq!(
            test_record.properties[1]
                .value
                .clone()
                .expect("Property value not returned")
                .service_id,
            Some(TEST_SERVICE_ID.to_string()),
        );
        validate_reporter(
            &test_record.properties[1]
                .value
                .clone()
                .expect("Property value not returned")
                .reporter,
            KEY2,
            Some(TEST_SERVICE_ID.to_string()),
        );

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
        assert_eq!(
            test_record.proposals[0].service_id,
            Some(TEST_SERVICE_ID.to_string())
        );
    }

    ///
    /// Verifies a GET /record/{record_id} responds with an OK response
    ///     and the Record with the specified record ID after the Record's
    ///     owners, custodians, and proposals have been updated.
    ///
    #[actix_rt::test]
    #[cfg(feature = "track-and-trace")]
    async fn test_fetch_record_updated_ok() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);

        clear_database();

        populate_grid_schema_table(get_grid_schema_for_record(None));
        populate_record_table(get_updated_record());
        populate_tnt_property_table(
            get_property_for_record(None),
            get_reported_value_for_property_record(None),
            get_reporter_for_property_record(None),
        );
        populate_associated_agent_table(get_associated_agents_updated());
        populate_proposal_table(get_updated_proposal());
        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!("/record/{}", "TestRecord".to_string())),
            )
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let test_record: RecordSlice =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();

        assert_eq!(test_record.record_id, "TestRecord".to_string());
        assert_eq!(test_record.schema, "TestGridSchema".to_string());
        assert_eq!(test_record.owner, KEY2.to_string());
        assert_eq!(test_record.custodian, KEY1.to_string());

        assert_eq!(test_record.properties.len(), 2);
        assert_eq!(test_record.properties[0].name, "TestProperty1");
        assert_eq!(test_record.properties[0].record_id, "TestRecord");
        assert_eq!(test_record.properties[0].data_type, "String");
        assert_eq!(test_record.properties[0].reporters, vec![KEY1.to_string()]);
        assert_eq!(test_record.properties[0].updates.len(), 1);
        assert_eq!(
            test_record.properties[0].updates[0].value,
            Value::String("value_1".to_string())
        );
        assert_eq!(test_record.properties[0].updates[0].timestamp, 5);
        validate_reporter(&test_record.properties[0].updates[0].reporter, KEY1, None);
        assert_eq!(
            test_record.properties[0]
                .value
                .clone()
                .expect("Property value not returned")
                .value,
            Value::String("value_1".to_string())
        );
        assert_eq!(
            test_record.properties[0]
                .value
                .clone()
                .expect("Property value not returned")
                .timestamp,
            5
        );
        validate_reporter(
            &test_record.properties[0]
                .value
                .clone()
                .expect("Property value not returned")
                .reporter,
            KEY1,
            None,
        );

        assert_eq!(test_record.properties[1].name, "TestProperty2");
        assert_eq!(test_record.properties[1].record_id, "TestRecord");
        assert_eq!(test_record.properties[1].data_type, "Boolean");
        assert_eq!(test_record.properties[1].reporters, vec![KEY2.to_string()]);
        assert_eq!(test_record.properties[1].updates.len(), 1);
        assert_eq!(
            test_record.properties[1].updates[0].value,
            Value::Bool(true)
        );
        assert_eq!(test_record.properties[1].updates[0].timestamp, 5);
        validate_reporter(&test_record.properties[1].updates[0].reporter, KEY2, None);
        assert_eq!(
            test_record.properties[1]
                .value
                .clone()
                .expect("Property value not returned")
                .value,
            Value::Bool(true)
        );
        assert_eq!(
            test_record.properties[1]
                .value
                .clone()
                .expect("Property value not returned")
                .timestamp,
            5
        );
        validate_reporter(
            &test_record.properties[1]
                .value
                .clone()
                .expect("Property value not returned")
                .reporter,
            KEY2,
            None,
        );

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
    #[actix_rt::test]
    #[cfg(feature = "track-and-trace")]
    async fn test_fetch_record_not_found() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);
        clear_database();
        let response = srv
            .request(http::Method::GET, srv.url("/record/not_in_database"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    ///
    /// Verifies a GET /record/{record_id}?service_id=test_service responds with a Not Found error
    ///     when there is no Record with the specified record_id.
    ///
    #[actix_rt::test]
    #[cfg(feature = "track-and-trace")]
    async fn test_fetch_record_with_service_id_not_found() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);

        clear_database();

        let response = srv
            .request(
                http::Method::GET,
                srv.url(&format!(
                    "/record/not_in_database?service_id={}",
                    TEST_SERVICE_ID
                )),
            )
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    ///
    /// Verifies a GET /record/{record_id}/property/{property_name} responds with an OK response
    ///     and the infomation on the Property requested
    ///
    #[actix_rt::test]
    #[cfg(feature = "track-and-trace")]
    async fn test_fetch_record_property_ok() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);

        clear_database();

        populate_grid_schema_table(get_grid_schema_for_struct_record(None));
        populate_record_table(get_record("record_01", None));
        populate_tnt_property_table(
            get_property(None),
            get_reported_value(None),
            get_reporter(None),
        );

        let mut response = srv
            .request(
                http::Method::GET,
                srv.url("/record/record_01/property/TestProperty"),
            )
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());

        let property_info: PropertySlice =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();

        assert_eq!(property_info.data_type, "Struct".to_string());
        assert_eq!(property_info.name, "TestProperty".to_string());
        assert_eq!(property_info.record_id, "record_01".to_string());

        assert_eq!(
            property_info.reporters,
            vec![KEY1.to_string(), KEY2.to_string()]
        );

        validate_current_value(
            &property_info
                .value
                .clone()
                .expect("Property value not returned"),
            None,
        );

        assert_eq!(property_info.updates.len(), 2);

        let first_update = &property_info.updates[0];

        validate_reporter(&first_update.reporter, KEY2, None);

        assert_eq!(first_update.timestamp, 3);

        match &first_update.value {
            Value::Struct(root) => {
                assert_eq!(root.len(), 1);
                match &root[0].value {
                    Value::Struct(struct_values) => {
                        assert_eq!(struct_values.len(), 5);
                        validate_struct_value(&struct_values[0], "value_1", false);
                        validate_location_value(
                            &struct_values[1],
                            LatLong {
                                latitude: 1,
                                longitude: 1,
                            },
                        );
                        validate_number_value(&struct_values[2], 1);
                        validate_enum_value(&struct_values[3], 1);
                        validate_bytes_value(&struct_values[4], &vec![0x01, 0x02, 0x03, 0x04]);
                    }
                    _ => panic!("Expected enum type Struct found: {:?}", first_update.value),
                }
            }
            _ => panic!("Expected enum type Struct found: {:?}", first_update.value),
        }

        let second_update = &property_info.updates[1];
        validate_current_value(second_update, None);
    }

    ///
    /// Verifies a GET /record/{record_id}/property/{property_name}?service_id=test_service responds
    ///     with an OK response and the infomation on the Property requested.
    ///
    #[actix_rt::test]
    #[cfg(feature = "track-and-trace")]
    async fn test_fetch_record_property_with_service_id_ok() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Splinter, ResponseType::ClientBatchStatusResponseOK);

        clear_database();

        populate_grid_schema_table(get_grid_schema_for_struct_record(Some(
            TEST_SERVICE_ID.to_string(),
        )));
        populate_record_table(get_record("record_01", Some(TEST_SERVICE_ID.to_string())));
        populate_tnt_property_table(
            get_property(Some(TEST_SERVICE_ID.to_string())),
            get_reported_value(Some(TEST_SERVICE_ID.to_string())),
            get_reporter(Some(TEST_SERVICE_ID.to_string())),
        );

        let mut response = srv
            .request(
                http::Method::GET,
                srv.url(&format!(
                    "/record/record_01/property/TestProperty?service_id={}",
                    TEST_SERVICE_ID
                )),
            )
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());

        let property_info: PropertySlice =
            serde_json::from_slice(&*response.body().await.unwrap()).unwrap();

        assert_eq!(property_info.data_type, "Struct".to_string());
        assert_eq!(property_info.name, "TestProperty".to_string());
        assert_eq!(property_info.record_id, "record_01".to_string());
        assert_eq!(property_info.service_id, Some(TEST_SERVICE_ID.to_string()));

        assert_eq!(
            property_info.reporters,
            vec![KEY1.to_string(), KEY2.to_string()]
        );

        validate_current_value(
            &property_info
                .value
                .clone()
                .expect("Property value not returned"),
            Some(TEST_SERVICE_ID.to_string()),
        );

        assert_eq!(property_info.updates.len(), 2);

        let first_update = &property_info.updates[0];

        validate_reporter(
            &first_update.reporter,
            KEY2,
            Some(TEST_SERVICE_ID.to_string()),
        );

        assert_eq!(first_update.timestamp, 3);

        match &first_update.value {
            Value::Struct(root) => {
                assert_eq!(root.len(), 1);
                match &root[0].value {
                    Value::Struct(struct_values) => {
                        assert_eq!(struct_values.len(), 5);
                        validate_struct_value(&struct_values[0], "value_1", false);
                        validate_location_value(
                            &struct_values[1],
                            LatLong {
                                latitude: 1,
                                longitude: 1,
                            },
                        );
                        validate_number_value(&struct_values[2], 1);
                        validate_enum_value(&struct_values[3], 1);
                        validate_bytes_value(&struct_values[4], &vec![0x01, 0x02, 0x03, 0x04]);
                    }
                    _ => panic!("Expected enum type Struct found: {:?}", first_update.value),
                }
            }
            _ => panic!("Expected enum type Struct found: {:?}", first_update.value),
        }

        let second_update = &property_info.updates[1];
        validate_current_value(second_update, Some(TEST_SERVICE_ID.to_string()));
    }

    #[cfg(feature = "track-and-trace")]
    fn validate_current_value(property_value: &PropertyValueSlice, service_id: Option<String>) {
        validate_reporter(&property_value.reporter, KEY1, service_id.clone());
        assert_eq!(property_value.timestamp, 5);
        assert_eq!(property_value.service_id, service_id);
        match &property_value.value {
            Value::Struct(root) => {
                assert_eq!(root.len(), 1);
                match &root[0].value {
                    Value::Struct(struct_values) => {
                        assert_eq!(struct_values.len(), 5);
                        validate_struct_value(&struct_values[0], "value_updated", true);
                        validate_location_value(
                            &struct_values[1],
                            LatLong {
                                latitude: 2,
                                longitude: 2,
                            },
                        );
                        validate_number_value(&struct_values[2], 2);
                        validate_enum_value(&struct_values[3], 2);
                        validate_bytes_value(&struct_values[4], &vec![0x05, 0x06, 0x07, 0x08]);
                    }
                    _ => panic!(
                        "Expected enum type Struct found: {:?}",
                        property_value.value
                    ),
                }
            }
            _ => panic!("Expected struct with single element"),
        }
    }

    #[cfg(feature = "track-and-trace")]
    fn validate_reporter(reporter: &ReporterSlice, public_key: &str, service_id: Option<String>) {
        assert_eq!(reporter.public_key, public_key.to_string());
        assert_eq!(reporter.service_id, service_id);
    }

    #[cfg(feature = "track-and-trace")]
    fn validate_struct_value(
        struct_value: &StructPropertyValue,
        expected_string_value: &str,
        expected_bool_value: bool,
    ) {
        assert_eq!(struct_value.data_type, "Struct".to_string());

        assert!(struct_value.name.contains("StructProperty"));

        match &struct_value.value {
            Value::Struct(inner_values) => {
                assert_eq!(inner_values.len(), 2);
                validate_string_value(&inner_values[0], expected_string_value);
                validate_boolean_value(&inner_values[1], expected_bool_value);
            }
            _ => panic!("Expected enum type Struct found: {:?}", struct_value.value),
        }
    }

    #[cfg(feature = "track-and-trace")]
    fn validate_string_value(string_value: &StructPropertyValue, expected_value: &str) {
        assert_eq!(string_value.data_type, "String".to_string());
        assert!(string_value.name.contains("StringProperty"));
        assert_eq!(
            string_value.value,
            Value::String(expected_value.to_string())
        );
    }

    #[cfg(feature = "track-and-trace")]
    fn validate_boolean_value(boolean_value: &StructPropertyValue, expected_value: bool) {
        assert_eq!(boolean_value.data_type, "Boolean".to_string());
        assert!(boolean_value.name.contains("BoolProperty"));
        assert_eq!(boolean_value.value, Value::Bool(expected_value));
    }

    #[cfg(feature = "track-and-trace")]
    fn validate_location_value(location_value: &StructPropertyValue, expected_value: LatLong) {
        assert_eq!(location_value.data_type, "LatLong".to_string());
        assert!(location_value.name.contains("LatLongProperty"));
        assert_eq!(location_value.value, Value::LatLong(expected_value));
    }

    #[cfg(feature = "track-and-trace")]
    fn validate_number_value(number_value: &StructPropertyValue, expected_value: i64) {
        assert_eq!(number_value.data_type, "Number".to_string());
        assert!(number_value.name.contains("NumberProperty"));
        assert_eq!(number_value.value, Value::Number(expected_value));
    }

    #[cfg(feature = "track-and-trace")]
    fn validate_enum_value(enum_value: &StructPropertyValue, expected_value: i32) {
        assert_eq!(enum_value.data_type, "Enum".to_string());
        assert!(enum_value.name.contains("EnumProperty"));
        match enum_value.value {
            Value::Enum(val) => assert_eq!(val, expected_value),
            Value::Number(val) => assert_eq!(val as i32, expected_value),
            _ => panic!("Expected enum value, found {:?}", enum_value.value),
        }
    }

    #[cfg(feature = "track-and-trace")]
    fn validate_bytes_value(bytes_value: &StructPropertyValue, expected_value: &[u8]) {
        assert_eq!(bytes_value.data_type, "Bytes".to_string());
        assert!(bytes_value.name.contains("BytesProperty"));
        match &bytes_value.value {
            Value::String(val) => assert_eq!(val, &base64::encode(expected_value)),
            Value::Bytes(val) => assert_eq!(val, &base64::encode(expected_value)),
            _ => panic!("Expected bytes value, found {:?}", bytes_value.value),
        }
    }

    ///
    /// Verifies a GET /record/{record_id}/property/{property_name} responds with a Not Found
    /// error when there is no property with the specified property_name.
    ///
    #[actix_rt::test]
    #[cfg(feature = "track-and-trace")]
    async fn test_fetch_property_name_not_found() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);
        clear_database();
        let response = srv
            .request(
                http::Method::GET,
                srv.url("/record/record_01/property/not_in_database"),
            )
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), http::StatusCode::NOT_FOUND);
    }

    ///
    /// Verifies a GET /record/{record_id}/property/{property_name} responds with a Not Found
    /// error when there is no record with the specified record_id.
    ///
    #[actix_rt::test]
    #[cfg(feature = "track-and-trace")]
    async fn test_fetch_property_record_id_not_found() {
        run_migrations(&DATABASE_URL);
        let srv = create_test_server(Backend::Sawtooth, ResponseType::ClientBatchStatusResponseOK);

        clear_database();

        populate_tnt_property_table(
            get_property(None),
            get_reported_value(None),
            get_reporter(None),
        );
        let response = srv
            .request(
                http::Method::GET,
                srv.url("/record/not_in_database/property/TestProperty"),
            )
            .send()
            .await
            .unwrap();

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

    fn get_agent(service_id: Option<String>) -> Vec<Agent> {
        vec![Agent {
            public_key: KEY1.to_string(),
            org_id: KEY2.to_string(),
            active: true,
            roles: vec![],
            metadata: vec![],
            start_commit_num: 0,
            end_commit_num: i64::MAX,
            service_id,
        }]
    }

    #[cfg(feature = "track-and-trace")]
    fn get_agents_with_roles(service_id: Option<String>) -> Vec<Agent> {
        vec![
            Agent {
                public_key: KEY1.to_string(),
                org_id: KEY3.to_string(),
                active: true,
                roles: vec!["OWNER".to_string()],
                metadata: vec![],
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                service_id: service_id.clone(),
            },
            Agent {
                public_key: KEY2.to_string(),
                org_id: KEY3.to_string(),
                active: true,
                roles: vec!["CUSTODIAN".to_string()],
                metadata: vec![],
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                service_id,
            },
        ]
    }

    fn populate_agent_table(agents: Vec<Agent>) {
        let pool = get_connection_pool();
        let store = DieselAgentStore::new(pool.pool);
        agents
            .into_iter()
            .for_each(|agent| store.add_agent(agent).unwrap());
    }

    fn get_organization(service_id: Option<String>) -> Vec<Organization> {
        vec![Organization {
            org_id: KEY2.to_string(),
            name: ORG_NAME_1.to_string(),
            address: ADDRESS_1.to_string(),
            metadata: vec![],
            start_commit_num: 1,
            end_commit_num: i64::MAX,
            service_id,
        }]
    }

    fn get_updated_organization() -> Vec<Organization> {
        vec![
            Organization {
                org_id: KEY3.to_string(),
                name: ORG_NAME_2.to_string(),
                address: ADDRESS_2.to_string(),
                metadata: vec![],
                start_commit_num: 2,
                end_commit_num: 4,
                service_id: None,
            },
            Organization {
                org_id: KEY3.to_string(),
                name: ORG_NAME_2.to_string(),
                address: UPDATED_ADDRESS_2.to_string(),
                metadata: vec![],
                start_commit_num: 4,
                end_commit_num: i64::MAX,
                service_id: None,
            },
        ]
    }

    fn populate_organization_table(organizations: Vec<Organization>) {
        let pool = get_connection_pool();
        let store = DieselOrganizationStore::new(pool.pool);
        store.add_organizations(organizations).unwrap();
    }

    fn get_grid_schema(service_id: Option<String>) -> Vec<Schema> {
        vec![Schema {
            start_commit_num: 0,
            end_commit_num: i64::MAX,
            name: "TestGridSchema".to_string(),
            properties: get_property_definition(service_id.clone()),
            description: "Example test grid schema".to_string(),
            owner: "phillips001".to_string(),
            service_id,
        }]
    }

    fn get_product(service_id: Option<String>) -> Vec<Product> {
        vec![Product {
            product_id: "041205707820".to_string(),
            product_address: "test_address".to_string(),
            product_namespace: "Grid Product".to_string(),
            owner: "phillips001".to_string(),
            start_commit_num: 0,
            end_commit_num: i64::MAX,
            properties: get_product_property_value(service_id.clone()),
            service_id,
        }]
    }

    fn populate_location_table(locations: Vec<Location>) {
        let pool = get_connection_pool();
        let store = DieselLocationStore::new(pool.pool);
        locations
            .into_iter()
            .for_each(|location| store.add_location(location).unwrap());
    }

    fn get_location(service_id: Option<String>) -> Vec<Location> {
        vec![Location {
            location_id: "0653114000000".to_string(),
            location_address: "location-address".to_string(),
            location_namespace: "Grid Location".to_string(),
            owner: "phillips001".to_string(),
            attributes: get_location_attributes(service_id.clone()),
            start_commit_num: 0,
            end_commit_num: i64::MAX,
            service_id,
        }]
    }

    fn get_location_attributes(service_id: Option<String>) -> Vec<LocationAttribute> {
        vec![
            LocationAttribute {
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                location_id: "0653114000000".to_string(),
                location_address: "test_address".to_string(),
                property_name: "location_name".to_string(),
                data_type: "STRING".to_string(),
                bytes_value: None,
                boolean_value: None,
                number_value: Some(0),
                string_value: Some("My Warehouse".to_string()),
                enum_value: None,
                struct_values: None,
                lat_long_value: None,
                service_id: service_id.clone(),
            },
            LocationAttribute {
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                location_id: "0653114000000".to_string(),
                location_address: "test_address".to_string(),
                property_name: "industry_sector".to_string(),
                data_type: "STRING".to_string(),
                bytes_value: None,
                boolean_value: None,
                number_value: Some(0),
                string_value: Some("Light bulbs".to_string()),
                enum_value: None,
                struct_values: None,
                lat_long_value: None,
                service_id,
            },
        ]
    }

    #[cfg(feature = "track-and-trace")]
    fn get_associated_agents(service_id: Option<String>) -> Vec<AssociatedAgent> {
        vec![
            AssociatedAgent {
                id: None,
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                agent_id: KEY1.to_string(),
                timestamp: 1,
                record_id: "TestRecord".to_string(),
                role: "OWNER".to_string(),
                service_id: service_id.clone(),
            },
            AssociatedAgent {
                id: None,
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                agent_id: KEY2.to_string(),
                timestamp: 1,
                record_id: "TestRecord".to_string(),
                role: "CUSTODIAN".to_string(),
                service_id,
            },
        ]
    }

    #[cfg(feature = "track-and-trace")]
    fn get_associated_agents_updated() -> Vec<AssociatedAgent> {
        vec![
            AssociatedAgent {
                id: None,
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                agent_id: KEY1.to_string(),
                timestamp: 1,
                record_id: "TestRecord".to_string(),
                role: "OWNER".to_string(),
                service_id: None,
            },
            AssociatedAgent {
                id: None,
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                agent_id: KEY2.to_string(),
                timestamp: 1,
                record_id: "TestRecord".to_string(),
                role: "CUSTODIAN".to_string(),
                service_id: None,
            },
            AssociatedAgent {
                id: None,
                start_commit_num: 1,
                end_commit_num: i64::MAX,
                agent_id: KEY2.to_string(),
                timestamp: 2,
                record_id: "TestRecord".to_string(),
                role: "OWNER".to_string(),
                service_id: None,
            },
            AssociatedAgent {
                id: None,
                start_commit_num: 1,
                end_commit_num: i64::MAX,
                agent_id: KEY1.to_string(),
                timestamp: 2,
                record_id: "TestRecord".to_string(),
                role: "CUSTODIAN".to_string(),
                service_id: None,
            },
        ]
    }

    #[cfg(feature = "track-and-trace")]
    fn get_proposal(service_id: Option<String>) -> Vec<Proposal> {
        vec![Proposal {
            id: None,
            start_commit_num: 0,
            end_commit_num: i64::MAX,
            record_id: "TestRecord".to_string(),
            timestamp: 1,
            issuing_agent: KEY1.to_string(),
            receiving_agent: KEY2.to_string(),
            properties: vec!["location".to_string()],
            role: "OWNER".to_string(),
            status: "OPEN".to_string(),
            terms: "Proposal Terms".to_string(),
            service_id,
        }]
    }

    #[cfg(feature = "track-and-trace")]
    fn get_updated_proposal() -> Vec<Proposal> {
        vec![
            Proposal {
                id: None,
                start_commit_num: 0,
                end_commit_num: 1,
                record_id: "TestRecord".to_string(),
                timestamp: 1,
                issuing_agent: KEY1.to_string(),
                receiving_agent: KEY2.to_string(),
                properties: vec!["location".to_string()],
                role: "OWNER".to_string(),
                status: "OPEN".to_string(),
                terms: "Proposal Terms".to_string(),
                service_id: None,
            },
            Proposal {
                id: None,
                start_commit_num: 1,
                end_commit_num: i64::MAX,
                record_id: "TestRecord".to_string(),
                timestamp: 1,
                issuing_agent: KEY1.to_string(),
                receiving_agent: KEY2.to_string(),
                properties: vec!["location".to_string()],
                role: "OWNER".to_string(),
                status: "CANCELED".to_string(),
                terms: "Proposal Terms".to_string(),
                service_id: None,
            },
        ]
    }

    #[cfg(feature = "track-and-trace")]
    fn get_record(record_id: &str, service_id: Option<String>) -> Vec<Record> {
        vec![Record {
            id: None,
            start_commit_num: 0,
            end_commit_num: i64::MAX,
            record_id: record_id.to_string(),
            schema: "TestGridSchema".to_string(),
            final_: false,
            owners: vec![KEY1.to_string()],
            custodians: vec![KEY2.to_string()],
            service_id,
        }]
    }

    #[cfg(feature = "track-and-trace")]
    fn get_updated_record() -> Vec<Record> {
        vec![
            Record {
                id: None,
                start_commit_num: 0,
                end_commit_num: 1,
                record_id: "TestRecord".to_string(),
                schema: "TestGridSchema".to_string(),
                final_: false,
                owners: vec![KEY1.to_string()],
                custodians: vec![KEY2.to_string()],
                service_id: None,
            },
            Record {
                id: None,
                start_commit_num: 1,
                end_commit_num: i64::MAX,
                record_id: "TestRecord".to_string(),
                schema: "TestGridSchema".to_string(),
                final_: true,
                owners: vec![KEY2.to_string(), KEY1.to_string()],
                custodians: vec![KEY1.to_string(), KEY2.to_string()],
                service_id: None,
            },
        ]
    }

    #[cfg(feature = "track-and-trace")]
    fn get_multiple_records() -> Vec<Record> {
        vec![
            Record {
                id: None,
                start_commit_num: 0,
                end_commit_num: 1,
                record_id: "TestRecord".to_string(),
                schema: "TestGridSchema".to_string(),
                final_: false,
                owners: vec![KEY1.to_string()],
                custodians: vec![KEY2.to_string()],
                service_id: None,
            },
            Record {
                id: None,
                start_commit_num: 1,
                end_commit_num: i64::MAX,
                record_id: "TestRecord".to_string(),
                schema: "TestGridSchema".to_string(),
                final_: true,
                owners: vec![KEY2.to_string(), KEY1.to_string()],
                custodians: vec![KEY1.to_string(), KEY2.to_string()],
                service_id: None,
            },
            Record {
                id: None,
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                record_id: "TestRecord 2".to_string(),
                schema: "TestGridSchema".to_string(),
                final_: false,
                owners: vec![KEY1.to_string()],
                custodians: vec![KEY2.to_string()],
                service_id: None,
            },
        ]
    }

    fn populate_grid_schema_table(schemas: Vec<Schema>) {
        let pool = get_connection_pool();
        let store = DieselSchemaStore::new(pool.pool);
        schemas
            .into_iter()
            .for_each(|schema| store.add_schema(schema).unwrap());
    }

    fn populate_product_table(products: Vec<Product>) {
        let pool = get_connection_pool();
        let store = DieselProductStore::new(pool.pool);
        products
            .into_iter()
            .for_each(|product| store.add_product(product).unwrap());
    }

    #[cfg(feature = "track-and-trace")]
    fn get_property_for_record(service_id: Option<String>) -> Vec<Property> {
        vec![
            Property {
                id: None,
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                name: "TestProperty1".to_string(),
                record_id: "TestRecord".to_string(),
                property_definition: "property_definition_1".to_string(),
                current_page: 1,
                wrapped: false,
                service_id: service_id.clone(),
            },
            Property {
                id: None,
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                name: "TestProperty2".to_string(),
                record_id: "TestRecord".to_string(),
                property_definition: "property_definition_2".to_string(),
                current_page: 1,
                wrapped: false,
                service_id,
            },
        ]
    }

    #[cfg(feature = "track-and-trace")]
    fn get_reporter_for_property_record(service_id: Option<String>) -> Vec<Reporter> {
        vec![
            Reporter {
                id: None,
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                property_name: "TestProperty1".to_string(),
                record_id: "TestRecord".to_string(),
                public_key: KEY1.to_string(),
                authorized: true,
                reporter_index: 0,
                service_id: service_id.clone(),
            },
            Reporter {
                id: None,
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                property_name: "TestProperty2".to_string(),
                record_id: "TestRecord".to_string(),
                public_key: KEY2.to_string(),
                authorized: true,
                reporter_index: 0,
                service_id,
            },
        ]
    }

    #[cfg(feature = "track-and-trace")]
    fn get_reported_value_for_property_record(service_id: Option<String>) -> Vec<ReportedValue> {
        vec![
            ReportedValue {
                id: None,
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                property_name: "TestProperty1".to_string(),
                record_id: "TestRecord".to_string(),
                reporter_index: 0,
                timestamp: 5,
                data_type: "String".to_string(),
                bytes_value: None,
                boolean_value: None,
                number_value: None,
                string_value: Some("value_1".to_string()),
                enum_value: None,
                struct_values: None,
                lat_long_value: None,
                service_id: service_id.clone(),
            },
            ReportedValue {
                id: None,
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                property_name: "TestProperty2".to_string(),
                record_id: "TestRecord".to_string(),
                reporter_index: 0,
                timestamp: 5,
                data_type: "Boolean".to_string(),
                bytes_value: None,
                boolean_value: Some(true),
                number_value: None,
                string_value: None,
                enum_value: None,
                struct_values: None,
                lat_long_value: None,
                service_id,
            },
        ]
    }
    #[cfg(feature = "track-and-trace")]
    fn get_property(service_id: Option<String>) -> Vec<Property> {
        vec![Property {
            id: None,
            start_commit_num: 0,
            end_commit_num: i64::MAX,
            name: "TestProperty".to_string(),
            record_id: "record_01".to_string(),
            property_definition: "property_definition_1".to_string(),
            current_page: 1,
            wrapped: false,
            service_id,
        }]
    }

    #[cfg(feature = "track-and-trace")]
    fn get_reporter(service_id: Option<String>) -> Vec<Reporter> {
        vec![
            Reporter {
                id: None,
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                property_name: "TestProperty".to_string(),
                record_id: "record_01".to_string(),
                public_key: KEY1.to_string(),
                authorized: true,
                reporter_index: 0,
                service_id: service_id.clone(),
            },
            Reporter {
                id: None,
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                property_name: "TestProperty".to_string(),
                record_id: "record_01".to_string(),
                public_key: KEY2.to_string(),
                authorized: true,
                reporter_index: 1,
                service_id,
            },
        ]
    }

    #[cfg(feature = "track-and-trace")]
    fn populate_tnt_property_table(
        properties: Vec<Property>,
        reported_values: Vec<ReportedValue>,
        reporter: Vec<Reporter>,
    ) {
        let pool = get_connection_pool();
        let store = DieselTrackAndTraceStore::new(pool.pool);
        store.add_properties(properties).unwrap();
        store.add_reported_values(reported_values).unwrap();
        store.add_reporters(reporter).unwrap();
    }

    fn get_property_definition(service_id: Option<String>) -> Vec<PropertyDefinition> {
        vec![
            PropertyDefinition {
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                name: "Definition Name".to_string(),
                schema_name: "TestGridSchema".to_string(),
                data_type: "Lightbulb".to_string(),
                required: false,
                description: "Definition Description".to_string(),
                number_exponent: 0,
                enum_options: vec![],
                struct_properties: vec![],
                service_id: service_id.clone(),
            },
            PropertyDefinition {
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                name: "Other Definition Name".to_string(),
                schema_name: "TestGridSchema".to_string(),
                data_type: "New Lightbulb".to_string(),
                required: false,
                description: "Definition Description".to_string(),
                number_exponent: 0,
                enum_options: vec![],
                struct_properties: vec![],
                service_id: service_id,
            },
        ]
    }

    #[cfg(feature = "track-and-trace")]
    fn get_grid_schema_for_struct_record(service_id: Option<String>) -> Vec<Schema> {
        vec![Schema {
            start_commit_num: 0,
            end_commit_num: i64::MAX,
            name: "TestGridSchema".to_string(),
            properties: get_grid_property_definition_struct_for_record(service_id.clone()),
            description: "Example test grid schema".to_string(),
            owner: "phillips001".to_string(),
            service_id,
        }]
    }

    #[cfg(feature = "track-and-trace")]
    fn get_grid_property_definition_struct_for_record(
        service_id: Option<String>,
    ) -> Vec<PropertyDefinition> {
        vec![PropertyDefinition {
            start_commit_num: 0,
            end_commit_num: i64::MAX,
            name: "TestProperty".to_string(),
            schema_name: "TestGridSchema".to_string(),
            data_type: "Struct".to_string(),
            required: false,
            description: "Definition Description".to_string(),
            number_exponent: 0,
            enum_options: vec![],
            struct_properties: vec![],
            service_id,
        }]
    }

    #[cfg(feature = "track-and-trace")]
    fn get_grid_schema_for_record(service_id: Option<String>) -> Vec<Schema> {
        vec![Schema {
            start_commit_num: 0,
            end_commit_num: i64::MAX,
            name: "TestGridSchema".to_string(),
            properties: get_grid_property_definition_for_record(service_id.clone()),
            description: "Example test grid schema".to_string(),
            owner: "phillips001".to_string(),
            service_id,
        }]
    }

    #[cfg(feature = "track-and-trace")]
    fn get_grid_property_definition_for_record(
        service_id: Option<String>,
    ) -> Vec<PropertyDefinition> {
        vec![
            PropertyDefinition {
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                name: "TestProperty1".to_string(),
                schema_name: "TestGridSchema".to_string(),
                data_type: "String".to_string(),
                required: false,
                description: "Definition Description".to_string(),
                number_exponent: 0,
                enum_options: vec![],
                struct_properties: vec![],
                service_id: service_id.clone(),
            },
            PropertyDefinition {
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                name: "TestProperty2".to_string(),
                schema_name: "TestGridSchema".to_string(),
                data_type: "Boolean".to_string(),
                required: false,
                description: "Definition Description".to_string(),
                number_exponent: 0,
                enum_options: vec![],
                struct_properties: vec![],
                service_id,
            },
        ]
    }

    fn get_product_property_value(service_id: Option<String>) -> Vec<PropertyValue> {
        vec![
            PropertyValue {
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                product_id: "041205707820".to_string(),
                product_address: "test_address".to_string(),
                property_name: "Test Grid Product".to_string(),
                data_type: "Lightbulb".to_string(),
                bytes_value: None,
                boolean_value: None,
                number_value: Some(0),
                string_value: None,
                enum_value: None,
                struct_values: vec![],
                lat_long_value: None,
                service_id: service_id.clone(),
            },
            PropertyValue {
                start_commit_num: 0,
                end_commit_num: i64::MAX,
                product_id: "041205707820".to_string(),
                product_address: "test_address".to_string(),
                property_name: "Test Grid Product".to_string(),
                data_type: "Lightbulb".to_string(),
                bytes_value: None,
                boolean_value: None,
                number_value: Some(0),
                string_value: None,
                enum_value: None,
                struct_values: vec![],
                lat_long_value: None,
                service_id,
            },
        ]
    }

    #[cfg(feature = "track-and-trace")]
    fn populate_associated_agent_table(associated_agents: Vec<AssociatedAgent>) {
        let pool = get_connection_pool();
        let store = DieselTrackAndTraceStore::new(pool.pool);
        store.add_associated_agents(associated_agents).unwrap();
    }

    #[cfg(feature = "track-and-trace")]
    fn populate_proposal_table(proposals: Vec<Proposal>) {
        let pool = get_connection_pool();
        let store = DieselTrackAndTraceStore::new(pool.pool);
        store.add_proposals(proposals).unwrap();
    }

    #[cfg(feature = "track-and-trace")]
    fn populate_record_table(records: Vec<Record>) {
        let pool = get_connection_pool();
        let store = DieselTrackAndTraceStore::new(pool.pool);
        store.add_records(records).unwrap();
    }

    #[cfg(not(feature = "test-postgres"))]
    fn run_migrations(database_url: &str) {
        let connection = SqliteConnection::establish(database_url)
            .expect("Failed to stablish connection with database");
        run_sqlite_migrations(&connection).expect("Migrations failed");
    }

    #[cfg(feature = "test-postgres")]
    fn run_migrations(database_url: &str) {
        let connection = PgConnection::establish(database_url)
            .expect("Failed to stablish connection with database");
        run_postgres_migrations(&connection).expect("Migrations failed");
    }

    #[cfg(not(feature = "test-postgres"))]
    fn clear_database() {
        let pool = get_connection_pool();
        clear_sqlite_database(&pool.get().unwrap()).unwrap();
    }

    #[cfg(feature = "test-postgres")]
    fn clear_database() {
        let pool = get_connection_pool();
        clear_postgres_database(&pool.get().unwrap()).unwrap();
    }

    #[cfg(feature = "track-and-trace")]
    fn get_reported_value(service_id: Option<String>) -> Vec<ReportedValue> {
        vec![
            ReportedValue {
                id: None,
                start_commit_num: 0,
                end_commit_num: 2,
                property_name: "TestProperty".to_string(),
                record_id: "record_01".to_string(),
                reporter_index: 1,
                timestamp: 3,
                data_type: "Struct".to_string(),
                bytes_value: None,
                boolean_value: None,
                number_value: None,
                string_value: None,
                enum_value: None,
                struct_values: Some(vec![
                    ReportedValue {
                        id: None,
                        start_commit_num: 0,
                        end_commit_num: 2,
                        property_name: "TestProperty_StructProperty".to_string(),
                        record_id: "record_01".to_string(),
                        reporter_index: 0,
                        timestamp: 3,
                        data_type: "Struct".to_string(),
                        bytes_value: None,
                        boolean_value: None,
                        number_value: None,
                        string_value: None,
                        enum_value: None,
                        struct_values: Some(vec![
                            ReportedValue {
                                id: None,
                                start_commit_num: 0,
                                end_commit_num: 2,
                                property_name: "TestProperty_StructProperty_StringProperty"
                                    .to_string(),
                                record_id: "record_01".to_string(),
                                reporter_index: 0,
                                timestamp: 3,
                                data_type: "String".to_string(),
                                bytes_value: None,
                                boolean_value: None,
                                number_value: None,
                                string_value: Some("value_1".to_string()),
                                enum_value: None,
                                struct_values: None,
                                lat_long_value: None,
                                service_id: service_id.clone(),
                            },
                            ReportedValue {
                                id: None,
                                start_commit_num: 0,
                                end_commit_num: 2,
                                property_name: "TestProperty_StructProperty_BoolProperty"
                                    .to_string(),
                                record_id: "record_01".to_string(),
                                reporter_index: 1,
                                timestamp: 3,
                                data_type: "Boolean".to_string(),
                                bytes_value: None,
                                boolean_value: Some(false),
                                number_value: None,
                                string_value: None,
                                enum_value: None,
                                struct_values: None,
                                lat_long_value: None,
                                service_id: service_id.clone(),
                            },
                        ]),
                        lat_long_value: None,
                        service_id: service_id.clone(),
                    },
                    ReportedValue {
                        id: None,
                        start_commit_num: 0,
                        end_commit_num: 2,
                        property_name: "TestProperty_LatLongProperty".to_string(),
                        record_id: "record_01".to_string(),
                        reporter_index: 1,
                        timestamp: 3,
                        data_type: "LatLong".to_string(),
                        bytes_value: None,
                        boolean_value: None,
                        number_value: None,
                        string_value: None,
                        enum_value: None,
                        struct_values: None,
                        lat_long_value: Some(LatLongValue(1, 1)),
                        service_id: service_id.clone(),
                    },
                    ReportedValue {
                        id: None,
                        start_commit_num: 0,
                        end_commit_num: 2,
                        property_name: "TestProperty_NumberProperty".to_string(),
                        record_id: "record_01".to_string(),
                        reporter_index: 1,
                        timestamp: 3,
                        data_type: "Number".to_string(),
                        bytes_value: None,
                        boolean_value: None,
                        number_value: Some(1),
                        string_value: None,
                        enum_value: None,
                        struct_values: None,
                        lat_long_value: None,
                        service_id: service_id.clone(),
                    },
                    ReportedValue {
                        id: None,
                        start_commit_num: 0,
                        end_commit_num: 2,
                        property_name: "TestProperty_EnumProperty".to_string(),
                        record_id: "record_01".to_string(),
                        reporter_index: 0,
                        timestamp: 3,
                        data_type: "Enum".to_string(),
                        bytes_value: None,
                        boolean_value: None,
                        number_value: None,
                        string_value: None,
                        enum_value: Some(1),
                        struct_values: None,
                        lat_long_value: None,
                        service_id: service_id.clone(),
                    },
                    ReportedValue {
                        id: None,
                        start_commit_num: 0,
                        end_commit_num: 2,
                        property_name: "TestProperty_BytesProperty".to_string(),
                        record_id: "record_01".to_string(),
                        reporter_index: 0,
                        timestamp: 3,
                        data_type: "Bytes".to_string(),
                        bytes_value: Some(vec![0x01, 0x02, 0x03, 0x04]),
                        boolean_value: None,
                        number_value: None,
                        string_value: None,
                        enum_value: None,
                        struct_values: None,
                        lat_long_value: None,
                        service_id: service_id.clone(),
                    },
                ]),
                lat_long_value: None,
                service_id: service_id.clone(),
            },
            ReportedValue {
                id: None,
                start_commit_num: 2,
                end_commit_num: i64::MAX,
                property_name: "TestProperty".to_string(),
                record_id: "record_01".to_string(),
                reporter_index: 0,
                timestamp: 5,
                data_type: "Struct".to_string(),
                bytes_value: None,
                boolean_value: None,
                number_value: None,
                string_value: None,
                enum_value: None,
                struct_values: Some(vec![
                    ReportedValue {
                        id: None,
                        start_commit_num: 2,
                        end_commit_num: i64::MAX,
                        property_name: "TestProperty_StructProperty".to_string(),
                        record_id: "record_01".to_string(),
                        reporter_index: 1,
                        timestamp: 5,
                        data_type: "Struct".to_string(),
                        bytes_value: None,
                        boolean_value: None,
                        number_value: None,
                        string_value: None,
                        enum_value: None,
                        struct_values: Some(vec![
                            ReportedValue {
                                id: None,
                                start_commit_num: 2,
                                end_commit_num: i64::MAX,
                                property_name: "TestProperty_StructProperty_StringProperty"
                                    .to_string(),
                                record_id: "record_01".to_string(),
                                reporter_index: 1,
                                timestamp: 5,
                                data_type: "String".to_string(),
                                bytes_value: None,
                                boolean_value: None,
                                number_value: None,
                                string_value: Some("value_updated".to_string()),
                                enum_value: None,
                                struct_values: None,
                                lat_long_value: None,
                                service_id: service_id.clone(),
                            },
                            ReportedValue {
                                id: None,
                                start_commit_num: 2,
                                end_commit_num: i64::MAX,
                                property_name: "TestProperty_StructProperty_BoolProperty"
                                    .to_string(),
                                record_id: "record_01".to_string(),
                                reporter_index: 0,
                                timestamp: 5,
                                data_type: "Boolean".to_string(),
                                bytes_value: None,
                                boolean_value: Some(true),
                                number_value: None,
                                string_value: None,
                                enum_value: None,
                                struct_values: None,
                                lat_long_value: None,
                                service_id: service_id.clone(),
                            },
                        ]),
                        lat_long_value: None,
                        service_id: service_id.clone(),
                    },
                    ReportedValue {
                        id: None,
                        start_commit_num: 2,
                        end_commit_num: i64::MAX,
                        property_name: "TestProperty_LatLongProperty".to_string(),
                        record_id: "record_01".to_string(),
                        reporter_index: 0,
                        timestamp: 5,
                        data_type: "LatLong".to_string(),
                        bytes_value: None,
                        boolean_value: None,
                        number_value: None,
                        string_value: None,
                        enum_value: None,
                        struct_values: None,
                        lat_long_value: Some(LatLongValue(2, 2)),
                        service_id: service_id.clone(),
                    },
                    ReportedValue {
                        id: None,
                        start_commit_num: 2,
                        end_commit_num: i64::MAX,
                        property_name: "TestProperty_NumberProperty".to_string(),
                        record_id: "record_01".to_string(),
                        reporter_index: 0,
                        timestamp: 5,
                        data_type: "Number".to_string(),
                        bytes_value: None,
                        boolean_value: None,
                        number_value: Some(2),
                        string_value: None,
                        enum_value: None,
                        struct_values: None,
                        lat_long_value: None,
                        service_id: service_id.clone(),
                    },
                    ReportedValue {
                        id: None,
                        start_commit_num: 2,
                        end_commit_num: i64::MAX,
                        property_name: "TestProperty_EnumProperty".to_string(),
                        record_id: "record_01".to_string(),
                        reporter_index: 1,
                        timestamp: 5,
                        data_type: "Enum".to_string(),
                        bytes_value: None,
                        boolean_value: None,
                        number_value: None,
                        string_value: None,
                        enum_value: Some(2),
                        struct_values: None,
                        lat_long_value: None,
                        service_id: service_id.clone(),
                    },
                    ReportedValue {
                        id: None,
                        start_commit_num: 2,
                        end_commit_num: i64::MAX,
                        property_name: "TestProperty_BytesProperty".to_string(),
                        record_id: "record_01".to_string(),
                        reporter_index: 1,
                        timestamp: 5,
                        data_type: "Bytes".to_string(),
                        bytes_value: Some(vec![0x05, 0x06, 0x07, 0x08]),
                        boolean_value: None,
                        number_value: None,
                        string_value: None,
                        enum_value: None,
                        struct_values: None,
                        lat_long_value: None,
                        service_id: service_id.clone(),
                    },
                ]),
                lat_long_value: None,
                service_id,
            },
        ]
    }
}
