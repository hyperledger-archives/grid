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

// State represents the persistant state of circuits that are connected to a node
// Includes the list of circuits and correlates the node id with their endpoints
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct State {
    nodes: BTreeMap<String, StateNode>,
    circuits: BTreeMap<String, StateCircuit>,
}

impl State {
    pub fn new() -> Self {
        State {
            nodes: BTreeMap::new(),
            circuits: BTreeMap::new(),
        }
    }

    pub fn add_node(&mut self, id: String, node: StateNode) {
        self.nodes.insert(id, node);
    }

    pub fn add_circuit(&mut self, name: String, circuit: StateCircuit) {
        self.circuits.insert(name, circuit);
    }

    pub fn remove_node(&mut self, id: &str) {
        self.nodes.remove(id);
    }

    pub fn remove_circuit(&mut self, name: &str) {
        self.circuits.remove(name);
    }

    pub fn nodes(&self) -> &BTreeMap<String, StateNode> {
        &self.nodes
    }

    pub fn circuits(&self) -> &BTreeMap<String, StateCircuit> {
        &self.circuits
    }
}

// StateNode represents the node in persistent state and has the endpoints it is avaliable on
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct StateNode {
    endpoints: Vec<String>,
}

impl StateNode {
    pub fn new(endpoints: Vec<String>) -> Self {
        StateNode {
            endpoints: endpoints,
        }
    }

    pub fn endpoints(&self) -> &[String] {
        &self.endpoints
    }
}

// StateCircuit represent a circuit including the memebers, services and circuit configuration
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct StateCircuit {
    auth: String,
    members: Vec<String>,
    services: Vec<String>,
    persistence: String,
    durability: String,
    routes: String,
}

impl StateCircuit {
    pub fn new(
        auth: String,
        members: Vec<String>,
        services: Vec<String>,
        persistence: String,
        durability: String,
        routes: String,
    ) -> Self {
        StateCircuit {
            auth,
            members,
            services,
            persistence,
            durability,
            routes,
        }
    }

    pub fn auth(&self) -> &str {
        &self.auth
    }

    pub fn members(&self) -> &[String] {
        &self.members
    }

    pub fn services(&self) -> &[String] {
        &self.services
    }

    pub fn persistence(&self) -> &str {
        &self.persistence
    }

    pub fn durability(&self) -> &str {
        &self.durability
    }

    pub fn routes(&self) -> &str {
        &self.routes
    }
}
