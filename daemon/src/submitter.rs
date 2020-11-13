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

use std::pin::Pin;

use futures::prelude::*;
use sawtooth_sdk::messages::batch::BatchList;
use sawtooth_sdk::messages::client_batch_submit::ClientBatchStatus;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::rest_api::error::RestApiResponseError;

pub const DEFAULT_TIME_OUT: u32 = 300; // Max timeout 300 seconds == 5 minutes

pub trait BatchSubmitter: Send + 'static {
    fn submit_batches(
        &self,
        submit_batches: SubmitBatches,
    ) -> Pin<Box<dyn Future<Output = Result<BatchStatusLink, RestApiResponseError>> + Send>>;

    fn batch_status(
        &self,
        batch_statuses: BatchStatuses,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<BatchStatus>, RestApiResponseError>> + Send>>;

    fn clone_box(&self) -> Box<dyn BatchSubmitter>;
}

impl Clone for Box<dyn BatchSubmitter> {
    fn clone(&self) -> Box<dyn BatchSubmitter> {
        self.clone_box()
    }
}
pub struct SubmitBatches {
    pub batch_list: BatchList,
    pub response_url: Url,
    pub service_id: Option<String>,
}

pub struct BatchStatuses {
    pub batch_ids: Vec<String>,
    pub wait: Option<u32>,
    pub service_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchStatus {
    pub id: String,
    pub invalid_transactions: Vec<InvalidTransaction>,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InvalidTransaction {
    pub id: String,
    pub message: String,
    pub extended_data: String,
}

impl BatchStatus {
    pub fn from_proto(proto: &ClientBatchStatus) -> BatchStatus {
        BatchStatus {
            id: proto.get_batch_id().to_string(),
            invalid_transactions: proto
                .get_invalid_transactions()
                .iter()
                .map(|txn| InvalidTransaction {
                    id: txn.get_transaction_id().to_string(),
                    message: txn.get_message().to_string(),
                    extended_data: base64::encode(txn.get_extended_data()),
                })
                .collect(),
            status: format!("{:?}", proto.get_status()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchStatusResponse {
    pub data: Vec<BatchStatus>,
    pub link: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchStatusLink {
    pub link: String,
}
