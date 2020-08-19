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

pub mod diesel;
mod error;

use std::convert::{TryFrom, TryInto};

pub use error::{CommitEventError, CommitStoreError, EventError, EventIoError};

#[cfg(feature = "diesel")]
use self::diesel::models::NewCommitModel;

#[cfg(feature = "sawtooth-compat")]
use sawtooth_sdk::messages::events::Event as SawtoothEvent;
#[cfg(feature = "sawtooth-compat")]
use sawtooth_sdk::messages::transaction_receipt::{
    StateChange as SawtoothStateChange, StateChangeList,
    StateChange_Type as SawtoothStateChange_Type,
};

pub const PIKE_NAMESPACE: &str = "cad11d";
pub const PIKE_AGENT: &str = "cad11d00";
pub const PIKE_ORG: &str = "cad11d01";

pub const GRID_NAMESPACE: &str = "621dee";
pub const GRID_SCHEMA: &str = "621dee01";
pub const GRID_PRODUCT: &str = "621dee02";

pub const TRACK_AND_TRACE_NAMESPACE: &str = "a43b46";
pub const TRACK_AND_TRACE_PROPERTY: &str = "a43b46ea";
pub const TRACK_AND_TRACE_PROPOSAL: &str = "a43b46aa";
pub const TRACK_AND_TRACE_RECORD: &str = "a43b46ec";

pub const SABRE_NAMESPACE: &str = "00ec";

pub const IGNORED_NAMESPACES: &[&str] = &[SABRE_NAMESPACE];

pub const ALL_GRID_NAMESPACES: &[&str] =
    &[PIKE_NAMESPACE, GRID_NAMESPACE, TRACK_AND_TRACE_NAMESPACE];

pub const NULL_BLOCK_ID: &str = "0000000000000000";
#[cfg(feature = "sawtooth-compat")]
const BLOCK_COMMIT_EVENT_TYPE: &str = "sawtooth/block-commit";
#[cfg(feature = "sawtooth-compat")]
const STATE_CHANGE_EVENT_TYPE: &str = "sawtooth/state-delta";
const BLOCK_ID_ATTR: &str = "block_id";
const BLOCK_NUM_ATTR: &str = "block_num";

/// Represents a Grid commit
#[derive(Clone, Serialize)]
pub struct Commit {
    pub id: i64,
    pub commit_id: String,
    pub commit_num: i64,
    pub service_id: Option<String>,
}

/// A change that has been applied to state, represented in terms of a key/value pair
#[derive(Eq, PartialEq)]
pub enum StateChange {
    Set { key: String, value: Vec<u8> },
    Delete { key: String },
}

impl StateChange {
    pub fn key_has_prefix(&self, prefix: &str) -> bool {
        let key = match self {
            Self::Set { key, .. } => key,
            Self::Delete { key, .. } => key,
        };
        key.get(0..prefix.len())
            .map(|key_prefix| key_prefix == prefix)
            .unwrap_or(false)
    }

    pub fn is_grid_state_change(&self) -> bool {
        ALL_GRID_NAMESPACES
            .iter()
            .any(|namespace| self.key_has_prefix(namespace))
    }
}

#[cfg(feature = "sawtooth-compat")]
impl TryInto<StateChange> for SawtoothStateChange {
    type Error = EventIoError;

    fn try_into(self) -> Result<StateChange, Self::Error> {
        match self.field_type {
            SawtoothStateChange_Type::TYPE_UNSET => Err(EventIoError::InvalidMessage(
                "state change type unset".into(),
            )),
            SawtoothStateChange_Type::SET => Ok(StateChange::Set {
                key: self.address,
                value: self.value,
            }),
            SawtoothStateChange_Type::DELETE => Ok(StateChange::Delete { key: self.address }),
        }
    }
}

/// A notification that some source has committed a set of changes to state
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

#[cfg(feature = "sawtooth-compat")]
impl TryFrom<&[SawtoothEvent]> for CommitEvent {
    type Error = EventIoError;

    fn try_from(events: &[SawtoothEvent]) -> Result<Self, Self::Error> {
        let (id, height) = get_id_and_height(events)?;
        let state_changes = get_state_changes(events)?;

        Ok(CommitEvent {
            service_id: None, // sawtooth is identified by the null service_id
            id,
            height,
            state_changes,
        })
    }
}

#[cfg(feature = "sawtooth-compat")]
fn get_id_and_height(events: &[SawtoothEvent]) -> Result<(String, Option<u64>), EventIoError> {
    let block_event = get_block_event(events)?;
    let block_id = get_required_attribute_from_event(block_event, BLOCK_ID_ATTR)?;
    let block_num = get_required_attribute_from_event(block_event, BLOCK_NUM_ATTR)?
        .parse::<u64>()
        .map_err(|err| {
            EventIoError::InvalidMessage(format!("block_num was not a valid u64: {}", err))
        })?;
    Ok((block_id, Some(block_num)))
}

#[cfg(feature = "sawtooth-compat")]
fn get_state_changes(events: &[SawtoothEvent]) -> Result<Vec<StateChange>, EventIoError> {
    Ok(events
        .iter()
        .filter(|event| event.get_event_type() == STATE_CHANGE_EVENT_TYPE)
        .map(|event| {
            get_sawtooth_state_changes_from_sawtooth_event(&event)
                .and_then(sawtooth_state_changes_into_native_state_changes)
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .filter(|state_change| state_change.is_grid_state_change())
        .collect())
}

#[cfg(feature = "sawtooth-compat")]
fn get_sawtooth_state_changes_from_sawtooth_event(
    sawtooth_event: &SawtoothEvent,
) -> Result<Vec<SawtoothStateChange>, EventIoError> {
    protobuf::parse_from_bytes::<StateChangeList>(&sawtooth_event.data)
        .map(|mut list| list.take_state_changes().to_vec())
        .map_err(|err| {
            EventIoError::InvalidMessage(format!(
                "failed to parse state change list from state change event: {}",
                err
            ))
        })
}

#[cfg(feature = "sawtooth-compat")]
fn sawtooth_state_changes_into_native_state_changes(
    sawtooth_state_changes: Vec<SawtoothStateChange>,
) -> Result<Vec<StateChange>, EventIoError> {
    sawtooth_state_changes
        .into_iter()
        .map(|sawtooth_state_change| sawtooth_state_change.try_into())
        .collect()
}

#[cfg(feature = "sawtooth-compat")]
fn get_block_event(events: &[SawtoothEvent]) -> Result<&SawtoothEvent, EventIoError> {
    events
        .iter()
        .find(|event| event.get_event_type() == BLOCK_COMMIT_EVENT_TYPE)
        .ok_or_else(|| EventIoError::InvalidMessage("no block event found".into()))
}

#[cfg(feature = "sawtooth-compat")]
fn get_required_attribute_from_event(
    event: &SawtoothEvent,
    required_attr_key: &str,
) -> Result<String, EventIoError> {
    event
        .get_attributes()
        .iter()
        .find(|attr| attr.get_key() == required_attr_key)
        .map(|attr| attr.get_value().to_string())
        .ok_or_else(|| {
            EventIoError::InvalidMessage(format!(
                "required attribute not in event: {}",
                required_attr_key
            ))
        })
}

#[cfg(feature = "diesel")]
pub trait CommitStore: Send + Sync {
    /// Adds an commit to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `commit` - The commit to be added
    fn add_commit(&self, commit: &NewCommitModel) -> Result<(), CommitStoreError>;

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
    ) -> Result<Option<NewCommitModel>, CommitEventError>;
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
