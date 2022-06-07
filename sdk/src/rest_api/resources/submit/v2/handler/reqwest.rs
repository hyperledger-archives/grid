// Copyright 2022 Cargill Incorporated
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

use futures::{prelude::future, FutureExt, TryFutureExt};
use reqwest::{Client, StatusCode};
use url::Url;

use crate::rest_api::resources::error::ErrorResponse;
use crate::rest_api::resources::submit::v2::handler::{
    BatchIdList, BatchSubmissionHandler, SerializedSubmitBatchRequest, SubmitBatchErrorResponse,
    SubmitBatchResponse,
};

#[derive(Clone)]
pub struct ReqwestBatchSubmissionHandler {
    submit_url: Url,
}

impl ReqwestBatchSubmissionHandler {
    pub fn new(submit_url: Url) -> Self {
        Self { submit_url }
    }
}

impl BatchSubmissionHandler for ReqwestBatchSubmissionHandler {
    fn submit_batches(self, submit_request: SerializedSubmitBatchRequest) -> SubmitBatchResponse {
        let submit_url = self.submit_url;

        let request = Client::new().post(submit_url).body(submit_request.body);

        request
            .send()
            .map_err(|err| {
                SubmitBatchErrorResponse::new(502, &format!("Failed to submit batch: {err}"))
            })
            .then(|client_resp| match client_resp {
                Ok(resp) => future::join(
                    future::ok(resp.status()),
                    resp.bytes().map_err(|err| {
                        SubmitBatchErrorResponse::new(
                            502,
                            &format!("Failed to retrieve response: {err}"),
                        )
                    }),
                )
                .boxed(),
                Err(err) => {
                    let error = SubmitBatchErrorResponse::new(502, "Failed to retrieve response");
                    future::join(future::err(error), future::err(err)).boxed()
                }
            })
            .map(|(status_res, body_res)| {
                let bytes = body_res?.to_vec();
                match status_res? {
                    StatusCode::OK => {
                        serde_json::from_slice::<BatchIdList>(&bytes).map_err(|err| {
                            SubmitBatchErrorResponse::new(
                                502,
                                &format!("Got Ok, but received malformed response: {err}"),
                            )
                        })
                    }
                    status => {
                        let error: ErrorResponse =
                            serde_json::from_slice(&bytes).map_err(|err| {
                                SubmitBatchErrorResponse::new(
                                    502,
                                    &format!(
                                        "Received {} from upstream, but body was malformed: {err}",
                                        status.as_u16()
                                    ),
                                )
                            })?;
                        Err(SubmitBatchErrorResponse::new(
                            error.status_code(),
                            &format!("Failed to submit batches: {}", error.message()),
                        ))
                    }
                }
            })
            .boxed()
    }

    fn cloned_box(&self) -> Box<dyn BatchSubmissionHandler> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use mockito::{self, mock};

    use crate::rest_api::resources::error::ErrorResponse;
    use crate::rest_api::resources::submit::v2::payloads::batch::{
        BatchIdentifier, SubmitBatchRequest, TrackingBatchResource,
    };

    const TEST_CIRCUIT_ID: &str = "z7499-QGFd3";
    const TEST_SERVICE_ID: &str = "gsAA";
    const TEST_BATCH_ID: &str = "one";
    const TEST_DATA_CHANGE_ID: &str = "data_change_id::two";
    const TEST_SIGNER_PUB_KEY: &str = "test_user";

    #[actix_rt::test]
    /// Validate the `ReqwestBatchSubmissionHandler` `submit_batches` method is able to return
    /// an error from the backend, preserving the original error
    async fn reqwest_batch_submission_handler_error() {
        let error = ErrorResponse::new(408, "Request timed out");
        let error_bytes = serde_json::to_vec(&error).expect("Unable to serialize error response");
        let expected_error_response = SubmitBatchErrorResponse::new(
            408,
            &format!("Failed to submit batches: {}", error.message()),
        );

        let endpoint = mock("POST", "/batches")
            .with_status(408)
            .with_header("content-type", "application/octet-stream")
            .with_body(error_bytes)
            .create();

        let response = setup_future_response().await;

        endpoint.assert();

        match response {
            Ok(_) => panic!("Endpoint should have failed"),
            Err(err) => assert_eq!(err, expected_error_response),
        }
    }

    #[actix_rt::test]
    /// Validate the `ReqwestBatchSubmissionHandler` `submit_batches` method is able to return
    /// successfully with the expected response
    async fn reqwest_batch_submission_handler_success() {
        let expected_success_response = BatchIdList {
            batch_identifiers: vec![get_test_batch_identity()],
        };
        let response_body = serde_json::to_vec(&expected_success_response)
            .expect("Unable to serialize batch ID list");

        let endpoint = mock("POST", "/batches")
            .with_status(200)
            .with_header("content-type", "application/octet-stream")
            .with_body(response_body)
            .create();

        let response = setup_future_response().await;

        endpoint.assert();

        match response {
            Ok(batch_id_list) => assert_eq!(batch_id_list, expected_success_response),
            Err(err) => panic!("Failed to submit batches, received response: {:?}", err),
        }
    }

    fn make_submit_batch_request() -> SerializedSubmitBatchRequest {
        let batch = TrackingBatchResource {
            signed_batch: vec![],
            batch_identity: get_test_batch_identity(),
            signer_public_key: TEST_SIGNER_PUB_KEY.to_string(),
        };
        let batch_request = SubmitBatchRequest {
            batches: vec![batch],
        };

        let batch_bytes = serde_json::to_vec(&batch_request).expect("Unable to serialize batch");

        SerializedSubmitBatchRequest { body: batch_bytes }
    }

    fn setup_future_response() -> SubmitBatchResponse {
        let mut url =
            Url::parse(&mockito::server_url()).expect("Unable to parse mockito server URL");
        url = url
            .join("/batches")
            .expect("Unable to create `/batches` URL");
        ReqwestBatchSubmissionHandler::new(url).submit_batches(make_submit_batch_request())
    }

    fn get_test_batch_identity() -> BatchIdentifier {
        BatchIdentifier {
            dlt_batch_id: TEST_BATCH_ID.to_string(),
            data_change_id: Some(TEST_DATA_CHANGE_ID.to_string()),
            service_id: Some(TEST_SERVICE_ID.to_string()),
        }
    }
}
