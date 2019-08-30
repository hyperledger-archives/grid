// Copyright 2019 Cargill Incorporated
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

use std::path::Path;

use protobuf::Message;
use sawtooth_sabre::handler::SabreTransactionHandler;
use sawtooth_sabre::{ADMINISTRATORS_SETTING_ADDRESS, ADMINISTRATORS_SETTING_KEY};
use transact::context::manager::sync::ContextManager;
use transact::database::{
    lmdb::{LmdbContext, LmdbDatabase},
    Database,
};
use transact::execution::{adapter::static_adapter::StaticExecutionAdapter, executor::Executor};
use transact::protocol::batch::BatchPair;
use transact::sawtooth::SawtoothToTransactHandlerAdapter;
use transact::scheduler::{serial::SerialScheduler, Scheduler, TransactionExecutionResult};
use transact::state::{
    merkle::{MerkleRadixTree, MerkleState, INDEXES},
    StateChange, Write,
};

use crate::protos::scabbard::{Setting, Setting_Entry};
use crate::rest_api::{EventDealer, Request, Response, ResponseError};

use super::error::ScabbardStateError;

const EXECUTION_TIMEOUT: u64 = 300; // five minutes

pub struct ScabbardState {
    db: Box<dyn Database>,
    context_manager: ContextManager,
    executor: Executor,
    current_state_root: String,
    pending_changes: Option<Vec<StateChange>>,
    event_dealer: EventDealer<Vec<StateChangeEvent>>,
}

impl ScabbardState {
    pub fn new(
        db_path: &Path,
        db_size: usize,
        admin_keys: Vec<String>,
    ) -> Result<Self, ScabbardStateError> {
        // Initialize the database
        let db = Box::new(LmdbDatabase::new(
            LmdbContext::new(db_path, INDEXES.len(), Some(db_size))?,
            &INDEXES,
        )?);

        // Set initial state (admin keys)
        let mut admin_keys_entry = Setting_Entry::new();
        admin_keys_entry.set_key(ADMINISTRATORS_SETTING_KEY.into());
        admin_keys_entry.set_value(admin_keys.join(","));
        let mut admin_keys_setting = Setting::new();
        admin_keys_setting.set_entries(vec![admin_keys_entry].into());
        let admin_keys_setting_bytes = admin_keys_setting.write_to_bytes().map_err(|err| {
            ScabbardStateError(format!(
                "failed to write admin keys setting to bytes: {}",
                err
            ))
        })?;
        let admin_keys_state_change = StateChange::Set {
            key: ADMINISTRATORS_SETTING_ADDRESS.into(),
            value: admin_keys_setting_bytes,
        };

        let initial_state_root = MerkleRadixTree::new(db.clone_box(), None)?.get_merkle_root();
        let current_state_root = MerkleState::new(db.clone()).commit(
            &initial_state_root,
            vec![admin_keys_state_change].as_slice(),
        )?;

        // Initialize transact
        let context_manager = ContextManager::new(Box::new(MerkleState::new(db.clone())));
        let executor = Executor::new(vec![Box::new(StaticExecutionAdapter::new_adapter(
            vec![Box::new(SawtoothToTransactHandlerAdapter::new(
                SabreTransactionHandler::new(),
            ))],
            context_manager.clone(),
        )?)]);

        let event_dealer = EventDealer::new();

        Ok(ScabbardState {
            db,
            context_manager,
            executor,
            current_state_root,
            pending_changes: None,
            event_dealer,
        })
    }

    pub fn prepare_change(&mut self, batch: BatchPair) -> Result<String, ScabbardStateError> {
        // Setup the transact scheduler
        let (result_tx, result_rx) = std::sync::mpsc::channel();
        let mut scheduler = SerialScheduler::new(
            Box::new(self.context_manager.clone()),
            self.current_state_root.clone(),
        )?;
        scheduler.set_result_callback(Box::new(move |batch_result| {
            result_tx
                .send(batch_result)
                .expect("Unable to send batch result")
        }))?;

        // Add the batch to, finalize, and execute the scheduler
        scheduler.add_batch(batch)?;
        scheduler.finalize()?;
        self.executor
            .execute(scheduler.take_task_iterator()?, scheduler.new_notifier()?)?;

        // Get the results and shutdown the scheduler
        let batch_result = result_rx
            .recv_timeout(std::time::Duration::from_secs(EXECUTION_TIMEOUT))
            .map_err(|_| ScabbardStateError("failed to receive result in reasonable time".into()))?
            .ok_or_else(|| ScabbardStateError("no result returned from executor".into()))?;
        let txn_results = batch_result
            .results
            .into_iter()
            .map(|txn_result| match txn_result {
                TransactionExecutionResult::Valid(receipt) => Ok(receipt),
                TransactionExecutionResult::Invalid(invalid_result) => Err(ScabbardStateError(
                    format!("transaction failed: {:?}", invalid_result),
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let state_changes = txn_results
            .into_iter()
            .flat_map(|txn_result| {
                txn_result
                    .state_changes
                    .into_iter()
                    .map(into_writable_state_change)
            })
            .collect::<Vec<_>>();
        scheduler.shutdown();

        // Save the results and compute the resulting state root
        self.pending_changes = Some(state_changes);
        Ok(MerkleState::new(self.db.clone()).compute_state_id(
            &self.current_state_root,
            self.pending_changes.as_ref().unwrap().as_slice(),
        )?)
    }

    pub fn commit(&mut self) -> Result<(), ScabbardStateError> {
        match self.pending_changes.take() {
            Some(state_changes) => {
                self.current_state_root = MerkleState::new(self.db.clone())
                    .commit(&self.current_state_root, state_changes.as_slice())?;

                info!(
                    "committed {} change(s) for new state root {}",
                    state_changes.len(),
                    self.current_state_root,
                );

                let events = state_changes
                    .into_iter()
                    .map(StateChangeEvent::from_state_change)
                    .collect();

                self.event_dealer.dispatch(events);
                Ok(())
            }
            None => Err(ScabbardStateError("no pending changes to commit".into())),
        }
    }

    pub fn rollback(&mut self) -> Result<(), ScabbardStateError> {
        match self.pending_changes.take() {
            Some(state_changes) => info!("discarded {} change(s)", state_changes.len()),
            None => debug!("no changes to rollback"),
        }

        Ok(())
    }

    pub fn subscribe_to_state(&mut self, request: Request) -> Result<Response, ResponseError> {
        self.event_dealer.subscribe(request)
    }
}

fn into_writable_state_change(
    change: transact::protocol::receipt::StateChange,
) -> transact::state::StateChange {
    match change {
        transact::protocol::receipt::StateChange::Set { key, value } => {
            transact::state::StateChange::Set { key, value }
        }
        transact::protocol::receipt::StateChange::Delete { key } => {
            transact::state::StateChange::Delete { key }
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "eventType", content = "message")]
enum StateChangeEvent {
    Set { key: String, value: Vec<u8> },
    Delete { key: String },
}

impl StateChangeEvent {
    fn from_state_change(state_change: StateChange) -> Self {
        match state_change {
            StateChange::Set { key, value } => StateChangeEvent::Set { key, value },
            StateChange::Delete { key } => StateChangeEvent::Delete { key },
        }
    }
}
