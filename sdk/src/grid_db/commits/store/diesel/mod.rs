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

pub(in crate::grid_db) mod models;
mod operations;
pub(in crate::grid_db) mod schema;

use diesel::r2d2::{ConnectionManager, Pool};

use super::{Commit, CommitEvent, CommitEventError, CommitStore, CommitStoreError};
use crate::database::DatabaseError;
use crate::grid_db::commits::store::diesel::models::{CommitModel, NewCommitModel};
use operations::add_commit::CommitStoreAddCommitOperation as _;
use operations::create_db_commit_from_commit_event::CommitStoreCreateDbCommitFromCommitEventOperation as _;
use operations::get_commit_by_commit_num::CommitStoreGetCommitByCommitNumOperation as _;
use operations::get_current_commit_id::CommitStoreGetCurrentCommitIdOperation as _;
use operations::get_next_commit_num::CommitStoreGetNextCommitNumOperation as _;
use operations::resolve_fork::CommitStoreResolveForkOperation as _;
use operations::CommitStoreOperations;

/// Manages creating commits in the database
#[derive(Clone)]
pub struct DieselCommitStore<C: diesel::Connection + 'static> {
    connection_pool: Pool<ConnectionManager<C>>,
}

#[cfg(feature = "diesel")]
impl<C: diesel::Connection> DieselCommitStore<C> {
    /// Creates a new DieselCommitStore
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: connection pool to the database
    // Allow dead code if diesel feature is not enabled
    #[allow(dead_code)]
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> Self {
        DieselCommitStore { connection_pool }
    }
}

#[cfg(feature = "postgres")]
impl CommitStore for DieselCommitStore<diesel::pg::PgConnection> {
    fn add_commit(&self, commit: Commit) -> Result<(), CommitStoreError> {
        CommitStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .add_commit(commit.into())
    }

    fn resolve_fork(&self, commit_num: i64) -> Result<(), CommitStoreError> {
        CommitStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .resolve_fork(commit_num)
    }

    fn get_commit_by_commit_num(
        &self,
        commit_num: i64,
    ) -> Result<Option<Commit>, CommitStoreError> {
        CommitStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .get_commit_by_commit_num(commit_num)
    }

    fn get_current_commit_id(&self) -> Result<Option<String>, CommitStoreError> {
        CommitStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .get_current_commit_id()
    }

    fn get_next_commit_num(&self) -> Result<i64, CommitStoreError> {
        CommitStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .get_next_commit_num()
    }

    fn create_db_commit_from_commit_event(
        &self,
        event: &CommitEvent,
    ) -> Result<Option<Commit>, CommitEventError> {
        CommitStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .create_db_commit_from_commit_event(event)
    }
}

impl From<CommitModel> for Commit {
    fn from(commit: CommitModel) -> Self {
        Self {
            commit_id: commit.commit_id,
            commit_num: commit.commit_num,
            service_id: commit.service_id,
        }
    }
}

impl From<NewCommitModel> for Commit {
    fn from(commit: NewCommitModel) -> Self {
        Self {
            commit_id: commit.commit_id,
            commit_num: commit.commit_num,
            service_id: commit.service_id,
        }
    }
}
