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

use transact::database::{btree::BTreeDatabase, Database};
use transact::protocol::batch::Batch;
use transact::state::{
    merkle::{MerkleRadixTree, MerkleState, StateDatabaseError, INDEXES},
};

mod adapter;

struct XoShared {
    current_state_root: Option<String>,
}

impl XoShared {
    fn new() -> Self {
        XoShared {
            current_state_root: None,
        }
    }
}

#[derive(Clone)]
pub struct XoState {
    db: Box<dyn Database>,
    shared: Arc<Mutex<XoShared>>,
}

impl XoState {
    pub fn new() -> Result<Self, XoStateError> {
        let db = Box::new(BTreeDatabase::new(&INDEXES));

        Ok(XoState {
            db,
            shared: Arc::new(Mutex::new(XoShared::new())),
        })
    }

    pub fn current_state_root(&self) -> Option<String> {
        let mut shared = self
            .shared
            .lock()
            .expect("Current state root lock poisoned");

        if shared.current_state_root.is_some() {
            shared.current_state_root.clone()
        } else {
            let merkle_db = MerkleDatabase::new(self.db.clone(), None)
                .expect("Cannot initialize merkle database");

            shared.current_state_root = Some(merkle_db.get_merkle_root());

            shared.current_state_root.clone()
        }
    }

    pub fn propose_change(batch: Batch) -> Result<String, XoStateError> {
        unimplemented!()
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
