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

mod error;

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};

use super::{Node, NodeRegistry, NodeRegistryError};

use error::YamlNodeRegistryError;

#[derive(Clone)]
pub struct YamlNodeRegistry {
    file_internal: Arc<Mutex<FileInternal>>,
}

pub struct FileInternal {
    pub file: File,
    pub cached_nodes: Vec<Node>,
}

impl YamlNodeRegistry {
    pub fn new(file_path: &str) -> Result<YamlNodeRegistry, YamlNodeRegistryError> {
        let file = OpenOptions::new().read(true).write(true).open(file_path)?;

        let nodes = serde_yaml::from_reader(&file)?;

        let file_internal = FileInternal {
            file,
            cached_nodes: nodes,
        };

        Ok(YamlNodeRegistry {
            file_internal: Arc::new(Mutex::new(file_internal)),
        })
    }

    fn get_cached_nodes(&self) -> Result<Vec<Node>, YamlNodeRegistryError> {
        let file_backend = self
            .file_internal
            .lock()
            .map_err(|err| YamlNodeRegistryError::PoisonLockError(format!("{}", err)))?;
        Ok(file_backend.cached_nodes.clone())
    }

    fn write_nodes(&self, data: &[Node]) -> Result<(), YamlNodeRegistryError> {
        let mut file_backend = self
            .file_internal
            .lock()
            .map_err(|err| YamlNodeRegistryError::PoisonLockError(format!("{}", err)))?;
        let output = serde_yaml::to_string(&data)?;
        file_backend.file.write_all(&output.into_bytes())?;
        file_backend.cached_nodes = data.to_vec();
        Ok(())
    }
}

impl NodeRegistry for YamlNodeRegistry {
    fn add_node(&self, node: Node) -> Result<(), NodeRegistryError> {
        let mut nodes = self
            .get_cached_nodes()
            .map_err(|err| NodeRegistryError::InternalError(Box::new(err)))?;
        if nodes
            .iter()
            .any(|existing_node| existing_node.identity == node.identity)
        {
            return Err(NodeRegistryError::DuplicateNodeError(format!(
                "Node with ID {} already exists",
                node.identity,
            )));
        }

        nodes.push(node);

        self.write_nodes(&nodes)
            .map_err(|err| NodeRegistryError::InternalError(Box::new(err)))
    }

