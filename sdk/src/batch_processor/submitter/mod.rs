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

mod error;
pub mod sawtooth;
pub mod splinter;

use sawtooth_sdk::messages::batch::BatchList;
use sawtooth_sdk::messages::client_batch_submit::ClientBatchStatus;

pub use error::BatchSubmitterError;
pub use sawtooth::{SawtoothBatchSubmitter, SawtoothConnection};
pub use splinter::SplinterBatchSubmitter;

pub trait BatchSubmitter: Send + Sync + 'static {
    fn submit_batches(&self, submit_batches: SubmitBatches) -> Result<(), BatchSubmitterError>;

    fn batch_status(
        &self,
        batch_statuses: BatchStatuses,
    ) -> Result<Vec<BatchStatus>, BatchSubmitterError>;

    fn clone_box(&self) -> Box<dyn BatchSubmitter>;
}

impl Clone for Box<dyn BatchSubmitter> {
    fn clone(&self) -> Box<dyn BatchSubmitter> {
        self.clone_box()
    }
}

pub struct SubmitBatches {
    pub batch_list: BatchList,
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
