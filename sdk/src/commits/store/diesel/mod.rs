// Copyright 2018-2021 Cargill Incorporated
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

pub(in crate::commits) mod models;
mod operations;
pub(in crate) mod schema;

use diesel::r2d2::{ConnectionManager, Pool};

use super::{Commit, CommitEvent, CommitEventError, CommitStore, CommitStoreError};
use crate::commits::store::diesel::models::{CommitModel, NewCommitModel};
use crate::error::{
    ConstraintViolationError, ConstraintViolationType, InternalError,
    ResourceTemporarilyUnavailableError,
};
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
            CommitStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_commit(commit.into())
    }

    fn resolve_fork(&self, commit_num: i64) -> Result<(), CommitStoreError> {
        CommitStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            CommitStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .resolve_fork(commit_num)
    }

    fn get_commit_by_commit_num(
        &self,
        commit_num: i64,
    ) -> Result<Option<Commit>, CommitStoreError> {
        CommitStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            CommitStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_commit_by_commit_num(commit_num)
    }

    fn get_current_commit_id(&self) -> Result<Option<String>, CommitStoreError> {
        CommitStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            CommitStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_current_commit_id()
    }

    fn get_next_commit_num(&self) -> Result<i64, CommitStoreError> {
        CommitStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            CommitStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_next_commit_num()
    }

    fn create_db_commit_from_commit_event(
        &self,
        event: &CommitEvent,
    ) -> Result<Option<Commit>, CommitEventError> {
        CommitStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            CommitEventError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .create_db_commit_from_commit_event(event)
    }
}

#[cfg(feature = "sqlite")]
impl CommitStore for DieselCommitStore<diesel::sqlite::SqliteConnection> {
    fn add_commit(&self, commit: Commit) -> Result<(), CommitStoreError> {
        CommitStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            CommitStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_commit(commit.into())
    }

    fn resolve_fork(&self, commit_num: i64) -> Result<(), CommitStoreError> {
        CommitStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            CommitStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .resolve_fork(commit_num)
    }

    fn get_commit_by_commit_num(
        &self,
        commit_num: i64,
    ) -> Result<Option<Commit>, CommitStoreError> {
        CommitStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            CommitStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_commit_by_commit_num(commit_num)
    }

    fn get_current_commit_id(&self) -> Result<Option<String>, CommitStoreError> {
        CommitStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            CommitStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_current_commit_id()
    }

    fn get_next_commit_num(&self) -> Result<i64, CommitStoreError> {
        CommitStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            CommitStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_next_commit_num()
    }

    fn create_db_commit_from_commit_event(
        &self,
        event: &CommitEvent,
    ) -> Result<Option<Commit>, CommitEventError> {
        CommitStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            CommitEventError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
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

impl Into<NewCommitModel> for Commit {
    fn into(self) -> NewCommitModel {
        NewCommitModel {
            commit_id: self.commit_id,
            commit_num: self.commit_num,
            service_id: self.service_id,
        }
    }
}

pub trait CloneBoxCommitStore: CommitStore {
    fn clone_box(&self) -> Box<dyn CloneBoxCommitStore>;
}

impl Clone for Box<dyn CloneBoxCommitStore> {
    fn clone(&self) -> Box<dyn CloneBoxCommitStore> {
        self.clone_box()
    }
}

impl std::fmt::Display for CommitEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("(")?;
        f.write_str(&self.id)?;
        f.write_str(", ")?;
        if self.service_id.is_some() {
            write!(f, "{}, ", self.service_id.as_ref().unwrap())?;
        }
        if self.height.is_some() {
            write!(f, "height: {}, ", self.height.as_ref().unwrap())?;
        }

        write!(f, "#changes: {})", self.state_changes.len())
    }
}

impl From<diesel::result::Error> for CommitStoreError {
    fn from(err: diesel::result::Error) -> CommitStoreError {
        match err {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => CommitStoreError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::Unique,
                    Box::new(err),
                ),
            ),
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                _,
            ) => CommitStoreError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::ForeignKey,
                    Box::new(err),
                ),
            ),
            _ => CommitStoreError::InternalError(InternalError::from_source(Box::new(err))),
        }
    }
}

impl From<diesel::r2d2::PoolError> for CommitStoreError {
    fn from(err: diesel::r2d2::PoolError) -> CommitStoreError {
        CommitStoreError::ResourceTemporarilyUnavailableError(
            ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
        )
    }
}
