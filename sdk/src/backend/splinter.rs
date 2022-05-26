// Copyright 2018-2021 Cargill Incorporated
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

use std::pin::Pin;
use std::str::FromStr;

use futures::prelude::*;
use protobuf::Message;
use reqwest::{Client, StatusCode};
use sawtooth_sdk::messages::batch::Batch;
use serde::Deserialize;
use serde_json;

use super::{
    BackendClient, BackendClientError, BatchStatus, BatchStatusLink, BatchStatuses,
    InvalidTransaction, SubmitBatches,
};

macro_rules! try_fut {
    ($try_expr:expr) => {
        match $try_expr {
            Ok(res) => res,
            Err(err) => return futures::future::err(err).boxed(),
        }
    };
}

#[derive(Deserialize)]
pub struct SplinterErrorResponse {
    pub message: String,
}

#[derive(Clone)]
pub struct SplinterBackendClient {
    node_url: String,
    authorization: String,
}

impl SplinterBackendClient {
    /// Constructs a new splinter BackendClient instance, using the given url for the node's REST
    /// API.
    pub fn new(node_url: String, authorization: String) -> Self {
        Self {
            node_url,
            authorization,
        }
    }
}

type BatchStatusResponse =
    Pin<Box<dyn Future<Output = Result<Vec<BatchStatus>, BackendClientError>> + Send>>;

impl BackendClient for SplinterBackendClient {
    fn submit_batches(
        &self,
        msg: SubmitBatches,
    ) -> Pin<Box<dyn Future<Output = Result<BatchStatusLink, BackendClientError>> + Send>> {
        let service_arg = try_fut!(msg.service_id.ok_or_else(|| {
            BackendClientError::BadRequestError("A service id must be provided".into())
        }));

        let service_info = try_fut!(SplinterService::from_str(&service_arg));

        let url = format!(
            "{}/scabbard/{}/{}/batches",
            self.node_url, service_info.circuit_id, service_info.service_id
        );

        let batch_list_bytes = try_fut!(msg.batch_list.write_to_bytes().map_err(|err| {
            BackendClientError::BadRequestError(format!("Malformed batch list: {}", err))
        }));

        let batch_query = msg
            .batch_list
            .get_batches()
            .iter()
            .map(Batch::get_header_signature)
            .collect::<Vec<_>>()
            .join(",");
        let mut response_url = msg.response_url;
        response_url.set_query(Some(&format!("id={}", batch_query)));
        let link = response_url.to_string();

        reqwest::Client::new()
            .post(&url)
            .header("GridProtocolVersion", "1")
            .header("Content-Type", "octet-stream")
            .header("Authorization", &self.authorization.to_string())
            .body(batch_list_bytes)
            .send()
            .then(|res| {
                future::ready(match res {
                    Ok(_) => Ok(BatchStatusLink { link }),
                    Err(err) => Err(BackendClientError::InternalError(format!(
                        "Unable to submit batch: {}",
                        err
                    ))),
                })
            })
            .boxed()
    }

    fn batch_status(&self, msg: BatchStatuses) -> BatchStatusResponse {
        let service_arg = try_fut!(msg.service_id.ok_or_else(|| {
            BackendClientError::BadRequestError("A service id must be provided".into())
        }));

        let service_info = try_fut!(SplinterService::from_str(&service_arg));

        // {base_url}/scabbard/{circuit_id}/{service_id}/batch_statuses?[wait={time}&]ids={batch_ids}
        let mut url = self.node_url.clone();
        url.push_str("/scabbard/");
        url.push_str(&service_info.circuit_id);
        url.push('/');
        url.push_str(&service_info.service_id);
        url.push_str("/batch_statuses?");

        if let Some(wait_time) = msg.wait {
            url.push_str("wait=");
            url.push_str(&wait_time.to_string());
            url.push('&');
        }

        url.push_str("ids=");
        url.push_str(&msg.batch_ids.join(","));

        Client::new()
            .get(&url)
            .header("GridProtocolVersion", "1")
            .header("Authorization", &self.authorization.to_string())
            .send()
            .then(|res| match res {
                Ok(res) => future::join(future::ok(res.status()), res.bytes()).boxed(),
                Err(err) => future::join(
                    future::err(BackendClientError::InternalError(format!(
                        "Unable to retrieve batch statuses: {}",
                        err
                    ))),
                    future::err(err),
                )
                .boxed(),
            })
            .map(|(status, bytes)| {
                let bytes = bytes.map_err(|err| {
                    BackendClientError::InternalError(format!(
                        "Error reading batch status bytes: {err}",
                    ))
                })?;

                match status? {
                    StatusCode::OK => serde_json::from_slice(&bytes)
                        .map(|stats: Vec<SplinterBatchStatus>| {
                            stats.into_iter().map(|status| status.into()).collect()
                        })
                        .map_err(|err| {
                            BackendClientError::InternalError(format!(
                                "Encountered error \"{err}\" while deserializing \
                                    Splinter batch status response: {resp}",
                                resp = String::from_utf8_lossy(&bytes)
                            ))
                        }),
                    status => {
                        let error: SplinterErrorResponse =
                            serde_json::from_slice(&bytes).map_err(|err| {
                                BackendClientError::InternalError(format!(
                                    "Encountered error \"{err}\" while deserializing \
                                    Splinter batch status error response: {resp}",
                                    resp = String::from_utf8_lossy(&bytes)
                                ))
                            })?;

                        Err(BackendClientError::BadRequestError(format!(
                            "Splinter responded with {status}: {message}",
                            message = error.message
                        )))
                    }
                }
            })
            .boxed()
    }

