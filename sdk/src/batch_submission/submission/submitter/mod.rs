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

use crate::{
    batch_submission::{
        submission::{submitter_observer::SubmitterObserver, url_resolver::UrlResolver},
        Submission,
    },
    error::InternalError,
    scope_id::ScopeId,
};

/// The interface for a submitter builder. Note that a submitter builder implementation may have
/// more methods than this (ex. for testing).
pub trait SubmitterBuilder<
    S: ScopeId,
    R: UrlResolver<Id = S> + Sync + Send,
    Q: Iterator<Item = Submission<S>> + Send,
    O: SubmitterObserver<Id = S> + Send,
>
{
    type RunnableSubmitter: RunnableSubmitter<S, R, Q, O>;

    fn new() -> Self;

    fn with_url_resolver(&mut self, url_resolver: &'static R);

    fn with_queue(&mut self, queue: Q);

    fn with_observer(&mut self, observer: O);

    fn build(self) -> Result<Self::RunnableSubmitter, InternalError>;
}

/// The interface for a submitter that is built but not yet running.
pub trait RunnableSubmitter<
    S: ScopeId,
    R: UrlResolver<Id = S> + Sync + Send,
    Q: Iterator<Item = Submission<S>> + Send,
    O: SubmitterObserver<Id = S> + Send,
>
{
    type RunningSubmitter: RunningSubmitter;

    /// Start running the submission service.
    fn run(self) -> Result<Self::RunningSubmitter, InternalError>;
}

/// The interface for a running submitter service. This is effectively a handle to the service.
pub trait RunningSubmitter {
    /// Signal to the internal submitter components to begin the shutdown process.
    fn signal_shutdown(&self) -> Result<(), InternalError>;

    /// Wind down and stop the submission service.
    fn shutdown(self) -> Result<(), InternalError>;
}
