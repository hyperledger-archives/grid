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

use std::collections::{HashMap, HashSet, VecDeque};

use transact::protocol::batch::BatchPair;

use crate::consensus::ProposalId;
use crate::service::ServiceNetworkSender;

use super::state::ScabbardState;

/// Data structure used to store information that's shared between components in this service
pub struct ScabbardShared {
    /// Queue of batches that have been submitted locally via the REST API, but have not yet been
    /// proposed.
    batch_queue: VecDeque<BatchPair>,
    /// Used to send messages to other services; set when the service is started and unset when the
    /// service is stopped.
    network_sender: Option<Box<dyn ServiceNetworkSender>>,
    /// List of service IDs that this service is configured to communicate and share state with.
    peer_services: HashSet<String>,
    /// Tracks which batches are currently being evaluated, indexed by corresponding proposal IDs.
    proposed_batches: HashMap<ProposalId, BatchPair>,
    state: ScabbardState,
}

impl ScabbardShared {
    pub fn new(
        batch_queue: VecDeque<BatchPair>,
        network_sender: Option<Box<dyn ServiceNetworkSender>>,
        peer_services: HashSet<String>,
        state: ScabbardState,
    ) -> Self {
        ScabbardShared {
            batch_queue,
            network_sender,
            peer_services,
            proposed_batches: HashMap::new(),
            state,
        }
    }

    pub fn add_batch_to_queue(&mut self, batch: BatchPair) {
        self.batch_queue.push_back(batch)
    }

    pub fn pop_batch_from_queue(&mut self) -> Option<BatchPair> {
        self.batch_queue.pop_front()
    }

    pub fn network_sender(&self) -> Option<&dyn ServiceNetworkSender> {
        self.network_sender.as_ref().map(|b| &**b)
    }

    pub fn set_network_sender(&mut self, sender: Box<dyn ServiceNetworkSender>) {
        self.network_sender = Some(sender)
    }

    pub fn take_network_sender(&mut self) -> Option<Box<dyn ServiceNetworkSender>> {
        self.network_sender.take()
    }

    pub fn peer_services(&self) -> &HashSet<String> {
        &self.peer_services
    }

    pub fn add_proposed_batch(
        &mut self,
        proposal_id: ProposalId,
        batch: BatchPair,
    ) -> Option<BatchPair> {
        self.proposed_batches.insert(proposal_id, batch)
    }

    pub fn get_proposed_batch(&self, proposal_id: &ProposalId) -> Option<&BatchPair> {
        self.proposed_batches.get(proposal_id)
    }

    pub fn remove_proposed_batch(&mut self, proposal_id: &ProposalId) -> Option<BatchPair> {
        self.proposed_batches.remove(&proposal_id)
    }

    pub fn state_mut(&mut self) -> &mut ScabbardState {
        &mut self.state
    }
}
