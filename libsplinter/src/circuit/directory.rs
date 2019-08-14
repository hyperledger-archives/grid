// Copyright 2018 Cargill Incorporated
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

use crate::circuit::service::SplinterNode;
use crate::circuit::Circuit;

// State represents the persistant state of circuits that are connected to a node
// Includes the list of circuits and correlates the node id with their endpoints
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CircuitDirectory {
    nodes: BTreeMap<String, SplinterNode>,
    circuits: BTreeMap<String, Circuit>,

    #[serde(skip_serializing)]
    #[serde(default = "Circuit::new_admin")]
    admin_circuit: Circuit,
}

impl Default for CircuitDirectory {
    fn default() -> Self {
        CircuitDirectory::new()
    }
}

impl CircuitDirectory {
    pub fn new() -> Self {
        CircuitDirectory {
            nodes: BTreeMap::new(),
            circuits: BTreeMap::new(),
            admin_circuit: Circuit::new_admin(),
        }
    }

    pub fn add_node(&mut self, id: String, node: SplinterNode) {
        self.nodes.insert(id, node);
    }

    pub fn add_circuit(&mut self, name: String, circuit: Circuit) {
        self.circuits.insert(name, circuit);
    }

    pub fn remove_node(&mut self, id: &str) {
        self.nodes.remove(id);
    }

    pub fn remove_circuit(&mut self, name: &str) {
        self.circuits.remove(name);
    }

    pub fn nodes(&self) -> &BTreeMap<String, SplinterNode> {
        &self.nodes
    }

    pub fn node(&self, node_id: &str) -> Option<&SplinterNode> {
        self.nodes.get(node_id)
    }

    pub fn circuits(&self) -> &BTreeMap<String, Circuit> {
        &self.circuits
    }

    pub fn circuit(&self, circuit_name: &str) -> Option<&Circuit> {
        if circuit_name == self.admin_circuit.id() {
            Some(&self.admin_circuit)
        } else {
            self.circuits.get(circuit_name)
        }
    }

    pub fn has_circuit(&self, circuit_name: &str) -> bool {
        self.circuits.contains_key(circuit_name)
    }
}
