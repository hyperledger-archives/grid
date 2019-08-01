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

use sawtooth_sabre::handler::SabreTransactionHandler;
use transact::context::manager::sync::ContextManager;
use transact::database::{btree::BTreeDatabase, Database};
use transact::execution::{adapter::static_adapter::StaticExecutionAdapter, executor::Executor};
use transact::protocol::batch::BatchPair;
use transact::sawtooth::SawtoothToTransactHandlerAdapter;
use transact::scheduler::{serial::SerialScheduler, Scheduler, TransactionExecutionResult};
use transact::state::{
    merkle::{MerkleRadixTree, MerkleState, INDEXES},
    StateChange, Write,
};

use super::error::ScabbardStateError;

const EXECUTION_TIMEOUT: u64 = 300; // five minutes

pub struct ScabbardState {
    db: Box<dyn Database>,
    context_manager: ContextManager,
    executor: Executor,
    current_state_root: String,
    pending_changes: Option<Vec<StateChange>>,
}

impl ScabbardState {
    pub fn new() -> Result<Self, ScabbardStateError> {
        let db = Box::new(BTreeDatabase::new(&INDEXES));
        let context_manager = ContextManager::new(Box::new(MerkleState::new(db.clone())));
        let executor = Executor::new(vec![Box::new(StaticExecutionAdapter::new_adapter(
            vec![Box::new(SawtoothToTransactHandlerAdapter::new(
                SabreTransactionHandler::new(),
            ))],
            context_manager.clone(),
        )?)]);
        let current_state_root = MerkleRadixTree::new(db.clone_box(), None)?.get_merkle_root();

        Ok(ScabbardState {
            db,
            context_manager,
            executor,
            current_state_root,
            pending_changes: None,
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
