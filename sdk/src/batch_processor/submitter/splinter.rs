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

use std::str::FromStr;

use protobuf::Message;

use crate::batch_processor::submitter::{
    BatchStatus, BatchStatuses, BatchSubmitter, InvalidTransaction, SubmitBatches,
};

use super::error::BatchSubmitterError;

#[derive(Clone)]
pub struct SplinterBatchSubmitter {
    node_url: String,
}

impl SplinterBatchSubmitter {
    /// Constructs a new splinter BatchSubmitter instance, using the given url for the node's REST
    /// API.
    pub fn new(node_url: &str) -> Self {
        Self {
            node_url: node_url.to_string(),
        }
    }
}

impl BatchSubmitter for SplinterBatchSubmitter {
    fn submit_batches(&self, msg: SubmitBatches) -> Result<(), BatchSubmitterError> {
        let service_arg = msg.service_id.ok_or_else(|| {
            BatchSubmitterError::BadRequestError("A service id must be provided".into())
        })?;

        let service_info = SplinterService::from_str(&service_arg)?;

        let url = format!(
            "{}/scabbard/{}/{}/batches",
            self.node_url, service_info.circuit_id, service_info.service_id
        );

        let batch_list_bytes = msg.batch_list.write_to_bytes().map_err(|err| {
            BatchSubmitterError::BadRequestError(format!("Malformed batch list: {}", err))
        })?;

        let client = reqwest::blocking::Client::new();
        let res = client
            .post(&url)
            .header("GridProtocolVersion", "1")
            .header("Content-Type", "octet-stream")
            .body(batch_list_bytes)
            .send()
            .map_err(|err| BatchSubmitterError::InternalError(format!("{}", err)))?;

        match res.status().as_u16() {
            200 | 202 => Ok(()),
            400 => {
                let error_message = res
                    .json::<SubmitErrorMessage>()
                    .map_err(|err| BatchSubmitterError::InternalError(format!("{}", err)))?;
                Err(BatchSubmitterError::BadRequestError(error_message.message))
            }
            404 => {
                let error_message = res
                    .json::<SubmitErrorMessage>()
                    .map_err(|err| BatchSubmitterError::InternalError(format!("{}", err)))?;
                Err(BatchSubmitterError::NotFound(error_message.message))
            }
            503 => {
                let error_message = res
                    .json::<SubmitErrorMessage>()
                    .map_err(|err| BatchSubmitterError::InternalError(format!("{}", err)))?;
                Err(BatchSubmitterError::ResourceTemporarilyUnavailableError(
                    error_message.message,
                ))
            }
            _ => {
                let error_message = res
                    .json::<SubmitErrorMessage>()
                    .map_err(|err| BatchSubmitterError::InternalError(format!("{}", err)))?;
                Err(BatchSubmitterError::InternalError(error_message.message))
            }
        }
    }

    fn batch_status(&self, msg: BatchStatuses) -> Result<Vec<BatchStatus>, BatchSubmitterError> {
        let service_arg = msg.service_id.ok_or_else(|| {
            BatchSubmitterError::BadRequestError("A service id must be provided".into())
        })?;

        let service_info = SplinterService::from_str(&service_arg)?;

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

        let client = reqwest::blocking::Client::new();
        let res = client.get(&url).send().map_err(|err| {
            BatchSubmitterError::InternalError(format!(
                "Unable to retrieve batch statuses: {}",
                err
            ))
        })?;

        match res.status().as_u16() {
            200 => {
                let state = res
                    .json::<Vec<SplinterBatchStatus>>()
                    .map_err(|err| BatchSubmitterError::InternalError(format!("{}", err)))?;
                Ok(state.into_iter().map(|status| status.into()).collect())
            }
            400 => {
                let error_message = res
                    .json::<SubmitErrorMessage>()
                    .map_err(|err| BatchSubmitterError::InternalError(format!("{}", err)))?;
                Err(BatchSubmitterError::BadRequestError(error_message.message))
            }
            503 => {
                let error_message = res
                    .json::<SubmitErrorMessage>()
                    .map_err(|err| BatchSubmitterError::InternalError(format!("{}", err)))?;
                Err(BatchSubmitterError::ResourceTemporarilyUnavailableError(
                    error_message.message,
                ))
            }
            _ => {
                let error_message = res
                    .json::<SubmitErrorMessage>()
                    .map_err(|err| BatchSubmitterError::InternalError(format!("{}", err)))?;
                Err(BatchSubmitterError::InternalError(error_message.message))
            }
        }
    }

    fn clone_box(&self) -> Box<dyn BatchSubmitter> {
        Box::new(self.clone())
    }
}

#[derive(Deserialize, Debug)]
struct SubmitErrorMessage {
    message: String,
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
    fn from(sbs: SplinterBatchStatus) -> Self {
        Self {
            id: sbs.id,
            status: sbs.status.status_type,
            invalid_transactions: sbs
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
    type Err = BatchSubmitterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split("::");
        let circuit_id: String = parts
            .next()
            .ok_or_else(|| {
                BatchSubmitterError::BadRequestError("Empty service_id parameter provided".into())
            })?
            .into();
        let service_id: String = parts
            .next()
            .ok_or_else(|| {
                BatchSubmitterError::BadRequestError(
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
