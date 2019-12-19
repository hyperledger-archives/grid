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

//! Unified NodeRegistry implementations.
//!
//! This module provides a unified node registry which combines the node data from one or more
//! read-only node registries with one local read-write node registry.  The data is merged from the
//! local source into values from the read-only sources, allowing the user to replace values from
//! the remove sources.
//!
//! This module is behind the `"node-registry-unified"` feature, and is considered experimental.

use std::sync::Arc;

use super::{
    MetadataPredicate, Node, NodeRegistryError, NodeRegistryReader, NodeRegistryWriter,
    RwNodeRegistry,
};

/// Unifies a set of read-only node registries with a local, read-write node registry.
///
/// Nodes read from the unified registry utilize the read-only sources to fetch node definitions
/// and any local changes as a replacement.
#[derive(Clone)]
pub struct UnifiedNodeRegistry {
    local_source: Arc<dyn RwNodeRegistry>,
    readable_sources: Vec<Arc<dyn NodeRegistryReader>>,
}

impl UnifiedNodeRegistry {
    /// Constructs a new UnifiedNodeRegistry with a local, read-write node registry and a
    /// arbitrary number of read-only node registries.
    pub fn new(
        local_source: Box<dyn RwNodeRegistry>,
        readable_sources: Vec<Box<dyn NodeRegistryReader>>,
    ) -> Self {
        Self {
            local_source: local_source.into(),
            readable_sources: readable_sources.into_iter().map(Arc::from).collect(),
        }
    }
}

// Some type conveniences to cleanup some of the type requirements in the list_nodes implementation
type NodeIter<'a> = Box<dyn Iterator<Item = Node> + Send + 'a>;

impl NodeRegistryReader for UnifiedNodeRegistry {
    fn list_nodes<'a, 'b: 'a>(
        &'b self,
        predicates: &'a [MetadataPredicate],
    ) -> Result<NodeIter<'a>, NodeRegistryError> {
        self.readable_sources
            .iter()
            .map(|registry| registry.list_nodes(predicates))
            .fold(self.local_source.list_nodes(predicates), |acc, iter| {
                let local_source = self.local_source.clone();
                acc.and_then(|chained| {
                    let res: NodeIter<'a> = Box::new(chained.chain(iter?.filter(move |node| {
                        match local_source.has_node(&node.identity) {
                            Ok(exists) => !exists,
                            Err(err) => {
                                error!(
                                    "unable to load local entry for {}; using read-only copy: {}",
                                    node.identity, err
                                );
                                false
                            }
                        }
                    })));
                    Ok(res)
                })
            })
    }

    /// This implementation of count_nodes does not take into account the replaced nodes in the
    /// node registry, in order to keep the overall operation more efficient.
    fn count_nodes(&self, predicates: &[MetadataPredicate]) -> Result<u32, NodeRegistryError> {
        let local_source_count = self.local_source.count_nodes(predicates)?;

        self.readable_sources
            .iter()
            .map(|source| source.count_nodes(predicates))
            .fold(Ok(local_source_count), |acc, count| {
                acc.and_then(|total| Ok(total + count?))
            })
    }

    fn fetch_node(&self, identity: &str) -> Result<Node, NodeRegistryError> {
        match self.local_source.fetch_node(identity) {
            Ok(node) => return Ok(node),
            Err(NodeRegistryError::NotFoundError(_)) => (),
            Err(err) => return Err(err),
        }

        self.readable_sources
            .iter()
            .map(|source| source.fetch_node(identity))
            .filter(|res| match res {
                Err(NodeRegistryError::NotFoundError(_)) => false,
                _ => true,
            })
            .find(Result::is_ok)
            .unwrap_or_else(|| Err(NodeRegistryError::NotFoundError(identity.to_string())))
    }
}

impl NodeRegistryWriter for UnifiedNodeRegistry {
    fn insert_node(&self, node: Node) -> Result<(), NodeRegistryError> {
        self.local_source.insert_node(node)
    }

    fn delete_node(&self, identity: &str) -> Result<(), NodeRegistryError> {
        self.local_source.delete_node(identity)
    }
}

