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

//! Provides an empty-list implemenation of the NodeRegistry trait.

use super::{
    MetadataPredicate, Node, NodeRegistryError, NodeRegistryReader, NodeRegistryWriter,
    RwNodeRegistry,
};

/// The NoOpNodeRegistry is an empty-list implementation of the NodeRegistry trait.
///
/// This implemenation returns an empty list of nodes, and NotFound for operations on individual
/// nodes.  It does not allow node creation.
pub struct NoOpNodeRegistry;

impl NodeRegistryReader for NoOpNodeRegistry {
    fn list_nodes<'a, 'b: 'a>(
        &'b self,
        _predicates: &'a [MetadataPredicate],
    ) -> Result<Box<dyn Iterator<Item = Node> + Send + 'a>, NodeRegistryError> {
        Ok(Box::new(std::iter::empty()))
    }

    fn fetch_node(&self, identity: &str) -> Result<Node, NodeRegistryError> {
        Err(NodeRegistryError::NotFoundError(identity.to_string()))
    }

    fn count_nodes(&self, _predicates: &[MetadataPredicate]) -> Result<u32, NodeRegistryError> {
        Ok(0)
    }
}

impl NodeRegistryWriter for NoOpNodeRegistry {
    fn insert_node(&self, _node: Node) -> Result<(), NodeRegistryError> {
        Err(NodeRegistryError::UnableToAddNode(
            "operation not supported".into(),
            None,
        ))
    }

    fn delete_node(&self, identity: &str) -> Result<(), NodeRegistryError> {
        Err(NodeRegistryError::NotFoundError(identity.to_string()))
    }
}

impl RwNodeRegistry for NoOpNodeRegistry {
    fn clone_box(&self) -> Box<dyn RwNodeRegistry> {
        Box::new(NoOpNodeRegistry)
    }
}
