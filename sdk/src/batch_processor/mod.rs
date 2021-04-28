// Copyright 2021 Cargill Incorporated
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

mod pacemaker;
pub mod submitter;

use std::{
    sync::{
        mpsc::{channel, Sender},
        Arc,
    },
    thread,
};

use sawtooth_sdk::messages::batch::BatchList;

use crate::{batches::BatchStore, error::InternalError, hex};

use submitter::{BatchSubmitter, BatchSubmitterError, SubmitBatches};

const DEFAULT_PACEMAKER_INTERVAL: u64 = 10;
const DEFAULT_SECS_CLAIM_IS_VALID: i64 = 30;
const DEFAULT_CLAIM_LIMIT: i64 = 1;

pub struct BatchProcessor {
    join_handle: thread::JoinHandle<()>,
    sender: Sender<BatchProcessorMessage>,
    pacemaker: pacemaker::Pacemaker,
}

impl BatchProcessor {
    pub fn connector(&self) -> Connector {
        Connector {
            sender: self.sender.clone(),
        }
    }

    pub fn shutdown_signaler(&self) -> ShutdownSignaler {
        ShutdownSignaler {
            sender: self.sender.clone(),
            pacemaker_shutdown_signaler: self.pacemaker.shutdown_signaler(),
        }
    }

    pub fn await_shutdown(self) {
        debug!("Shutting down batch processor pacemaker...");
        self.pacemaker.await_shutdown();
        debug!("Shutting down batch processor pacemaker (complete)");

        if let Err(err) = self.join_handle.join() {
            error!(
                "Batch processor thread did not shutdown correctly: {:?}",
                err
            );
        }
    }
}

pub struct Connector {
    sender: Sender<BatchProcessorMessage>,
}

impl Connector {
    pub fn wake_up(&self) -> Result<(), InternalError> {
        self.sender
            .send(BatchProcessorMessage::WakeUp)
            .map_err(|err| InternalError::from_source(Box::new(err)))
    }
}

pub enum BatchProcessorMessage {
    WakeUp,
    Shutdown,
}

pub struct BatchProcessorBuilder {
    pacemaker_interval: u64,
    claim_limit: i64,
    secs_claim_is_valid: i64,
    store: Arc<dyn BatchStore>,
    submitter: Arc<dyn BatchSubmitter>,
}

impl BatchProcessorBuilder {
    pub fn new(store: Arc<dyn BatchStore>, submitter: Arc<dyn BatchSubmitter>) -> Self {
        Self {
            store,
            pacemaker_interval: DEFAULT_PACEMAKER_INTERVAL,
            claim_limit: DEFAULT_CLAIM_LIMIT,
            secs_claim_is_valid: DEFAULT_SECS_CLAIM_IS_VALID,
            submitter,
        }
    }

    pub fn with_pacemaker_interval(mut self, pacemaker_interval: u64) -> Self {
        self.pacemaker_interval = pacemaker_interval;
        self
    }

    pub fn with_claim_limit(mut self, claim_limit: i64) -> Self {
        self.claim_limit = claim_limit;
        self
    }

    pub fn with_secs_claim_is_valid(mut self, secs_claim_is_valid: i64) -> Self {
        self.secs_claim_is_valid = secs_claim_is_valid;
        self
    }

