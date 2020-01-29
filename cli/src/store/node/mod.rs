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

mod error;

pub use error::NodeStoreError;

pub struct Node {
    alias: String,
    endpoint: String,
}

impl Node {
    pub fn new(alias: &str, endpoint: &str) -> Node {
        Node {
            alias: alias.to_owned(),
            endpoint: endpoint.to_owned(),
        }
    }

    pub fn alias(&self) -> String {
        self.alias.to_owned()
    }

    pub fn endpoint(&self) -> String {
        self.endpoint.to_owned()
    }
}

pub trait NodeStore {
    /// Get node from the store
    fn get_node(&self, alias: &str) -> Result<Option<Node>, NodeStoreError>;

    /// List nodes from the store
    fn list_nodes(&self) -> Result<Vec<Node>, NodeStoreError>;

    /// Add nodes to the store
    fn add_node(&self, node: &Node) -> Result<(), NodeStoreError>;

    /// Delete node from the store
    fn delete_node(&self, alias: &str) -> Result<(), NodeStoreError>;
}
