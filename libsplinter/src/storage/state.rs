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

use circuits::circuit::Circuit;
use circuits::service::SplinterNode;
use std::collections::BTreeMap;

// State represents the persistant state of circuits that are connected to a node
// Includes the list of circuits and correlates the node id with their endpoints
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct State {
    nodes: BTreeMap<String, SplinterNode>,
    circuits: BTreeMap<String, Circuit>,
}

impl State {
    pub fn new() -> Self {
        State {
            nodes: BTreeMap::new(),
            circuits: BTreeMap::new(),
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

    pub fn circuits(&self) -> &BTreeMap<String, Circuit> {
        &self.circuits
    }
}
