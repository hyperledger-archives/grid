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
use reqwest::Client;

use std::{fmt, sync::Arc};

use crate::{
    batch_submission::{
        submission::{submitter_observer::SubmitterObserver, url_resolver::UrlResolver},
        Submission,
    },
    error::ClientError,
    scope_id::ScopeId,
};

// Number of times a submitter task will retry submission in quick succession
const RETRY_ATTEMPTS: u16 = 10;

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

// Struct used to collect the queue, url resolver, and observer when the submission service is
// stopped; to be used in Arc<Mutex<Collector>>
struct Collector<S: ScopeId> {
    queue: Option<Box<(dyn Iterator<Item = Submission<S>> + Send)>>,
    observer: Option<Box<dyn SubmitterObserver<Id = S> + Send>>,
    command_factory: Option<Arc<dyn ExecuteCommandFactory<S>>>,
}

impl<S: ScopeId> Collector<S> {
    fn new() -> Self {
        Self {
            queue: None,
            observer: None,
            command_factory: None,
        }
    }
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

#[derive(Debug)]
// Responsible for executing the submission request
struct SubmissionCommand<S: ScopeId> {
    url_resolver: Arc<dyn UrlResolver<Id = S>>,
    submission: Submission<S>,
    attempts: u16,
}

#[async_trait]
impl<S: ScopeId> ExecuteCommand<S> for SubmissionCommand<S> {
    async fn execute(&mut self) -> Result<SubmissionResponse<S>, reqwest::Error> {
        let client = Client::builder()
            .timeout(tokio::time::Duration::from_secs(15))
            .build()?;

        self.attempts += 1;

        let res = client
            .post(&self.url_resolver.url(self.submission.scope_id()))
            .body(self.submission.serialized_batch().clone())
            .send()
            .await?;

        Ok(SubmissionResponse::new(
            self.submission.batch_header().clone(),
            self.submission.scope_id().clone(),
            res.status().as_u16(),
            res.text().await?,
            self.attempts,
        ))
    }
}

#[derive(Clone, Debug)]
// Creates a submission command inside the task
struct SubmissionCommandFactory<S: ScopeId> {
    url_resolver: Arc<dyn UrlResolver<Id = S>>,
}

impl<S: ScopeId> SubmissionCommandFactory<S> {
    fn new(url_resolver: Arc<dyn UrlResolver<Id = S>>) -> Self {
        Self { url_resolver }
    }
}

impl<S: ScopeId> ExecuteCommandFactory<S> for SubmissionCommandFactory<S> {
    fn new_command(&self, submission: Submission<S>) -> Box<dyn ExecuteCommand<S>> {
        Box::new(SubmissionCommand {
            url_resolver: Arc::clone(&self.url_resolver),
            submission,
            attempts: 0,
        })
    }
}

#[derive(Debug, PartialEq)]
// Responsible for controlling retry behavior
struct SubmissionController;

impl SubmissionController {
    async fn run<S: ScopeId>(
        mut command: Box<dyn ExecuteCommand<S>>,
    ) -> Result<SubmissionResponse<S>, ClientError> {
        let mut wait: u64 = 250;
        let mut response: Result<SubmissionResponse<S>, reqwest::Error> = command.execute().await;
        for _ in 1..RETRY_ATTEMPTS {
            match &response {
                Ok(res) => match &res.status {
                    200 => break,
                    503 => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(wait)).await;
                        response = command.execute().await;
                    }
                    _ => break,
                },
                Err(e) => {
                    if e.is_timeout() {
                        tokio::time::sleep(tokio::time::Duration::from_millis(wait)).await;
                        response = command.execute().await;
                    } else {
                        // This error is returned outside of the loop and not behind a &
                        break;
                    }
                }
            }
            wait += 500;
        }
        let res = response.map_err(ClientError::from)?;
        Ok(res)
    }
}

// Task unit within the async runtime
// Responsible for messaging with the synchronous listener thread
struct TaskHandler;

impl TaskHandler {
    async fn spawn<'a, S: ScopeId>(
        task: NewTask<S>,
        submission_command_factory: Arc<dyn ExecuteCommandFactory<S>>,
    ) {
        let batch_header = task.submission.batch_header().clone();
        let scope_id = task.submission.scope_id().clone();
        let submission_command = submission_command_factory.new_command(task.submission);
        let submission: Result<SubmissionResponse<S>, ClientError> =
            SubmissionController::run(submission_command).await;

        let task_message = match submission {
            Ok(s) => BatchMessage::SubmissionResponse(s),
            Err(e) => BatchMessage::ErrorResponse(ErrorResponse {
                batch_header,
                scope_id,
                error: e.to_string(),
            }),
        };
        let _ = task.tx.send(task_message);
    }
}
