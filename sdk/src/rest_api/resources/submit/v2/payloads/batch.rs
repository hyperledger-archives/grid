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

use cylinder::Signer;
use transact::{
    protocol::{
        batch::{BatchBuildError, BatchBuilder as TransactBatchBuilder},
        transaction::Transaction,
    },
    protos::IntoBytes,
};

use crate::rest_api::resources::error::ErrorResponse;
use crate::rest_api::resources::submit::v2::payloads::TransactionPayload;

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct TrackingBatchResource {
    pub signed_batch: BatchBytes,
    #[serde(flatten)]
    pub batch_identity: BatchIdentifier,
    pub signer_public_key: String,
}

/// A serialized `Batch`
pub type BatchBytes = Vec<u8>;

/// Represents a list of batches created from the REST API
#[derive(Default, Serialize, Deserialize)]
pub struct SubmitBatchRequest {
    pub batches: Vec<TrackingBatchResource>,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
/// A batch's identifying information
pub struct BatchIdentifier {
    pub dlt_batch_id: String,
    pub data_change_id: Option<String>,
    pub service_id: Option<String>,
}

#[derive(Default)]
pub struct TrackingBatchResourceBuilder {
    pub transactions: Vec<Box<dyn TransactionPayload>>,
    pub data_change_id: Option<String>,
    pub service_id: Option<String>,
}

impl TrackingBatchResourceBuilder {
    fn with_transactions(mut self, transactions: Vec<Box<dyn TransactionPayload>>) -> Self {
        self.transactions = transactions;
        self
    }

    fn with_data_change_id(mut self, data_change_id: String) -> Self {
        self.data_change_id = Some(data_change_id);
        self
    }

    fn with_service_id(mut self, service_id: String) -> Self {
        self.service_id = Some(service_id);
        self
    }

    fn build(self, signer: Box<dyn Signer>) -> Result<TrackingBatchResource, ErrorResponse> {
        let signer_public_key = signer
            .public_key()
            .map_err(|err| ErrorResponse::internal_error(Box::new(err)))?
            .as_hex();
        // Create the Transact batch with the list of transaction payloads
        let signed_transactions = self
            .transactions
            .iter()
            .map(|txn| txn.build_transaction(signer.clone()))
            .collect::<Result<Vec<Transaction>, ErrorResponse>>()?;
        let transact_batch = TransactBatchBuilder::new()
            .with_transactions(signed_transactions)
            .build(&*signer)?;
        let batch_identity = BatchIdentifier {
            dlt_batch_id: transact_batch.header_signature().to_string(),
            data_change_id: self.data_change_id,
            service_id: self.service_id,
        };
        let signed_batch = transact_batch
            .into_bytes()
            .map_err(|err| ErrorResponse::internal_error(Box::new(err)))?;

        Ok(TrackingBatchResource {
            signed_batch,
            batch_identity,
            signer_public_key,
        })
    }
}

/// Convert the Transact-specific BatchBuildError into an ErrorResponse that may be sent across
/// the REST API
impl From<BatchBuildError> for ErrorResponse {
    fn from(build_error: BatchBuildError) -> Self {
        match build_error {
            BatchBuildError::MissingField(msg) => {
                ErrorResponse::new(400, &format!("Unable to build Transact batch: {msg}"))
            }
            BatchBuildError::SerializationError(msg) => {
                ErrorResponse::new(500, &format!("Failed to serialize batch parts: {msg}"))
            }
            BatchBuildError::DeserializationError(msg) => {
                ErrorResponse::new(400, &format!("Unable to deserialize batch: {msg}"))
            }
            BatchBuildError::SigningError(msg) => {
                ErrorResponse::new(500, &format!("Unable to sign batch: {msg}"))
            }
        }
    }
}
