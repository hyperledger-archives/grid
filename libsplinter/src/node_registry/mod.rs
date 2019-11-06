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

pub mod error;
pub mod noop;
pub mod yaml;

pub use error::NodeRegistryError;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Node {
    /// The Splinter identity of the node.
    pub identity: String,
    /// A map with node metadata.
    pub metadata: HashMap<String, String>,
}

pub trait NodeRegistry: Send + Sync {
    /// Registers a new node.
    ///
    /// # Arguments
    ///
    /// * `identity` - The Splinter ID of the node.
    /// * `data` - A map with node metadata.
    ///
    fn create_node(
        &self,
        identity: &str,
        data: HashMap<String, String>,
    ) -> Result<(), NodeRegistryError>;

    /// Returns a list of nodes.
    ///
    /// # Arguments
    ///
    /// * `filters` - A map that defines list filters. The key is the property to be filtered by
    /// and the value is a tuple. The first item of the tuple defines the operator "=", "<", ">",
    /// "<=" or "<=". The second item in the tuple is the value to compare the node property
    /// against. If the filters map has more than one key-value pair, this function should return
    /// only nodes that match all the provided filters.
    ///
    /// * `limit` - The maximum number of items to return
    ///
    /// * `offset` - The index of the resource to start the resulting array
    fn list_nodes(
        &self,
        filters: Option<HashMap<String, (String, String)>>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Node>, NodeRegistryError>;

    /// Returns a node with the given identity.
    ///
    /// # Arguments
    ///
    ///  * `identity` - The Splinter identity of the node.
    ///
    fn fetch_node(&self, identity: &str) -> Result<Node, NodeRegistryError>;

    /// Updates a node with the given identity.
    /// The node's exiting metadata properties that are not in the updates map will not be
    /// changed. New properties that are not already in the nodes's metadata will be added to
    /// the metadata.
    ///
    /// # Arguments
    ///
    ///  * `identity` - The Splinter identity of the node.
    ///  * `updates` - A map containing the updated properties.
    ///
    fn update_node(
        &self,
        identity: &str,
        updates: HashMap<String, String>,
    ) -> Result<(), NodeRegistryError>;

    /// Deletes a node with the given identity.
    ///
    /// # Arguments
    ///
    ///  * `identity` - The Splinter identity of the node.
    ///
    fn delete_node(&self, identity: &str) -> Result<(), NodeRegistryError>;

    /// Clone implementation for Box<NodeRegistry>.
    /// The implementation of Clone for NodeRegistry calls this method.
    ///
    /// # Example
    ///  fn clone_box(&self) -> Box<NodeRegistry> {
    ///     Box::new(Clone::clone(self))
    ///  }
    fn clone_box(&self) -> Box<dyn NodeRegistry>;
}

impl Clone for Box<dyn NodeRegistry> {
    fn clone(&self) -> Box<dyn NodeRegistry> {
        self.clone_box()
    }
}