    pub fn start(self) -> Result<BatchProcessor, InternalError> {
        let (sender, recv) = channel();

        let store = self.store.clone();
        let submitter = self.submitter.clone();
        let claim_limit = self.claim_limit;
        let secs_claim_is_valid = self.secs_claim_is_valid;

        let join_handle = thread::Builder::new()
            .name("Batch Submitter".into())
            .spawn(move || loop {
                match recv.recv() {
                    Ok(BatchProcessorMessage::Shutdown) => break,
                    Ok(BatchProcessorMessage::WakeUp) => {
                        let batches =
                            match store.get_unclaimed_batches(claim_limit, secs_claim_is_valid) {
                                Ok(ub) => ub,
                                Err(err) => {
                                    error!("Failed to retrieve unclaimed batches: {}", err);
                                    continue;
                                }
                            };

                        for batch_submit_info in batches {
                            let bytes = match hex::parse_hex(&batch_submit_info.serialized_batch) {
                                Ok(b) => b,
                                Err(err) => {
                                    error!("Failed to deserialize batch: {}", err);
                                    if let Err(err) = store.update_submission_error_info(
                                        &batch_submit_info.header_signature,
                                        "Deserialization Error",
                                        &err.to_string(),
                                    ) {
                                        error!("Failed to update error status: {}", err);
                                    }
                                    continue;
                                }
                            };

                            let batch_list: BatchList =
                                match protobuf::Message::parse_from_bytes(&bytes) {
                                    Ok(batch_list) => batch_list,
                                    Err(err) => {
                                        error!("Failed to deserialize batch: {}", err);
                                        if let Err(err) = store.update_submission_error_info(
                                            &batch_submit_info.header_signature,
                                            "Deserialization Error",
                                            &err.to_string(),
                                        ) {
                                            error!("Failed to update error status: {}", err);
                                        }
                                        continue;
                                    }
                                };

                            match submitter.submit_batches(SubmitBatches {
                                batch_list,
                                service_id: batch_submit_info.service_id,
                            }) {
                                Ok(()) => {
                                    info!(
                                        "Batch submitted successfully {}",
                                        batch_submit_info.header_signature
                                    );
                                    if let Err(err) = store.change_batch_to_submitted(
                                        &batch_submit_info.header_signature,
                                    ) {
                                        error!("Failed to update batch status: {}", err);
                                    } else {
                                        info!("Batch status updated to submitted");
                                    }
                                }
                                Err(BatchSubmitterError::BadRequestError(ref msg))
                                | Err(BatchSubmitterError::NotFound(ref msg)) => {
                                    if let Err(err) = store.update_submission_error_info(
                                        &batch_submit_info.header_signature,
                                        "Bad Request",
                                        msg,
                                    ) {
                                        error!("Failed to update error status: {}", err);
                                    }
                                }
                                Err(BatchSubmitterError::ConnectionError(ref msg))
                                | Err(BatchSubmitterError::InternalError(ref msg))
                                | Err(BatchSubmitterError::ResourceTemporarilyUnavailableError(
                                    ref msg,
                                )) => {
                                    error!("Internal service error: {}", msg);
                                    if let Err(err) =
                                        store.relinquish_claim(&batch_submit_info.header_signature)
                                    {
                                        error!("Failed to relinquish claim: {}", err);
                                    } else {
                                        info!("Batch claim relinquished");
                                    }
                                }
                            };
                        }
                    }
                    Err(_) => {
                        warn!("All senders have disconnected");
                        break;
                    }
                }
            })
            .map_err(|err| InternalError::from_source(Box::new(err)))?;

        let pacemaker = pacemaker::Pacemaker::builder()
            .with_interval(self.pacemaker_interval)
            .with_sender(sender.clone())
            .with_message_factory(|| BatchProcessorMessage::WakeUp)
            .start()
            .map_err(|err| InternalError::from_source(Box::new(err)))?;

        Ok(BatchProcessor {
            join_handle,
            sender,
            pacemaker,
        })
    }
}

#[derive(Clone)]
pub struct ShutdownSignaler {
    sender: Sender<BatchProcessorMessage>,
    pacemaker_shutdown_signaler: pacemaker::ShutdownSignaler,
}

impl ShutdownSignaler {
    pub fn shutdown(self) {
        self.pacemaker_shutdown_signaler.shutdown();

        if self.sender.send(BatchProcessorMessage::Shutdown).is_err() {
            warn!("Batch processor is no longer running");
        }
    }
}
