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

use crate::{error::InternalError, scope_id::ScopeId, threading::lifecycle::ShutdownHandle};

pub mod batch_submitter;

/// The interface for a submitter that is built but not yet running.
pub trait RunnableSubmitter<S: ScopeId> {
    type RunningSubmitter: RunningSubmitter<S>;

    /// Start running the submission service.
    ///
    /// This method consumes the `RunnableSubmitter` and returns a `RunningSubmitter`
    fn run(self) -> Result<Self::RunningSubmitter, InternalError>;
}

/// The interface for a running submitter.
pub trait RunningSubmitter<S: ScopeId>: ShutdownHandle {
    type RunnableSubmitter: RunnableSubmitter<S>;

    /// Stop the running submitter service and return a runnable submitter (pause the service).
    fn stop(self) -> Result<Self::RunnableSubmitter, InternalError>;
}