impl RwNodeRegistry for UnifiedNodeRegistry {
    fn clone_box(&self) -> Box<dyn RwNodeRegistry> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};

    use super::*;

    /// Simple macro for creating nodes
    macro_rules! node {
        ($identity:expr, $($key:expr => $val:expr),*) => {
            {
                let mut node = Node::new($identity, "test://example.com");

                $(
                    node.metadata.insert($key.into(), $val.into());
                )*

                node
            }
        };
    }

    /// This test ensures that the resulting fetched node is returned from a read-only source, if
    /// it only exists there.
    #[test]
    fn test_unified_fetch_node_readable() {
        let readable = MemRegistry::default();
        readable
            .insert_node(node!("node1", "meta_a" => "a value"))
            .expect("Unable to insert node");

        let writable = MemRegistry::default();

        let unified = UnifiedNodeRegistry::new(Box::new(writable), vec![Box::new(readable)]);

        let retreived_node = unified.fetch_node("node1").expect("Unable to fetch node");

        assert_eq!(node!("node1", "meta_a" => "a value"), retreived_node);
    }

    /// This test ensures that the resulting fetched node is returned from the local source, if it
    /// only exists there.
    #[test]
    fn test_unified_fetch_node_local() {
        let readable = MemRegistry::default();

        let writable = MemRegistry::default();
        writable
            .insert_node(node!("node1", "meta_b" => "b value"))
            .expect("Unable to insert node");

        let unified = UnifiedNodeRegistry::new(Box::new(writable), vec![Box::new(readable)]);

        let retreived_node = unified.fetch_node("node1").expect("Unable to fetch node");

        assert_eq!(node!("node1", "meta_b" => "b value"), retreived_node);
    }

    /// This test ensures that the resulting fetched node is from the local store, even if a node
    /// is in a read-only source.
    #[test]
    fn test_unified_fetch_node_local_selected() {
        let readable = MemRegistry::default();
        readable
            .insert_node(node!("node1", "meta_a" => "a value"))
            .expect("Unable to insert node");

        let writable = MemRegistry::default();
        writable
            .insert_node(node!("node1", "meta_b" => "b value"))
            .expect("Unable to insert node");

        let unified = UnifiedNodeRegistry::new(Box::new(writable), vec![Box::new(readable)]);

        let retreived_node = unified.fetch_node("node1").expect("Unable to fetch node");

        assert_eq!(node!("node1", "meta_b" => "b value"), retreived_node);
    }

    /// This test ensures that the results are unified into a single iterator and nodes are
    /// replaced with the values from the local store, if the exist.
    #[test]
    fn test_unified_iteration_local_selected() {
        let readable = MemRegistry::default();
        readable
            .insert_node(node!("node1", "meta_a" => "a value"))
            .expect("Unable to insert node");
        readable
            .insert_node(node!("node2", "meta_c" => "c value"))
            .expect("Unable to insert node");

        let writable = MemRegistry::default();
        writable
            .insert_node(node!("node1", "meta_b" => "b value"))
            .expect("Unable to insert node");

        let unified = UnifiedNodeRegistry::new(Box::new(writable), vec![Box::new(readable)]);

        let mut iterator = unified.list_nodes(&[]).expect("Unable to list nodes");

        assert_eq!(Some(node!("node1", "meta_b" => "b value")), iterator.next());
        assert_eq!(Some(node!("node2", "meta_c" => "c value")), iterator.next());

        assert_eq!(None, iterator.next());
    }

    /// This test ensures that a node which is selected by MetadataPredicates is returned
    /// regardless of source.
    ///
    /// Likewise, nodes that should be filtered out based on properties in one source should not be
    /// returned in the final iteration.
    #[test]
    fn test_unified_iteration_filtering() {
        let readable = MemRegistry::default();
        readable
            .insert_node(node!("node1", "meta_a" => "a value"))
            .expect("Unable to insert node");
        readable
            .insert_node(node!("node2", "meta_c" => "c value"))
            .expect("Unable to insert node");

        let writable = MemRegistry::default();
        writable
            .insert_node(node!("node1", "meta_b" => "b value"))
            .expect("Unable to insert node");

        let unified = UnifiedNodeRegistry::new(Box::new(writable), vec![Box::new(readable)]);

        let predicates = vec![MetadataPredicate::eq("meta_b", "b value")];
        let mut iterator = unified
            .list_nodes(&predicates)
            .expect("Unable to list nodes");

        assert_eq!(Some(node!("node1", "meta_b" => "b value")), iterator.next());

        assert_eq!(None, iterator.next());

        let predicates = vec![MetadataPredicate::ne("meta_b", "b value")];
        let mut iterator = unified
            .list_nodes(&predicates)
            .expect("Unable to list nodes");

        assert_eq!(Some(node!("node2", "meta_c" => "c value")), iterator.next());

        assert_eq!(None, iterator.next());
    }

    #[derive(Clone, Default)]
    struct MemRegistry {
        nodes: Arc<Mutex<BTreeMap<String, Node>>>,
    }

    impl NodeRegistryReader for MemRegistry {
        fn list_nodes<'a, 'b: 'a>(
            &'b self,
            predicates: &'a [MetadataPredicate],
        ) -> Result<Box<dyn Iterator<Item = Node> + Send + 'a>, NodeRegistryError> {
            Ok(Box::new(SnapShotIter {
                snapshot: self
                    .nodes
                    .lock()
                    .expect("mem registry lock was poisoned")
                    .iter()
                    .map(|(_, node)| node)
                    .filter(move |node| predicates.iter().all(|predicate| predicate.apply(node)))
                    .cloned()
                    .collect(),
            }))
        }

        fn count_nodes(&self, predicates: &[MetadataPredicate]) -> Result<u32, NodeRegistryError> {
            self.list_nodes(predicates).map(|iter| iter.count() as u32)
        }

        fn fetch_node(&self, identity: &str) -> Result<Node, NodeRegistryError> {
            self.nodes
                .lock()
                .expect("mem registry lock was poisoned")
                .get(identity)
                .cloned()
                .ok_or_else(|| NodeRegistryError::NotFoundError(identity.to_string()))
        }
    }

    impl NodeRegistryWriter for MemRegistry {
        fn insert_node(&self, node: Node) -> Result<(), NodeRegistryError> {
            self.nodes
                .lock()
                .expect("mem registry lock was poisoned")
                .insert(node.identity.clone(), node);
            Ok(())
        }

        fn delete_node(&self, identity: &str) -> Result<(), NodeRegistryError> {
            self.nodes
                .lock()
                .expect("mem registry lock was poisoned")
                .remove(identity);
            Ok(())
        }
    }

    impl RwNodeRegistry for MemRegistry {
        fn clone_box(&self) -> Box<dyn RwNodeRegistry> {
            Box::new(self.clone())
        }
    }

    struct SnapShotIter<V: Send + Clone> {
        snapshot: std::collections::VecDeque<V>,
    }

    impl<V: Send + Clone> Iterator for SnapShotIter<V> {
        type Item = V;

        fn next(&mut self) -> Option<Self::Item> {
            self.snapshot.pop_front()
        }
    }
}
