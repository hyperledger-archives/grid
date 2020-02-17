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

use std::collections::BTreeMap;

use serde_derive::{Deserialize, Serialize};

use crate::protos::admin::CircuitProposal;
use crate::storage::get_storage;

use super::error::OpenProposalError;
use super::messages;

pub struct OpenProposals {
    storage_location: String,
    proposal_registry: ProposalRegistry,
}

impl OpenProposals {
    /// Constructs a new OpenProposals using the given location.
    ///
    /// # Errors
    ///
    /// Returns a `OpenProposalError` if the persisted registry fails to load.
    pub fn new(storage_location: String) -> Result<Self, OpenProposalError> {
        let proposal_registry = get_storage(&storage_location, ProposalRegistry::default)
            .map_err(OpenProposalError::WriteError)?
            .read()
            .clone();

        Ok(Self {
            storage_location,
            proposal_registry,
        })
    }

    pub fn add_proposal(
        &mut self,
        circuit_proposal: CircuitProposal,
    ) -> Result<Option<CircuitProposal>, OpenProposalError> {
        let proposal = self.proposal_registry.add_proposal(circuit_proposal);
        self.write_open_proposals()?;

        proposal
    }

    pub fn remove_proposal(
        &mut self,
        circuit_id: &str,
    ) -> Result<Option<CircuitProposal>, OpenProposalError> {
        let proposal = self.proposal_registry.remove_proposal(circuit_id);
        self.write_open_proposals()?;

        proposal
    }

    pub fn get_proposal(
        &self,
        circuit_id: &str,
    ) -> Result<Option<CircuitProposal>, OpenProposalError> {
        self.proposal_registry.get_proposal(circuit_id)
    }

    pub fn get_proposals(&self) -> Proposals {
        self.proposal_registry.get_proposals()
    }

    pub fn has_proposal(&self, circuit_id: &str) -> bool {
        self.proposal_registry.has_proposal(circuit_id)
    }

    pub fn storage_location(&self) -> &str {
        &self.storage_location
    }

    fn write_open_proposals(&self) -> Result<(), OpenProposalError> {
        // Replace stored key_registry with the current key registry
        let mut storage = get_storage(self.storage_location(), || self.proposal_registry.clone())
            .map_err(OpenProposalError::WriteError)?;

        // when this is dropped the new state will be written to storage
        **storage.write() = self.proposal_registry.clone();
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct ProposalRegistry {
    proposals: BTreeMap<String, messages::CircuitProposal>,
}

impl ProposalRegistry {
    pub fn add_proposal(
        &mut self,
        circuit_proposal: CircuitProposal,
    ) -> Result<Option<CircuitProposal>, OpenProposalError> {
        let circuit_id = circuit_proposal.get_circuit_id().to_string();
        let proposal = messages::CircuitProposal::from_proto(circuit_proposal)?;
        let proposal = self.proposals.insert(circuit_id, proposal);
        match proposal {
            Some(circuit_proposal) => Ok(Some(circuit_proposal.into_proto()?)),
            None => Ok(None),
        }
    }

    pub fn remove_proposal(
        &mut self,
        circuit_id: &str,
    ) -> Result<Option<CircuitProposal>, OpenProposalError> {
        let proposal = self.proposals.remove(circuit_id);
        match proposal {
            Some(circuit_proposal) => Ok(Some(circuit_proposal.into_proto()?)),
            None => Ok(None),
        }
    }

    pub fn get_proposal(
        &self,
        circuit_id: &str,
    ) -> Result<Option<CircuitProposal>, OpenProposalError> {
        match self.proposals.get(circuit_id) {
            Some(circuit_proposal) => Ok(Some(circuit_proposal.clone().into_proto()?)),
            None => Ok(None),
        }
    }

    pub fn get_proposals(&self) -> Proposals {
        Proposals {
            inner: Box::new(self.proposals.clone().into_iter()),
            size: self.proposals.len(),
        }
    }

    pub fn has_proposal(&self, circuit_id: &str) -> bool {
        self.proposals.contains_key(circuit_id)
    }
}

/// An iterator over CircuitProposals and the time that each occurred.
pub struct Proposals {
    inner: Box<dyn Iterator<Item = (String, messages::CircuitProposal)>>,
    size: usize,
}

impl Proposals {
    pub fn total(&self) -> usize {
        self.size
    }
}

impl Iterator for Proposals {
    type Item = (String, messages::CircuitProposal);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}