    fn clone_box(&self) -> Box<dyn BackendClient> {
        Box::new(self.clone())
    }
}

#[derive(Deserialize, Debug)]
struct SplinterBatchStatus {
    id: String,
    status: Status,
}

#[derive(Deserialize, Debug)]
struct Status {
    #[serde(rename(deserialize = "statusType"))]
    status_type: String,
    message: Vec<ErrorMessage>,
}

#[derive(Deserialize, Debug)]
struct ErrorMessage {
    transaction_id: String,
    error_message: Option<String>,
    error_data: Option<Vec<u8>>,
}

impl From<SplinterBatchStatus> for BatchStatus {
    fn from(batch_status: SplinterBatchStatus) -> Self {
        Self {
            id: batch_status.id,
            status: batch_status.status.status_type,
            invalid_transactions: batch_status
                .status
                .message
                .into_iter()
                .filter(|message| message.error_message.is_some() && message.error_data.is_some())
                .map(|message| InvalidTransaction {
                    id: message.transaction_id,
                    message: message.error_message.unwrap(),
                    extended_data: base64::encode(&message.error_data.unwrap()),
                })
                .collect(),
        }
    }
}

struct SplinterService {
    circuit_id: String,
    service_id: String,
}

impl FromStr for SplinterService {
    type Err = BackendClientError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split("::");
        let circuit_id: String = parts
            .next()
            .ok_or_else(|| {
                BackendClientError::BadRequestError("Empty service_id parameter provided".into())
            })?
            .into();
        let service_id: String = parts
            .next()
            .ok_or_else(|| {
                BackendClientError::BadRequestError(
                    "Must provide a fully-qualified service_id: <circuit_id>::<service_id>".into(),
                )
            })?
            .into();

        Ok(Self {
            circuit_id,
            service_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{self, Matcher, Mock};
    use pretty_assertions::assert_eq;

    const TEST_CIRCUIT_ID: &str = "z7499-QGFd3";
    const TEST_SERVICE_ID: &str = "gsAA";
    const TEST_AUTHORIZATION: &str = "foo";
    const TEST_BATCH_ID: &str = "one";
    const TEST_SUCCESS_STATUS_RESPONSE: &str = r#"[
    {
        "id": "one",
        "status": {
            "statusType": "sampleStatusType",
            "message": []
        }
    }
]"#;

    fn setup_basic_batch_statuses_request() -> (Mock, BatchStatusResponse) {
        let mock_endpoint = mockito::mock(
            "GET",
            Matcher::Exact(format!(
                "/scabbard/{TEST_CIRCUIT_ID}/\
                {TEST_SERVICE_ID}/batch_statuses?ids={TEST_BATCH_ID}"
            )),
        );

        let response =
            SplinterBackendClient::new(mockito::server_url(), TEST_AUTHORIZATION.to_string())
                .batch_status(BatchStatuses {
                    batch_ids: vec![TEST_BATCH_ID.to_string()],
                    wait: None,
                    service_id: Some(format!("{TEST_CIRCUIT_ID}::{TEST_SERVICE_ID}")),
                });

        (mock_endpoint, response)
    }

    #[tokio::test]
    async fn batch_statuses_returns_useful_message_on_404() {
        let (endpoint, response) = setup_basic_batch_statuses_request();

        let endpoint = endpoint
            .with_status(404)
            .with_body(format!(
                "{{\"message\":\"scabbard service {TEST_SERVICE_ID} \
                on circuit {TEST_CIRCUIT_ID} not found\"}}"
            ))
            .create();

        let result = response.await;

        endpoint.assert();
        assert_eq!(
            format!("{:?}", result),
            "Err(BadRequestError(\"Splinter \
            responded with 404 Not Found: scabbard service \
            gsAA on circuit z7499-QGFd3 not found\"))"
        );
    }

    #[tokio::test]
    async fn batch_statuses_returns_correctly_on_200_success() {
        let (endpoint, response) = setup_basic_batch_statuses_request();

        let endpoint = endpoint
            .with_status(200)
            .with_body(TEST_SUCCESS_STATUS_RESPONSE)
            .create();

        let result = response.await;

        endpoint.assert();
        assert_eq!(
            format!("{:?}", result),
            "Ok([BatchStatus { id: \"one\", invalid_transactions: [], status: \
            \"sampleStatusType\" }])"
        );
    }

    #[tokio::test]
    async fn batch_statuses_returns_useful_message_on_200_deserialize_error() {
        let (endpoint, response) = setup_basic_batch_statuses_request();

        let endpoint = endpoint.with_status(200).with_body("bad json").create();

        let result = response.await;

        endpoint.assert();
        assert_eq!(
            format!("{:?}", result),
            "Err(InternalError(\"Encountered \
            error \\\"expected value at line 1 column 1\\\" while deserializing \
            Splinter batch status response: bad json\"))"
        );
    }

    #[tokio::test]
    async fn batch_statuses_returns_useful_message_on_503_deserialize_error() {
        let (endpoint, response) = setup_basic_batch_statuses_request();

        let endpoint = endpoint.with_status(503).with_body("bad json").create();

        let result = response.await;

        endpoint.assert();
        assert_eq!(
            format!("{:?}", result),
            "Err(InternalError(\"Encountered error \\\"expected value at line 1 column 1\\\" \
            while deserializing Splinter batch status error response: bad json\"))"
        );
    }
}
