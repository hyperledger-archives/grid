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

//! Provides an empty-list implemenation of the NodeRegistry trait.

use std::collections::HashMap;

use super::{Node, NodeRegistry, NodeRegistryError};

/// The NoOpNodeRegistry is an empty-list implementation of the NodeRegistry trait.
///
/// This implemenation returns an empty list of nodes, and NotFound for operations on individual
/// nodes.  It does not allow node creation.
pub struct NoOpNodeRegistry;

impl NodeRegistry for NoOpNodeRegistry {
    fn create_node(
        &self,
        _identity: &str,
        _data: HashMap<String, String>,
    ) -> Result<(), NodeRegistryError> {
        Err(NodeRegistryError::UnableToCreateNode(
            "operation not supported".into(),
            None,
        ))
    }

    fn list_nodes(
        &self,
        _filters: Option<HashMap<String, (String, String)>>,
        _limit: Option<usize>,
        _offset: Option<usize>,
    ) -> Result<Vec<Node>, NodeRegistryError> {
        Ok(vec![])
    }

    fn fetch_node(&self, identity: &str) -> Result<Node, NodeRegistryError> {
        Err(NodeRegistryError::NotFoundError(identity.to_string()))
    }

    fn update_node(
        &self,
        identity: &str,
        _updates: HashMap<String, String>,
    ) -> Result<(), NodeRegistryError> {
        Err(NodeRegistryError::NotFoundError(identity.to_string()))
    }

    fn delete_node(&self, identity: &str) -> Result<(), NodeRegistryError> {
        Err(NodeRegistryError::NotFoundError(identity.to_string()))
    }

    fn clone_box(&self) -> Box<dyn NodeRegistry> {
        Box::new(NoOpNodeRegistry)
    }
}
