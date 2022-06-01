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

use async_trait::async_trait;

use std::fmt;

use crate::{batch_submission::Submission, scope_id::ScopeId};

#[derive(Debug, PartialEq)]
// Carries the submission response from the http client back through the submitter to the observer
struct SubmissionResponse<S: ScopeId> {
    batch_header: String,
    scope_id: S,
    status: u16,
    message: String,
    attempts: u16,
}

impl<S: ScopeId> SubmissionResponse<S> {
    fn new(batch_header: String, scope_id: S, status: u16, message: String, attempts: u16) -> Self {
        Self {
            batch_header,
            scope_id,
            status,
            message,
            attempts,
        }
    }
}

#[derive(Debug, PartialEq)]
// A message about a batch; sent between threads
enum BatchMessage<S: ScopeId> {
    SubmissionNotification((String, S)),
    SubmissionResponse(SubmissionResponse<S>),
    ErrorResponse(ErrorResponse<S>),
}

#[derive(Debug)]
// A message sent from the leader thread to the async runtime
enum CentralMessage<S: ScopeId> {
    NewTask(NewTask<S>),
    Stop,
}

// A message used to instruct the leader and listener threads to terminate
enum ControlMessage {
    // Signal to stop the submission service and collect the configured components
    Stop,
}

// The object required for an async task to function
// Provides the batch and a channel sender with which the task communicates back to the listener
// thread about the batch
struct NewTask<S: ScopeId> {
    tx: std::sync::mpsc::Sender<BatchMessage<S>>,
    submission: Submission<S>,
}

impl<S: ScopeId> NewTask<S> {
    fn new(tx: std::sync::mpsc::Sender<BatchMessage<S>>, submission: Submission<S>) -> Self {
        Self { tx, submission }
    }
}

impl<S: ScopeId> fmt::Debug for NewTask<S> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{:?}", self.submission)
    }
}

#[derive(Debug, PartialEq)]
// Communicates an error message from the task handler to the listener thread
struct ErrorResponse<S: ScopeId> {
    batch_header: String,
    scope_id: S,
    error: String,
}

// Subcomponent traits

trait ExecuteCommandFactory<S: ScopeId>: Sync + Send {
    fn new_command(&self, submission: Submission<S>) -> Box<dyn ExecuteCommand<S>>;
}

#[async_trait]
trait ExecuteCommand<S: ScopeId>: fmt::Debug + Sync + Send {
    async fn execute(&mut self) -> Result<SubmissionResponse<S>, reqwest::Error>;
}
