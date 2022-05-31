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

//! Provides an interface for submitting a `Batch` via a REST API

#[cfg(feature = "rest-api-batch-submission-handler-reqwest")]
mod reqwest;

#[cfg(feature = "rest-api-batch-submission-handler-reqwest")]
pub use self::reqwest::ReqwestBatchSubmissionHandler;

use std::future::Future;
use std::pin::Pin;

use crate::rest_api::resources::{
    error::ErrorResponse, submit::v2::payloads::batch::BatchIdentifier,
};

/// Defines interactions for submitting a `Batch` via the REST API.
pub trait BatchSubmissionHandler: Send + Sync + 'static {
    // Submit a `Batch` to be persisted in state and submitted to the backend DLT.
    fn submit_batches(self, submit_request: SerializedSubmitBatchRequest) -> SubmitBatchResponse;

    fn cloned_box(&self) -> Box<dyn BatchSubmissionHandler>;
}

impl Clone for Box<dyn BatchSubmissionHandler> {
    fn clone(&self) -> Box<dyn BatchSubmissionHandler> {
        self.cloned_box()
    }
}

/// Future which results in a list of `Batch`s that have been submitted
pub type SubmitBatchResponse =
    Pin<Box<dyn Future<Output = Result<BatchIdList, SubmitBatchErrorResponse>> + Send>>;

/// Represents a list of Batch IDs of the persisted batches
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct BatchIdList {
    pub batch_identifiers: Vec<BatchIdentifier>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct SerializedSubmitBatchRequest {
    // Represents the serialized bytes of a list of `TrackingBatchResource`s
    pub body: Vec<u8>,
}

/// Represents errors encountered in the process of persisting a `Batch`
#[derive(Default, Debug, Deserialize, Serialize, PartialEq)]
pub struct SubmitBatchErrorResponse {
    pub status: u16,
    pub message: String,
}

impl SubmitBatchErrorResponse {
    pub fn new(status: u16, message: &str) -> Self {
        Self {
            status,
            message: message.to_string(),
        }
    }
}

impl From<ErrorResponse> for SubmitBatchErrorResponse {
    fn from(err: ErrorResponse) -> Self {
        Self {
            status: err.status_code(),
            message: err.message().to_string(),
        }
    }
}
