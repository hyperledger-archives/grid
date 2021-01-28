// Copyright 2018-2020 Cargill Incorporated
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

use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};

use super::CommitStore;
use crate::commits::store::{
    error::{CommitEventError, CommitStoreError},
    ChainRecord, Commit, CommitEvent,
};
use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;

/// Implementation of CommitStore that stores Commits in memory. Useful for when persistence isn't
/// necessary.
#[derive(Clone, Default)]
pub struct MemoryCommitStore {
    inner_commit: Arc<Mutex<HashMap<String, Commit>>>,
    inner_cr: Arc<Mutex<HashMap<String, ChainRecord>>>,
}

impl MemoryCommitStore {
    pub fn new() -> Self {
        MemoryCommitStore {
            inner_commit: Arc::new(Mutex::new(HashMap::new())),
            inner_cr: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl CommitStore for MemoryCommitStore {
    fn add_commit(&self, commit: Commit) -> Result<(), CommitStoreError> {
        let mut inner_commit = self.inner_commit.lock().map_err(|_| {
            CommitStoreError::InternalError(InternalError::with_message(
                "Cannot access commits: mutex lock poisoned".to_string(),
            ))
        })?;
        inner_commit.insert(commit.commit_id.clone(), commit);
        Ok(())
    }

    fn resolve_fork(&self, commit_num: i64) -> Result<(), CommitStoreError> {
        let mut inner_commit = self.inner_commit.lock().map_err(|_| {
            CommitStoreError::InternalError(InternalError::with_message(
                "Cannot access commits: mutex lock poisoned".to_string(),
            ))
        })?;
        let mut inner_cr = self.inner_cr.lock().map_err(|_| {
            CommitStoreError::InternalError(InternalError::with_message(
                "Cannot access chain_records: mutex lock poisoned".to_string(),
            ))
        })?;

        inner_cr.retain(|_, v| v.start_commit_num.lt(&commit_num));

        for (_, v) in inner_cr.iter_mut() {
            if v.end_commit_num.ge(&commit_num) {
                v.end_commit_num = MAX_COMMIT_NUM;
            }
        }

        inner_commit.retain(|_, v| v.commit_num.lt(&commit_num));

        Ok(())
    }

    fn get_commit_by_commit_num(
        &self,
        commit_num: i64,
    ) -> Result<Option<Commit>, CommitStoreError> {
        let inner_commit = self.inner_commit.lock().map_err(|_| {
            CommitStoreError::InternalError(InternalError::with_message(
                "Cannot access commits: mutex lock poisoned".to_string(),
            ))
        })?;
        if let Some(commit) = inner_commit.get(&commit_num.to_string()) {
            Ok(Some(commit.clone()))
        } else {
            Err(CommitStoreError::NotFoundError(format!(
                "Commit with commit_num {} not found.",
                commit_num
            )))
        }
    }

    fn get_current_commit_id(&self) -> Result<Option<String>, CommitStoreError> {
        let inner_commit = self.inner_commit.lock().map_err(|_| {
            CommitStoreError::InternalError(InternalError::with_message(
                "Cannot access commits: mutex lock poisoned".to_string(),
            ))
        })?;

        if let Some(commit) = inner_commit.values().max_by_key(|v| v.commit_num) {
            Ok(Some(commit.commit_id.clone()))
        } else {
            Err(CommitStoreError::NotFoundError(
                "Commit not found".to_string(),
            ))
        }
    }

    fn get_next_commit_num(&self) -> Result<i64, CommitStoreError> {
        let inner_commit = self.inner_commit.lock().map_err(|_| {
            CommitStoreError::InternalError(InternalError::with_message(
                "Cannot access commits: mutex lock poisoned".to_string(),
            ))
        })?;

        if let Some(commit) = inner_commit.values().max_by_key(|v| v.commit_num) {
            Ok(commit.commit_num + 1)
        } else {
            Ok(0)
        }
    }

    fn create_db_commit_from_commit_event(
        &self,
        event: &CommitEvent,
    ) -> Result<Option<Commit>, CommitEventError> {
        let inner_commit = self.inner_commit.lock().map_err(|_| {
            CommitEventError::InternalError(InternalError::with_message(
                "Cannot access commits: mutex lock poisoned".to_string(),
            ))
        })?;

        let commit_id = event.id.clone();
        let commit_num = match event.height {
            Some(height_u64) => height_u64.try_into().map_err(|err| {
                CommitEventError::InternalError(InternalError::from_source(Box::new(err)))
            })?,
            None => {
                if let Some(commit) = inner_commit.values().max_by_key(|v| v.commit_num) {
                    commit.commit_num + 1
                } else {
                    0
                }
            }
        };
        let service_id = event.service_id.clone();
        Ok(Some(Commit {
            commit_id,
            commit_num,
            service_id,
        }))
    }
}
