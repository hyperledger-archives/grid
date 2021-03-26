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

use crate::submitter::{BatchSubmitter, SawtoothBatchSubmitter, SplinterBatchSubmitter};

#[cfg(feature = "batch-submitter")]
#[derive(Clone)]
pub struct BatchSubmitterState {
    pub batch_submitter: Arc<dyn BatchSubmitter + 'static>,
}

impl BatchSubmitterState {
    pub fn new(batch_submitter: Arc<dyn BatchSubmitter + 'static>) -> Self {
        Self { batch_submitter }
    }

    pub fn with_sawtooth(submitter: SawtoothBatchSubmitter) -> Self {
        Self {
            batch_submitter: Arc::new(submitter),
        }
    }

    pub fn with_splinter(submitter: SplinterBatchSubmitter) -> Self {
        Self {
            batch_submitter: Arc::new(submitter),
        }
    }
}
