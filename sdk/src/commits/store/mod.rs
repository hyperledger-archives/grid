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

#[cfg(feature = "diesel")]
pub(crate) mod diesel;
mod error;

#[cfg(feature = "diesel")]
pub use self::diesel::{DieselCommitStore, DieselConnectionCommitStore};
pub use error::CommitStoreError;

/// Represents a Grid commit
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Commit {
    pub commit_id: String,
    pub commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ChainRecord {
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

/// A change that has been applied to state, represented in terms of a key/value pair
#[derive(Clone, Eq, PartialEq)]
pub enum StateChange {
    Set { key: String, value: Vec<u8> },
    Delete { key: String },
}

/// A notification that some source has committed a set of changes to state
#[derive(Clone)]
pub struct CommitEvent {
    /// An identifier for specifying where the event came from
    pub service_id: Option<String>,
    /// An identifier that is unique among events from the source
    pub id: String,
    /// May be used to provide ordering of commits from the source. If `None`, ordering is not
    /// explicitly provided, so it must be inferred from the order in which events are received.
    pub height: Option<u64>,
    /// All state changes that are included in the commit
    pub state_changes: Vec<StateChange>,
}

pub trait CommitStore {
    /// Adds an commit to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `commit` - The commit to be added
    fn add_commit(&self, commit: Commit) -> Result<(), CommitStoreError>;

    /// Gets a commit from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `commit_num` - The commit to be fetched
    fn get_commit_by_commit_num(&self, commit_num: i64)
        -> Result<Option<Commit>, CommitStoreError>;

    /// Gets the current commit ID from the underlying storage
    fn get_current_commit_id(&self) -> Result<Option<String>, CommitStoreError>;

    /// Gets all the current commits on services.
    ///
    /// This returns the latest commit values for all commits where `commit.service_id` is not
    /// `None`.
    fn get_current_service_commits(&self) -> Result<Vec<Commit>, CommitStoreError>;

    /// Gets the next commit number from the underlying storage
    fn get_next_commit_num(&self) -> Result<i64, CommitStoreError>;

    /// Resolves a fork
    ///
    /// # Arguments
    ///
    ///  * `commit_num` - The commit to be fetched
    fn resolve_fork(&self, commit_num: i64) -> Result<(), CommitStoreError>;

    /// Creates a commit model from a commit event
    ///
    /// # Arguments
    ///
    ///  * `event` - The commit event to be processed
    fn create_db_commit_from_commit_event(
        &self,
        event: &CommitEvent,
    ) -> Result<Option<Commit>, CommitStoreError>;
}

impl<CS> CommitStore for Box<CS>
where
    CS: CommitStore + ?Sized,
{
    fn add_commit(&self, commit: Commit) -> Result<(), CommitStoreError> {
        (**self).add_commit(commit)
    }

    fn get_commit_by_commit_num(
        &self,
        commit_num: i64,
    ) -> Result<Option<Commit>, CommitStoreError> {
        (**self).get_commit_by_commit_num(commit_num)
    }

    fn get_current_commit_id(&self) -> Result<Option<String>, CommitStoreError> {
        (**self).get_current_commit_id()
    }

    fn get_current_service_commits(&self) -> Result<Vec<Commit>, CommitStoreError> {
        (**self).get_current_service_commits()
    }

    fn get_next_commit_num(&self) -> Result<i64, CommitStoreError> {
        (**self).get_next_commit_num()
    }

    fn resolve_fork(&self, commit_num: i64) -> Result<(), CommitStoreError> {
        (**self).resolve_fork(commit_num)
    }

    fn create_db_commit_from_commit_event(
        &self,
        event: &CommitEvent,
    ) -> Result<Option<Commit>, CommitStoreError> {
        (**self).create_db_commit_from_commit_event(event)
    }
}
