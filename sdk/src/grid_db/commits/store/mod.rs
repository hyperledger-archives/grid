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

#[cfg(feature = "diesel")]
pub mod diesel;
mod error;
pub mod memory;

pub use error::{CommitEventError, CommitStoreError};

#[cfg(feature = "diesel")]
use self::diesel::models::NewCommitModel;

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

// /// A notification that some source has committed a set of changes to state
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

#[cfg(feature = "diesel")]
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

pub trait CommitStore: Send + Sync {
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
    ///
    /// # Arguments
    ///
    fn get_current_commit_id(&self) -> Result<Option<String>, CommitStoreError>;

    /// Gets the next commit number from the underlying storage
    ///
    /// # Arguments
    ///
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
    ) -> Result<Option<Commit>, CommitEventError>;
}

#[cfg(feature = "diesel")]
impl Into<NewCommitModel> for Commit {
    fn into(self) -> NewCommitModel {
        NewCommitModel {
            commit_id: self.commit_id,
            commit_num: self.commit_num,
            service_id: self.service_id,
        }
    }
}

#[cfg(feature = "diesel")]
pub trait CloneBoxCommitStore: CommitStore {
    fn clone_box(&self) -> Box<dyn CloneBoxCommitStore>;
}

#[cfg(feature = "diesel")]
impl Clone for Box<dyn CloneBoxCommitStore> {
    fn clone(&self) -> Box<dyn CloneBoxCommitStore> {
        self.clone_box()
    }
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

    fn get_next_commit_num(&self) -> Result<i64, CommitStoreError> {
        (**self).get_next_commit_num()
    }

    fn resolve_fork(&self, commit_num: i64) -> Result<(), CommitStoreError> {
        (**self).resolve_fork(commit_num)
    }

    fn create_db_commit_from_commit_event(
        &self,
        event: &CommitEvent,
    ) -> Result<Option<Commit>, CommitEventError> {
        (**self).create_db_commit_from_commit_event(event)
    }
}
