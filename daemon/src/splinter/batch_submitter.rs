/*
 * Copyright 2020 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use crate::rest_api::error::RestApiResponseError;
use crate::submitter::{
    BatchStatus, BatchStatusLink, BatchStatuses, BatchSubmitter, SubmitBatches,
};

#[derive(Clone)]
pub struct SplinterBatchSubmitter {}

impl SplinterBatchSubmitter {
    pub fn new() -> Self {
        Self {}
    }
}

impl BatchSubmitter for SplinterBatchSubmitter {
    fn submit_batches(
        &self,
        _submit_batches: SubmitBatches,
    ) -> Result<BatchStatusLink, RestApiResponseError> {
        Err(RestApiResponseError::RequestHandlerError(format!(
            "Operation not supported"
        )))
    }

    fn batch_status(
        &self,
        _batch_statuses: BatchStatuses,
    ) -> Result<Vec<BatchStatus>, RestApiResponseError> {
        Err(RestApiResponseError::RequestHandlerError(format!(
            "Operation not supported"
        )))
    }

    fn clone_box(&self) -> Box<dyn BatchSubmitter> {
        Box::new(self.clone())
    }
}
