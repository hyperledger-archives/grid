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
use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};

use sawtooth_xo::handler::XoTransactionHandler;
use transact::context::manager::sync::ContextManager;
use transact::database::{btree::BTreeDatabase, Database};
use transact::execution::{adapter::static_adapter::StaticExecutionAdapter, executor::Executor};
use transact::protocol::batch::Batch;
use transact::sawtooth::SawtoothToTransactHandlerAdapter;
use transact::scheduler::{serial::SerialScheduler, Scheduler, TransactionExecutionResult};
use transact::state::{
    merkle::{MerkleRadixTree, MerkleState, StateDatabaseError, INDEXES},
    StateChange, Write,
};


mod adapter;

const EXECUTION_TIMEOUT: u64 = 300; // five minutes

struct XoShared {
    current_state_root: Option<String>,
    executor: Executor,
    pending_changes: Option<Vec<StateChange>>,
}

impl XoShared {
    fn new(context_manager: ContextManager) -> Result<Self, XoStateError> {
        let mut executor = Executor::new(vec![Box::new(
            StaticExecutionAdapter::new_adapter(
                vec![Box::new(SawtoothToTransactHandlerAdapter::new(
                    XoTransactionHandler::new(),
                ))],
                context_manager,
            )
            .map_err(|err| {
                XoStateError(format!(
                    "Unable to create static execution adapter: {}",
                    err
                ))
            })?,
        )]);

        executor
            .start()
            .map_err(|err| XoStateError(format!("Unable to start executor: {}", err)))?;

        Ok(XoShared {
            current_state_root: None,
            pending_changes: None,
            executor,
        })
    }
}

#[derive(Clone)]
pub struct XoState {
    db: Box<dyn Database>,
    shared: Arc<Mutex<XoShared>>,
    context_manager: ContextManager,
}

impl XoState {
    pub fn new() -> Result<Self, XoStateError> {
        let db = Box::new(BTreeDatabase::new(&INDEXES));

        let context_manager = ContextManager::new(Box::new(MerkleState::new(db.clone())));

        Ok(XoState {
            db,
            shared: Arc::new(Mutex::new(XoShared::new(context_manager.clone())?)),
            context_manager,
        })
    }

    pub fn current_state_root(&self) -> String {
        let mut shared = self
            .shared
            .lock()
            .expect("Current state root lock poisoned");

        XoState::unlocked_current_state_root(&self.db, &mut shared)
    }

    fn unlocked_current_state_root(db: &Box<dyn Database>, shared: &mut XoShared) -> String {
        if shared.current_state_root.is_some() {
            shared.current_state_root.clone().unwrap()
        } else {
            let merkle_db =
                MerkleRadixTree::new(db.clone(), None).expect("Cannot initialize merkle database");

            shared.current_state_root = Some(merkle_db.get_merkle_root());

            shared.current_state_root.clone().unwrap()
        }
    }

    pub fn propose_change(&self, batch: Batch) -> Result<String, XoStateError> {
        let mut shared = self
            .shared
            .lock()
            .expect("Current state root lock poisoned");
        let state_root = XoState::unlocked_current_state_root(&self.db, &mut shared);

        let mut scheduler =
            SerialScheduler::new(Box::new(self.context_manager.clone()), state_root.clone())
                .map_err(|err| XoStateError(format!("Unable to create scheduler")))?;

        let (result_tx, result_rx) = std::sync::mpsc::channel();
        scheduler.set_result_callback(Box::new(move |batch_result| {
            result_tx
                .send(batch_result)
                .expect("Unable to send batch result")
        }));

        let batch_pair = batch
            .into_pair()
            .map_err(|err| XoStateError(format!("Unable to create batch pair: {}", err)))?;
        scheduler.add_batch(batch_pair);
        scheduler.finalize();

        let task_iter = scheduler
            .take_task_iterator()
            .expect("Should have only taken this once");
        shared
            .executor
            .execute(task_iter, scheduler.new_notifier())
            .map_err(|err| XoStateError(format!("Unable to execute schedule: {}", err)))?;

        let batch_result = result_rx
            .recv_timeout(std::time::Duration::from_secs(EXECUTION_TIMEOUT))
            .map_err(|_| XoStateError("Unable to receive result in reasonable time".into()))?
            .ok_or_else(|| XoStateError("No result returned from executor".into()))?;

        scheduler.shutdown();

        let txn_results: Result<Vec<_>, XoStateError> = batch_result
            .results
            .into_iter()
            .map(|txn_result| match txn_result {
                TransactionExecutionResult::Valid(receipt) => Ok(receipt),
                TransactionExecutionResult::Invalid(invalid_result) => Err(XoStateError(format!(
                    "Transaction failed: {:?}",
                    invalid_result
                ))),
            })
            .collect();

        let state_changes = txn_results?
            .into_iter()
            .flat_map(|txn_result| {
                txn_result
                    .state_changes
                    .into_iter()
                    .map(into_writable_state_change)
            })
            .collect::<Vec<_>>();

        shared.pending_changes = Some(state_changes);

        let merkle_state = MerkleState::new(self.db.clone());
        merkle_state
            .compute_state_id(
                &state_root,
                shared.pending_changes.as_ref().unwrap().as_slice(),
            )
            .map_err(|err| XoStateError(format!("unable to compute next state root: {}", err)))
    }

