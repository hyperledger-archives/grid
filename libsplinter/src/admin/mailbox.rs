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
use std::error::Error;
use std::fmt;
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
    pub fn iter<'a>(
        &'a self,
    ) -> Result<Box<(dyn Iterator<Item = (SystemTime, AdminServiceEvent)> + 'a)>, MailboxError>
    {
        Ok(Box::new(
            self.durable_set
                .iter()
                .map_err(|err| {
                    MailboxError::with_source(
                        "Unable to iterate over underlying storage",
                        Box::new(err),
                    )
                })?
                .map(|event| (event.timestamp, event.event)),
        ))
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
