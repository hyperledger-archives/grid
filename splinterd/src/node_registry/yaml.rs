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

use super::error::YamlNodeRegistryError;
use libsplinter::node_registry::{error::NodeRegistryError, Node, NodeRegistry};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct YamlNodeRegistry {
    file_internal: Arc<Mutex<FileInternal>>,
}

pub struct FileInternal {
    pub file: File,
    pub cached_nodes: Vec<Node>,
}

impl YamlNodeRegistry {
    #[allow(dead_code)]
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
}

impl NodeRegistry for YamlNodeRegistry {
    fn create_node(
        &self,
        _identity: &str,
        _data: HashMap<String, String>,
    ) -> Result<(), NodeRegistryError> {
        unimplemented!()
    }

    fn list_nodes(
        &self,
        _filters: Option<HashMap<String, (String, String)>>,
        _limit: Option<usize>,
        _offset: Option<usize>,
    ) -> Result<Vec<Node>, NodeRegistryError> {
        unimplemented!()
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
        _identity: &str,
        _updates: HashMap<String, String>,
    ) -> Result<(), NodeRegistryError> {
        unimplemented!()
    }

    fn delete_node(&self, _identity: &str) -> Result<(), NodeRegistryError> {
        unimplemented!()
    }

    fn clone_box(&self) -> Box<NodeRegistry> {
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
