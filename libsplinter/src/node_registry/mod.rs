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

use std::collections::HashMap;

pub use error::NodeRegistryError;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Node {
    /// The Splinter identity of the node.
    pub identity: String,
    /// A map with node metadata.
    pub metadata: HashMap<String, String>,
}

/// A predicate on a key/value pair in a Node's metadata table.
///
/// Each variant is an operator, and supplies a tuple representing a key/value pair. It is applied
/// by the comparison operator on the value found at the given key (the first item in the tuple)
/// against the predicate's value (the second item in the tuple).
///
/// If the item is missing in a node's metadata table, the predicate returns false.
#[derive(Clone)]
pub enum MetadataPredicate {
    /// Applies the `==` operator.
    Eq(String, String),
    /// Applies the `!=` operator.
    Ne(String, String),
    /// Applies the `>` operator.
    Gt(String, String),
    /// Applies the `>=` operator.
    Ge(String, String),
    /// Applies the `<` operator.
    Lt(String, String),
    /// Applies the `<=` operator.
    Le(String, String),
}

impl MetadataPredicate {
    /// Apply this predicate against a given node.
    pub fn apply(&self, node: &Node) -> bool {
        match self {
            MetadataPredicate::Eq(key, val) => {
                node.metadata.get(key).map(|v| v == val).unwrap_or(false)
            }
            MetadataPredicate::Ne(key, val) => {
                node.metadata.get(key).map(|v| v != val).unwrap_or(false)
            }
            MetadataPredicate::Gt(key, val) => {
                node.metadata.get(key).map(|v| v > val).unwrap_or(false)
            }
            MetadataPredicate::Ge(key, val) => {
                node.metadata.get(key).map(|v| v >= val).unwrap_or(false)
            }
            MetadataPredicate::Lt(key, val) => {
                node.metadata.get(key).map(|v| v < val).unwrap_or(false)
            }
            MetadataPredicate::Le(key, val) => {
                node.metadata.get(key).map(|v| v <= val).unwrap_or(false)
            }
        }
    }
}

/// Provides Node Registry read capabilities.
pub trait NodeRegistryReader: Send + Sync {
    /// Returns an iterator over the nodes in the registry.
    ///
    /// # Arguments
    ///
    /// * `predicates` - A list of of predicates to be applied to the resulting list. These are
    /// applied as an AND, from a query perspective. If the list is empty, it is the equivalent of
    /// no predicates (i.e. return all).
    fn list_nodes<'a, 'b: 'a>(
        &'b self,
        predicates: &'a [MetadataPredicate],
    ) -> Result<Box<dyn Iterator<Item = Node> + Send + 'a>, NodeRegistryError>;

    /// Returns the count of nodes in the registry.
    ///
    /// # Arguments
    ///
    /// * `predicates` - A list of of predicates to be applied before counting the nodes. These are
    /// applied as an AND, from a query perspective. If the list is empty, it is the equivalent of
    /// no predicates (i.e. return all).
    fn count_nodes(&self, predicates: &[MetadataPredicate]) -> Result<u32, NodeRegistryError>;

    /// Returns a node with the given identity.
    ///
    /// # Arguments
    ///
    ///  * `identity` - The Splinter identity of the node.
    ///
    fn fetch_node(&self, identity: &str) -> Result<Node, NodeRegistryError>;
}

/// Provides Node Registry write capabilities.
pub trait NodeRegistryWriter: Send + Sync {
    /// Registers a new node.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to be added to the registry.
    ///
    fn add_node(&self, node: Node) -> Result<(), NodeRegistryError>;

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
}

/// Provides a marker trait for a clonable, readable and writable Node Registry.
pub trait RwNodeRegistry: NodeRegistryWriter + NodeRegistryReader {
    /// Clone implementation for Box<NodeRegistry>.
    /// The implementation of Clone for NodeRegistry calls this method.
    ///
    /// # Example
    ///  fn clone_box(&self) -> Box<NodeRegistry> {
    ///     Box::new(Clone::clone(self))
    ///  }
    fn clone_box(&self) -> Box<dyn RwNodeRegistry>;
}

impl Clone for Box<dyn RwNodeRegistry> {
    fn clone(&self) -> Box<dyn RwNodeRegistry> {
        self.clone_box()
    }
}

impl<NR> NodeRegistryReader for Box<NR>
where
    NR: NodeRegistryReader + ?Sized,
{
    fn list_nodes<'a, 'b: 'a>(
        &'b self,
        predicates: &'a [MetadataPredicate],
    ) -> Result<Box<dyn Iterator<Item = Node> + Send + 'a>, NodeRegistryError> {
        (**self).list_nodes(predicates)
    }

    fn count_nodes(&self, predicates: &[MetadataPredicate]) -> Result<u32, NodeRegistryError> {
        (**self).count_nodes(predicates)
    }

    fn fetch_node(&self, identity: &str) -> Result<Node, NodeRegistryError> {
        (**self).fetch_node(identity)
    }
}

impl<NW> NodeRegistryWriter for Box<NW>
where
    NW: NodeRegistryWriter + ?Sized,
{
    fn add_node(&self, node: Node) -> Result<(), NodeRegistryError> {
        (**self).add_node(node)
    }

    fn update_node(
        &self,
        identity: &str,
        updates: HashMap<String, String>,
    ) -> Result<(), NodeRegistryError> {
        (**self).update_node(identity, updates)
    }

    fn delete_node(&self, identity: &str) -> Result<(), NodeRegistryError> {
        (**self).delete_node(identity)
    }
}