    fn list_nodes(
        &self,
        filters: Option<HashMap<String, (String, String)>>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Node>, NodeRegistryError> {
        let nodes = self
            .get_cached_nodes()
            .map_err(|err| NodeRegistryError::InternalError(Box::new(err)))?;
        let offset_value = offset.unwrap_or(0);
        let limit_value = limit.unwrap_or_else(|| nodes.len());
        match filters {
            Some(filters) => filters
                .iter()
                .try_fold(nodes, |acc, (key, (operator, value))| {
                    let nodes = match operator as &str {
                        "=" => acc
                            .into_iter()
                            .filter(|node| match node.metadata.get(key) {
                                Some(current_value) => current_value == value,
                                None => false,
                            })
                            .skip(offset_value)
                            .take(limit_value)
                            .collect(),
                        ">" => acc
                            .into_iter()
                            .filter(|node| match node.metadata.get(key) {
                                Some(current_value) => current_value > value,
                                None => false,
                            })
                            .skip(offset_value)
                            .take(limit_value)
                            .collect(),
                        "<" => acc
                            .into_iter()
                            .filter(|node| match node.metadata.get(key) {
                                Some(current_value) => current_value < value,
                                None => false,
                            })
                            .skip(offset_value)
                            .take(limit_value)
                            .collect(),
                        "<=" => acc
                            .into_iter()
                            .filter(|node| match node.metadata.get(key) {
                                Some(current_value) => current_value <= value,
                                None => false,
                            })
                            .skip(offset_value)
                            .take(limit_value)
                            .collect(),
                        ">=" => acc
                            .into_iter()
                            .filter(|node| match node.metadata.get(key) {
                                Some(current_value) => current_value >= value,
                                None => false,
                            })
                            .skip(offset_value)
                            .take(limit_value)
                            .collect(),
                        "!=" => acc
                            .into_iter()
                            .filter(|node| match node.metadata.get(key) {
                                Some(current_value) => current_value != value,
                                None => false,
                            })
                            .skip(offset_value)
                            .take(limit_value)
                            .collect(),
                        _ => {
                            return Err(NodeRegistryError::InvalidFilterError(format!(
                                "Unknown operator {}",
                                operator
                            )))
                        }
                    };
                    Ok(nodes)
                }),
            None => Ok(nodes
                .into_iter()
                .skip(offset_value)
                .take(limit_value)
                .collect()),
        }
    }

    fn fetch_node(&self, identity: &str) -> Result<Node, NodeRegistryError> {
        match self
            .get_cached_nodes()
            .map_err(|err| NodeRegistryError::InternalError(Box::new(err)))?
            .iter()
            .find(|node| node.identity == identity)
        {
            Some(node) => Ok(node.clone()),
            None => Err(NodeRegistryError::NotFoundError(format!(
                "Could not find node with identity {}",
                identity
            ))),
        }
    }

    fn update_node(
        &self,
        identity: &str,
        updates: HashMap<String, String>,
    ) -> Result<(), NodeRegistryError> {
        let mut nodes = self
            .get_cached_nodes()
            .map_err(|err| NodeRegistryError::InternalError(Box::new(err)))?;
        let mut index = None;
        for (i, node) in nodes.iter().enumerate() {
            if node.identity == identity {
                index = Some(i);
                break;
            }
        }
        match index {
            Some(i) => {
                let node = &nodes[i];
                let mut updated_metadata = node.metadata.clone();
                updated_metadata.extend(updates);
                let updated_node = Node {
                    identity: node.identity.clone(),
                    metadata: updated_metadata,
                };
                nodes[i] = updated_node;
            }
            None => {
                return Err(NodeRegistryError::NotFoundError(format!(
                    "Could not find node with identity: {}",
                    identity
                )))
            }
        };

        self.write_nodes(&nodes)
            .map_err(|err| NodeRegistryError::InternalError(Box::new(err)))
    }

    fn delete_node(&self, identity: &str) -> Result<(), NodeRegistryError> {
        let mut nodes = self
            .get_cached_nodes()
            .map_err(|err| NodeRegistryError::InternalError(Box::new(err)))?;
        let mut index = None;
        for (i, node) in nodes.iter().enumerate() {
            if node.identity == identity {
                index = Some(i);
                break;
            }
        }
        match index {
            Some(i) => nodes.remove(i),
            None => {
                return Err(NodeRegistryError::NotFoundError(format!(
                    "Could not find node with identity: {}",
                    identity
                )))
            }
        };

        self.write_nodes(&nodes)
            .map_err(|err| NodeRegistryError::InternalError(Box::new(err)))
    }

    fn clone_box(&self) -> Box<dyn NodeRegistry> {
        Box::new(Clone::clone(self))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::env;
    use std::fs::{remove_file, File};
    use std::panic;
    use std::thread;

    ///
    /// Verifies that fetch_node with a valid identity, returns the correct node.
    ///
    #[test]
    fn test_fetch_node_ok() {
        run_test(|test_yaml_file_path| {
            write_to_file(&vec![get_node_1(), get_node_2()], test_yaml_file_path);

            let registry = YamlNodeRegistry::new(test_yaml_file_path)
                .expect("Failed to create YamlNodeRegistry");

            let node = registry
                .fetch_node(&get_node_1().identity)
                .expect("Failed to fetch node");
            assert_eq!(node, get_node_1());
        })
    }

    ///
    /// Verifies that fetch_node with an invalid identity, returns NotFoundError
    ///
    #[test]
    fn test_fetch_node_not_found() {
        run_test(|test_yaml_file_path| {
            write_to_file(&vec![get_node_1(), get_node_2()], test_yaml_file_path);

            let registry = YamlNodeRegistry::new(test_yaml_file_path)
                .expect("Failed to create YamlNodeRegistry");

            let result = registry.fetch_node("NodeNotInRegistry");
            match result {
                Ok(_) => panic!("Node is not in the Registry. Error should be returned"),
                Err(NodeRegistryError::NotFoundError(_)) => (),
                Err(err) => panic!("Should have gotten NotFoundError but got {}", err),
            }
        })
    }

    ///
    /// Verifies that list_nodes returns a list of nodes.
    ///
    #[test]
    fn test_list_nodes_ok() {
        run_test(|test_yaml_file_path| {
            write_to_file(&vec![get_node_1(), get_node_2()], test_yaml_file_path);

            let registry = YamlNodeRegistry::new(test_yaml_file_path)
                .expect("Failed to create YamlNodeRegistry");

            let nodes = registry
                .list_nodes(None, None, None)
                .expect("Failed to retrieve nodes");

            assert_eq!(nodes.len(), 2);
            assert_eq!(nodes[0], get_node_1());
            assert_eq!(nodes[1], get_node_2());
        })
    }

    ///
    /// Verifies that list_nodes returns an empty list when there are no nodes in the registry.
    ///
    #[test]
    fn test_list_nodes_empty_ok() {
        run_test(|test_yaml_file_path| {
            write_to_file(&vec![], test_yaml_file_path);

            let registry = YamlNodeRegistry::new(test_yaml_file_path)
                .expect("Failed to create YamlNodeRegistry");

            let nodes = registry
                .list_nodes(None, None, None)
                .expect("Failed to retrieve nodes");

            assert_eq!(nodes.len(), 0);
        })
    }

    ///
    /// Verifies that list_nodes returns the correct items when there is a filter by metadata.
    ///
    #[test]
    fn test_list_nodes_filter_metadata_ok() {
        run_test(|test_yaml_file_path| {
            write_to_file(&vec![get_node_1(), get_node_2()], test_yaml_file_path);

            let registry = YamlNodeRegistry::new(test_yaml_file_path)
                .expect("Failed to create YamlNodeRegistry");

            let mut filter = HashMap::new();
            filter.insert(
                "company".to_string(),
                (
                    "=".to_string(),
                    get_node_2().metadata.get("company").unwrap().to_string(),
                ),
            );

            let nodes = registry
                .list_nodes(Some(filter), None, None)
                .expect("Failed to retrieve nodes");

            assert_eq!(nodes.len(), 1);
            assert_eq!(nodes[0], get_node_2());
        })
    }

    ///
    /// Verifies that list_nodes returns the correct items when there is more than one filter.
    ///
    #[test]
    fn test_list_nodes_filter_multiple_ok() {
        run_test(|test_yaml_file_path| {
            write_to_file(
                &vec![get_node_1(), get_node_2(), get_node_3()],
                test_yaml_file_path,
            );

            let registry = YamlNodeRegistry::new(test_yaml_file_path)
                .expect("Failed to create YamlNodeRegistry");

            let mut filter = HashMap::new();

            // node_2 and node_3 have the same company
            filter.insert(
                "company".to_string(),
                (
                    "=".to_string(),
                    get_node_3().metadata.get("company").unwrap().to_string(),
                ),
            );

            filter.insert(
                "url".to_string(),
                (
                    "=".to_string(),
                    get_node_3().metadata.get("url").unwrap().to_string(),
                ),
            );

            let nodes = registry
                .list_nodes(Some(filter), None, None)
                .expect("Failed to retrieve nodes");

            assert_eq!(nodes.len(), 1);
            assert_eq!(nodes[0], get_node_3());
        })
    }

    ///
    /// Verifies that list_nodes returns an error when an incorrect operator is passed as a filter
    #[test]
    fn test_list_nodes_filter_error() {
        run_test(|test_yaml_file_path| {
            write_to_file(&vec![get_node_1(), get_node_2()], test_yaml_file_path);

            let registry = YamlNodeRegistry::new(test_yaml_file_path)
                .expect("Failed to create YamlNodeRegistry");

            let mut filter = HashMap::new();
            filter.insert(
                "company".to_string(),
                (
                    "==".to_string(),
                    get_node_2().metadata.get("company").unwrap().to_string(),
                ),
            );

            let result = registry.list_nodes(Some(filter), None, None);

            match result {
                Ok(_) => panic!("Incorrect operator was passed.. Error should be returned"),
                Err(NodeRegistryError::InvalidFilterError(_)) => (),
                Err(err) => panic!("Should have gotten InvalidFilterError but got {}", err),
            }
        })
    }

    ///
    /// Verifies that list_nodes returns an empty list when no nodes fits the filtering criteria.
    ///
    #[test]
    fn test_list_nodes_filter_empty_ok() {
        run_test(|test_yaml_file_path| {
            write_to_file(&vec![get_node_1(), get_node_2()], test_yaml_file_path);

            let registry = YamlNodeRegistry::new(test_yaml_file_path)
                .expect("Failed to create YamlNodeRegistry");

            let mut filter = HashMap::new();

            filter.insert(
                "url".to_string(),
                (
                    "=".to_string(),
                    get_node_3().metadata.get("url").unwrap().to_string(),
                ),
            );

            let nodes = registry
                .list_nodes(Some(filter), None, None)
                .expect("Failed to retrieve nodes");

            assert_eq!(nodes.len(), 0);
        })
    }
    ///
    /// Verifies that list_nodes returns the correct items when limit value is passed.
    ///
    #[test]
    fn test_list_nodes_limit_ok() {
        run_test(|test_yaml_file_path| {
            write_to_file(
                &vec![get_node_1(), get_node_2(), get_node_3()],
                test_yaml_file_path,
            );

            let registry = YamlNodeRegistry::new(test_yaml_file_path)
                .expect("Failed to create YamlNodeRegistry");

            let nodes = registry
                .list_nodes(None, Some(2), None)
                .expect("Failed to retrieve nodes");

            assert_eq!(nodes.len(), 2);
            assert_eq!(nodes[0], get_node_1());
            assert_eq!(nodes[1], get_node_2());
        })
    }

    ///
    /// Verifies that list_nodes returns the correct items when offset value is passed.
    ///
    #[test]
    fn test_list_nodes_offset_ok() {
        run_test(|test_yaml_file_path| {
            write_to_file(
                &vec![get_node_1(), get_node_2(), get_node_3()],
                test_yaml_file_path,
            );

            let registry = YamlNodeRegistry::new(test_yaml_file_path)
                .expect("Failed to create YamlNodeRegistry");

            let nodes = registry
                .list_nodes(None, None, Some(1))
                .expect("Failed to retrieve nodes");

            assert_eq!(nodes.len(), 2);
            assert_eq!(nodes[0], get_node_2());
            assert_eq!(nodes[1], get_node_3());
        })
    }

    ///
    /// Verifies that add_node successfully adds a new node to the yaml file.
    ///
    #[test]
    fn test_list_add_node_ok() {
        run_test(|test_yaml_file_path| {
            write_to_file(&vec![], test_yaml_file_path);

            let registry = YamlNodeRegistry::new(test_yaml_file_path)
                .expect("Failed to create YamlNodeRegistry");

            let node = get_node_1();

            registry
                .add_node(node.clone())
                .expect("Failed to add not to file.");

            let nodes = registry
                .list_nodes(None, None, None)
                .expect("Failed to retrieve nodes");

            assert_eq!(nodes.len(), 1);

            assert_eq!(nodes[0], node);
        })
    }

    ///
    /// Verifies that add_node returns DuplicateNodeError when a node with the same identity already
    /// exists in the yaml file.
    ///
    #[test]
    fn test_list_add_node_duplicate_error() {
        run_test(|test_yaml_file_path| {
            write_to_file(&vec![get_node_1()], test_yaml_file_path);

            let registry = YamlNodeRegistry::new(test_yaml_file_path)
                .expect("Failed to create YamlNodeRegistry");

            let node = get_node_1();

            let result = registry.add_node(node.clone());

            match result {
                Ok(_) => panic!("Duplicate node exists. Error should be returned"),
                Err(NodeRegistryError::DuplicateNodeError(_)) => (),
                Err(err) => panic!("Should have gotten DuplicateNodeError but got {}", err),
            }
        })
    }

    ///
    /// Verifies that delete_node with a valid identity, deletes the correct node.
    ///
    #[test]
    fn test_delete_node_ok() {
        run_test(|test_yaml_file_path| {
            write_to_file(&vec![get_node_1(), get_node_2()], test_yaml_file_path);

            let registry = YamlNodeRegistry::new(test_yaml_file_path)
                .expect("Failed to create YamlNodeRegistry");

            registry
                .delete_node(&get_node_1().identity)
                .expect("Failed to delete node");

            let nodes = registry
                .list_nodes(None, None, None)
                .expect("Failed to retrieve nodes");

            assert_eq!(nodes.len(), 1);

            assert_eq!(nodes[0], get_node_2());
        })
    }

    ///
    /// Verifies that delete_node with an invalid identity, returns NotFoundError
    ///
    #[test]
    fn test_delete_node_not_found() {
        run_test(|test_yaml_file_path| {
            write_to_file(&vec![get_node_1(), get_node_2()], test_yaml_file_path);

            let registry = YamlNodeRegistry::new(test_yaml_file_path)
                .expect("Failed to create YamlNodeRegistry");

            let result = registry.delete_node("NodeNotInRegistry");
            match result {
                Ok(_) => panic!("Node is not in the Registry. Error should be returned"),
                Err(NodeRegistryError::NotFoundError(_)) => (),
                Err(err) => panic!("Should have gotten NotFoundError but got {}", err),
            }
        })
    }

    ///
    /// Verifies that update_node with a valid ID, updates the metadata of the correct node.
    ///
    #[test]
    fn test_update_node_ok() {
        run_test(|test_yaml_file_path| {
            write_to_file(&vec![get_node_1(), get_node_2()], test_yaml_file_path);

            let mut updatated_metada = HashMap::new();
            updatated_metada.insert("url".to_string(), "10.0.1.123".to_string());
            updatated_metada.insert("accepting_connections".to_string(), "true".to_string());
            let registry = YamlNodeRegistry::new(test_yaml_file_path)
                .expect("Failed to create YamlNodeRegistry");

            registry
                .update_node(&get_node_1().identity, updatated_metada)
                .expect("Failed to update node");

            let nodes = registry
                .list_nodes(None, None, None)
                .expect("Failed to retrieve nodes");

            assert_eq!(nodes.len(), 2);
            assert_eq!(nodes[1], get_node_2());

            assert_eq!(nodes[0].identity, get_node_1().identity);

            let mut expected_metadata = get_node_1().metadata;
            expected_metadata.insert("url".to_string(), "10.0.1.123".to_string());
            expected_metadata.insert("accepting_connections".to_string(), "true".to_string());
            assert_eq!(nodes[0].metadata, expected_metadata);
        })
    }

    ///
    /// Verifies that update_node with an invalid ID, returns NotFoundError
    ///
    #[test]
    fn test_update_node_not_found() {
        run_test(|test_yaml_file_path| {
            write_to_file(&vec![get_node_1(), get_node_2()], test_yaml_file_path);

            let registry = YamlNodeRegistry::new(test_yaml_file_path)
                .expect("Failed to create YamlNodeRegistry");

            let result = registry.update_node("NodeNotInRegistry", HashMap::new());
            match result {
                Ok(_) => panic!("Node is not in the Registry. Error should be returned"),
                Err(NodeRegistryError::NotFoundError(_)) => (),
                Err(err) => panic!("Should have gotten NotFoundError but got {}", err),
            }
        })
    }

    fn get_node_1() -> Node {
        let mut metadata = HashMap::new();
        metadata.insert("url".to_string(), "12.0.0.123:8431".to_string());
        metadata.insert("company".to_string(), "Bitwise IO".to_string());
        Node {
            identity: "Node-123".to_string(),
            metadata,
        }
    }

    fn get_node_2() -> Node {
        let mut metadata = HashMap::new();
        metadata.insert("url".to_string(), "13.0.0.123:8434".to_string());
        metadata.insert("company".to_string(), "Cargill".to_string());
        Node {
            identity: "Node-456".to_string(),
            metadata,
        }
    }

    fn get_node_3() -> Node {
        let mut metadata = HashMap::new();
        metadata.insert("url".to_string(), "13.0.0.123:8435".to_string());
        metadata.insert("company".to_string(), "Cargill".to_string());
        Node {
            identity: "Node-789".to_string(),
            metadata,
        }
    }

    fn write_to_file(data: &[Node], file_path: &str) {
        let file = File::create(file_path).expect("Error creating test nodes yaml file.");
        serde_yaml::to_writer(file, data).expect("Error writing nodes to file.");
    }

    fn run_test<T>(test: T) -> ()
    where
        T: FnOnce(&str) -> () + panic::UnwindSafe,
    {
        let test_yaml_file = temp_yaml_file_path();

        let test_path = test_yaml_file.clone();
        let result = panic::catch_unwind(move || test(&test_path));

        remove_file(test_yaml_file).unwrap();

        assert!(result.is_ok())
    }

    fn temp_yaml_file_path() -> String {
        let mut temp_dir = env::temp_dir();

        let thread_id = thread::current().id();
        temp_dir.push(format!("test_node_registry-{:?}.yaml", thread_id));
        temp_dir.to_str().unwrap().to_string()
    }
}
