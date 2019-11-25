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

use std::cmp;
use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::ops::Bound;
use std::time::SystemTime;

use crate::storage::sets::DurableOrderedSet;

use super::messages::AdminServiceEvent;

/// A simple entry for AdminServiceEvent values, marked with a timestamp
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct EventEntry {
    timestamp: SystemTime,
    event: AdminServiceEvent,
}

impl cmp::Ord for EventEntry {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl cmp::PartialOrd for EventEntry {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::borrow::Borrow<SystemTime> for EventEntry {
    fn borrow(&self) -> &SystemTime {
        &self.timestamp
    }
}

/// A Mailbox stores all admin services events that have occurred, ordered by a timestamp generated
/// upon addition to the mailbox.
///
/// These events are stored in a durable ordered set, determined by the caller.
#[derive(Clone)]
pub struct Mailbox {
    durable_set: Box<dyn DurableOrderedSet<EventEntry, SystemTime>>,
}

impl Mailbox {
    /// Constructs a new event mailbox with the given backing store.
    pub fn new(durable_set: Box<dyn DurableOrderedSet<EventEntry, SystemTime>>) -> Self {
        Self { durable_set }
    }

    /// Add an event to the mailbox.  Returns the recorded event time and a copy of the event.
    ///
    /// # Errors
    ///
    /// Returns a MailboxError if there is an issue with the underlying storage set.
    pub fn add(
        &mut self,
        event: AdminServiceEvent,
    ) -> Result<(SystemTime, AdminServiceEvent), MailboxError> {
        let entry = EventEntry {
            timestamp: SystemTime::now(),
            event,
        };
        self.durable_set.add(entry.clone()).map_err(|err| {
            MailboxError::with_source("Unable to add event to storage", Box::new(err))
        })?;

        Ok((entry.timestamp, entry.event))
    }

    /// Returns an iterator over all of the values in the mailbox.
    pub fn iter(&self) -> Result<MailboxIter, MailboxError> {
        MailboxIter::new(
            self.durable_set.clone(),
            SystemTime::UNIX_EPOCH,
            SystemTime::now(),
        )
    }
}

const ITER_CACHE_SIZE: usize = 100;

pub struct MailboxIter {
    source: Box<dyn DurableOrderedSet<EventEntry, SystemTime>>,
    start_search: SystemTime,
    end_search: SystemTime,
    cache: VecDeque<EventEntry>,
}

impl MailboxIter {
    fn new(
        source: Box<dyn DurableOrderedSet<EventEntry, SystemTime>>,
        start_search: SystemTime,
        end_search: SystemTime,
    ) -> Result<Self, MailboxError> {
        let initial_cache = source
            .range_iter((&start_search..&end_search).into())
            .map_err(|err| {
                MailboxError::with_source(
                    "Unable to iterate over underlying storage",
                    Box::new(err),
                )
            })?
            .take(ITER_CACHE_SIZE)
            .collect::<VecDeque<_>>();

        let start_search = if initial_cache.is_empty() {
            end_search
        } else {
            initial_cache.back().unwrap().timestamp
        };

        Ok(Self {
            source,
            start_search,
            end_search,
            cache: initial_cache,
        })
    }

    fn reload_cache(&mut self) -> Result<(), MailboxError> {
        self.cache = self
            .source
            .range_iter(
                (
                    Bound::Excluded(&self.start_search),
                    Bound::Excluded(&self.end_search),
                )
                    .into(),
            )
            .map_err(|err| {
                MailboxError::with_source(
                    "Unable to iterate over underlying storage",
                    Box::new(err),
                )
            })?
            .take(ITER_CACHE_SIZE)
            .collect();

        self.start_search = if self.cache.is_empty() {
            self.end_search
        } else {
            self.cache.back().unwrap().timestamp
        };

        Ok(())
    }
}

impl Iterator for MailboxIter {
    type Item = (SystemTime, AdminServiceEvent);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cache.is_empty() && self.start_search < self.end_search {
            if let Err(err) = self.reload_cache() {
                error!("Unable to load iterator cache: {}", err);
            }
        }

        self.cache
            .pop_front()
            .map(|event| (event.timestamp, event.event))
    }
}

#[derive(Debug)]
pub struct MailboxError {
    pub context: String,
    pub source: Option<Box<dyn Error + Send>>,
}

impl MailboxError {
    fn with_source(context: &str, source: Box<dyn Error + Send>) -> Self {
        Self {
            context: context.into(),
            source: Some(source),
        }
    }
}

impl Error for MailboxError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Some(ref err) = self.source {
            Some(&**err)
        } else {
            None
        }
    }
}

impl fmt::Display for MailboxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref err) = self.source {
            write!(f, "{}: {}", self.context, err)
        } else {
            f.write_str(&self.context)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::admin::messages::{self, AdminServiceEvent, CircuitProposal, ProposalType};
    use crate::storage::sets::mem::DurableBTreeSet;

    use super::*;

    #[test]
    fn test_iterate() {
        let mut mailbox = Mailbox::new(DurableBTreeSet::new_boxed());

        mailbox
            .add(make_event("circuit_one", "default"))
            .expect("Unable to add event");
        mailbox
            .add(make_event("gameroom_one", "gameroom"))
            .expect("Unable to add event");
        mailbox
            .add(make_event("circuit_two", "default"))
            .expect("Unable to add event");

        assert_eq!(
            vec![
                make_event("circuit_one", "default"),
                make_event("gameroom_one", "gameroom"),
                make_event("circuit_two", "default"),
            ],
            mailbox
                .iter()
                .expect("Unable to create an iterator")
                .map(|(_, evt)| evt)
                .collect::<Vec<_>>(),
        );
    }

    fn make_event(circuit_id: &str, event_type: &str) -> AdminServiceEvent {
        AdminServiceEvent::ProposalSubmitted(CircuitProposal {
            proposal_type: ProposalType::Create,
            circuit_id: circuit_id.into(),
            circuit_hash: "not real hash for tests".into(),
            circuit: messages::CreateCircuit {
                circuit_id: circuit_id.into(),
                roster: vec![],
                members: vec![],
                authorization_type: messages::AuthorizationType::Trust,
                persistence: messages::PersistenceType::Any,
                durability: messages::DurabilityType::NoDurability,
                routes: messages::RouteType::Any,
                circuit_management_type: event_type.into(),
                application_metadata: vec![],
            },
            votes: vec![],
            requester: vec![],
            requester_node_id: "another-node".into(),
        })
    }
}