    pub fn get_state(
        &self,
        state_root: &str,
        address: &str,
    ) -> Result<Option<Vec<u8>>, XoStateError> {
        let merkle_db = MerkleRadixTree::new(self.db.clone(), Some(state_root))?;

        merkle_db.get_value(address).map_err(|err| {
            error!("Unable to get value from db: {}", &err);
            XoStateError::from(err)
        })
    }

    pub fn list_state(
        &self,
        state_root: &str,
        prefix: Option<&str>,
    ) -> Result<Box<dyn Iterator<Item = Result<(String, Vec<u8>), XoStateError>>>, XoStateError>
    {
        let merkle_db = MerkleRadixTree::new(self.db.clone(), Some(state_root))?;

        let iter: Box<dyn Iterator<Item = Result<(String, Vec<u8>), StateDatabaseError>>> =
            match merkle_db.leaves(prefix) {
                Ok(iter) => iter,
                Err(StateDatabaseError::NotFound(_)) => {
                    let empty_vec: Vec<Result<(String, Vec<u8>), StateDatabaseError>> = vec![];
                    Box::new(empty_vec.into_iter())
                }
                Err(err) => return Err(XoStateError::from(err)),
            };

        Ok(Box::new(
            iter.map(|entry_res| entry_res.map_err(XoStateError::from)),
        ))
    }

    pub fn commit(&self) -> Result<(), XoStateError> {
        let mut shared = self
            .shared
            .lock()
            .expect("Current state root lock poisoned");

        let state_root = XoState::unlocked_current_state_root(&self.db, &mut shared);
        if let Some(state_changes) = shared.pending_changes.take() {
            let merkle_state = MerkleState::new(self.db.clone());
            let new_state_root = merkle_state
                .commit(&state_root, state_changes.as_slice())
                .map_err(|err| XoStateError(format!("Unable to commit changes: {}", err)))?;

            info!(
                "Committed {} change(s) for new state root {}",
                state_changes.len(),
                &new_state_root
            );
            shared.current_state_root = Some(new_state_root);

            Ok(())
        } else {
            Err(XoStateError("No pending changes to commit".into()))
        }
    }

    pub fn rollback(&self) -> Result<(), XoStateError> {
        let mut shared = self
            .shared
            .lock()
            .expect("Current state root lock poisoned");

        if let Some(state_changes) = shared.pending_changes.take() {
            info!("Discarding {} change(s)", state_changes.len());
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

#[derive(Debug)]
pub struct XoStateError(String);

impl Error for XoStateError {}

impl fmt::Display for XoStateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "State Error: {}", self.0)
    }
}

impl From<transact::database::error::DatabaseError> for XoStateError {
    fn from(err: transact::database::error::DatabaseError) -> Self {
        XoStateError(err.to_string())
    }
}

impl From<StateDatabaseError> for XoStateError {
    fn from(err: StateDatabaseError) -> Self {
        XoStateError(err.to_string())
    }
}
