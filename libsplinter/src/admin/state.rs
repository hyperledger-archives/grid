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

use std::collections::HashMap;

use crossbeam_channel::Sender;

use crate::admin::messages::AdminServiceEvent;
use crate::network::peer::PeerConnector;
use crate::protos::admin::CircuitProposal;
use crate::service::ServiceNetworkSender;

pub struct AdminServiceState {
    pub open_proposals: HashMap<String, CircuitProposal>,
    pub peer_connector: PeerConnector,
    pub network_sender: Option<Box<dyn ServiceNetworkSender>>,
    pub socket_senders: Vec<(String, Sender<AdminServiceEvent>)>,
}

impl AdminServiceState {
    pub fn add_proposal(&mut self, circuit_proposal: CircuitProposal) {
        let circuit_id = circuit_proposal.get_circuit_id().to_string();

        self.open_proposals.insert(circuit_id, circuit_proposal);
    }

    pub fn has_proposal(&self, circuit_id: &str) -> bool {
        self.open_proposals.contains_key(circuit_id)
    }

    pub fn add_socket_sender(
        &mut self,
        circuit_management_type: String,
        sender: Sender<AdminServiceEvent>,
    ) {
        self.socket_senders.push((circuit_management_type, sender));
    }

    pub fn send_event(&mut self, circuit_management_type: &str, event: AdminServiceEvent) {
        // The use of retain allows us to drop any senders that are no longer valid.
        self.socket_senders.retain(|(mgmt_type, sender)| {
            if mgmt_type != circuit_management_type {
                return true;
            }

            if let Err(err) = sender.send(event.clone()) {
                warn!(
                    "Dropping sender for {} due to error: {}",
                    circuit_management_type, err
                );
                return false;
            }

            true
        });
    }
}
