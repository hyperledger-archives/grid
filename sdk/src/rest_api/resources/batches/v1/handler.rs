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

use std::sync::Arc;

use sawtooth_sdk::messages::batch::BatchList;
use url::Url;

use crate::{
    rest_api::resources::error::ErrorResponse,
    submitter::{
        BatchStatuses, BatchSubmitter, BatchSubmitterError, SubmitBatches, DEFAULT_TIME_OUT,
    },
};

use super::payloads::{BatchStatus, BatchStatusLink, BatchStatusResponse};

pub async fn submit_batches(
    response_url: Url,
    submitter: Arc<dyn BatchSubmitter>,
    bytes: &[u8],
    service_id: Option<String>,
) -> Result<BatchStatusLink, ErrorResponse> {
    let batch_list: BatchList = match protobuf::Message::parse_from_bytes(bytes) {
        Ok(batch_list) => batch_list,
        Err(err) => {
            return Err(ErrorResponse::new(
                400,
                &format!("Protobuf message was badly formatted. {}", err.to_string()),
            ));
        }
    };

    submitter
        .submit_batches(SubmitBatches {
            batch_list,
            response_url,
            service_id,
        })
        .await
        .map_err(|err| match err {
            BatchSubmitterError::BadRequestError(ref msg) => ErrorResponse::new(400, msg),
            BatchSubmitterError::ConnectionError(ref msg) => ErrorResponse::new(503, msg),
            BatchSubmitterError::InternalError(ref msg) => ErrorResponse::new(500, msg),
            BatchSubmitterError::ResourceTemporarilyUnavailableError(ref msg) => {
                ErrorResponse::new(503, msg)
            }
        })
        .map(BatchStatusLink::from)
}

pub async fn fetch_batch_statuses(
    response_url: String,
    submitter: Arc<dyn BatchSubmitter>,
    ids: String,
    wait: Option<String>,
    service_id: Option<String>,
) -> Result<BatchStatusResponse, ErrorResponse> {
    let batch_ids = ids.split(',').map(ToString::to_string).collect();

    // Max wait time allowed is 95% of network's configured timeout
    let max_wait_time = (DEFAULT_TIME_OUT * 95) / 100;

    let wait = match wait {
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
                        return Err(ErrorResponse::new(
                            400,
                            &format!(
                                "Query wait has invalid value {}. \
                             It should set to false or a time in seconds to wait for the commit",
                                wait_time
                            ),
                        ));
                    }
                }
            }
        }

        None => Some(max_wait_time),
    };

    submitter
        .batch_status(BatchStatuses {
            batch_ids,
            wait,
            service_id,
        })
        .await
        .map_err(|err| match err {
            BatchSubmitterError::BadRequestError(ref msg) => ErrorResponse::new(400, msg),
            BatchSubmitterError::ConnectionError(ref msg) => ErrorResponse::new(503, msg),
            BatchSubmitterError::InternalError(ref msg) => ErrorResponse::new(500, msg),
            BatchSubmitterError::ResourceTemporarilyUnavailableError(ref msg) => {
                ErrorResponse::new(500, msg)
            }
        })
        .map(|batches| BatchStatusResponse {
            data: batches.into_iter().map(BatchStatus::from).collect(),
            link: response_url,
        })
}
