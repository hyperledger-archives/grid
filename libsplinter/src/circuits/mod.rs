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
pub mod circuit;
pub mod circuit_state;
pub mod service;

use std::collections::BTreeMap;
use std::collections::HashMap;

use circuits::circuit::Circuit;
use circuits::circuit_state::CircuitState;
use circuits::service::{Service, SplinterNode};
use storage::get_storage;

pub struct SplinterState {
    // location of the persisted state
    storage_location: String,
    // The state that is persisted
    circuit_state: CircuitState,
    // Service id to Service that contains the node the service is connected to. Not persisted.
    directory: HashMap<String, Service>,
}

impl SplinterState {
    pub fn new(storage_location: String, circuit_state: CircuitState) -> Self {
        SplinterState {
            storage_location,
            circuit_state,
            directory: HashMap::new(),
        }
    }

    pub fn storage_location(&self) -> &str {
        &self.storage_location
    }

    fn write_circuit_state(&self) -> Result<(), WriteError> {
        // Replace stored state with the current splinter state
        let mut storage = get_storage(self.storage_location(), || self.circuit_state.clone())
            .map_err(|err| {
                WriteError::GetStorageError(format!("Unable to get storage: {}", err))
            })?;

        // when this is dropped the new state will be written to storage
        **storage.write() = self.circuit_state.clone();
        Ok(())
    }

    // ---------- methods to access service directory ----------
    pub fn directory(&self) -> &HashMap<String, Service> {
        &self.directory
    }

    pub fn add_service(&mut self, service_id: String, service: Service) {
        self.directory.insert(service_id, service);
    }

    pub fn remove_service(&mut self, service_id: &str) {
        self.directory.remove(service_id);
    }

    // ---------- methods to access circuit state ----------
    pub fn add_node(&mut self, id: String, node: SplinterNode) -> Result<(), WriteError> {
        self.circuit_state.add_node(id, node);
        self.write_circuit_state()?;
        Ok(())
    }

    pub fn add_circuit(&mut self, name: String, circuit: Circuit) -> Result<(), WriteError> {
        self.circuit_state.add_circuit(name, circuit);
        self.write_circuit_state()?;
        Ok(())
    }

    pub fn remove_node(&mut self, id: &str) -> Result<(), WriteError> {
        self.circuit_state.remove_node(id);
        self.write_circuit_state()?;
        Ok(())
    }

    pub fn remove_circuit(&mut self, name: &str) -> Result<(), WriteError> {
        self.circuit_state.remove_circuit(name);
        self.write_circuit_state()?;
        Ok(())
    }

    pub fn nodes(&self) -> &BTreeMap<String, SplinterNode> {
        &self.circuit_state.nodes()
    }

    pub fn node(&self, node_id: &str) -> Option<&SplinterNode> {
        self.circuit_state.node(node_id)
    }

    pub fn circuits(&self) -> &BTreeMap<String, Circuit> {
        &self.circuit_state.circuits()
    }

    pub fn circuit(&self, circuit_name: &str) -> Option<&Circuit> {
        self.circuit_state.circuit(circuit_name)
    }
}

#[derive(Debug)]
pub enum WriteError {
    GetStorageError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempdir::TempDir;

    fn setup_storage(mut temp_dir: PathBuf) -> String {
        // Creat the temp file
        temp_dir.push("circuits.yaml");
        let path = temp_dir.to_str().unwrap().to_string();

        // Write out the mock state file to the temp directory
        path
    }

    #[test]
    fn test_circuit_write_file() {
        // create temp directoy
        let temp_dir = TempDir::new("test_circuit_write_file").unwrap();
        let temp_dir = temp_dir.path().to_path_buf();

        // setup empty state filename
        let path = setup_storage(temp_dir);
        let mut storage = get_storage(&path, || CircuitState::new()).unwrap();
        let circuit_state = storage.write().clone();
        let mut state = SplinterState::new(path.to_string(), circuit_state);

        // Check that SplinterState does not have any circuits
        assert!(state.circuits().len() == 0);

        let circuit = Circuit::new(
            "alpha".into(),
            "trust".into(),
            vec!["123".into()],
            vec!["abc".into(), "def".into()],
            "any".into(),
            "none".into(),
            "require_direct".into(),
        );
        // add circuit to splinter state
        state.add_circuit("alpha".into(), circuit).unwrap();

        // reload storage and check that the circuit was written
        let storage = get_storage(&path, || CircuitState::new()).unwrap();
        assert_eq!(storage.read().circuits().len(), 1);
        assert!(storage.read().circuits().contains_key("alpha"));

        assert_eq!(
            storage
                .read()
                .circuits()
                .get("alpha")
                .unwrap()
                .roster()
                .to_vec(),
            vec!["abc".to_string(), "def".to_string()]
        );

        assert_eq!(
            storage
                .read()
                .circuits()
                .get("alpha")
                .unwrap()
                .members()
                .to_vec(),
            vec!["123".to_string()],
        );

        state.remove_circuit("alpha".into()).unwrap();
        // reload storage and check that the circuit was written
        let storage = get_storage(&path, || CircuitState::new()).unwrap();

        // Check that state does not have any nodes
        assert!(storage.read().circuits().len() == 0);
    }

    #[test]
    fn test_node_write_file() {
        // create temp directoy
        let temp_dir = TempDir::new("test_node_write_file").unwrap();
        let temp_dir = temp_dir.path().to_path_buf();

        // setup empty state filename
        let path = setup_storage(temp_dir);
        let mut storage = get_storage(&path, || CircuitState::new()).unwrap();
        let circuit_state = storage.write().clone();
        let mut state = SplinterState::new(path.to_string(), circuit_state);

        // Check that SplinterState does not have any nodes
        assert!(state.nodes().len() == 0);

        let node = SplinterNode::new("123".into(), vec!["tcp://127.0.0.1:8000".into()]);
        state.add_node("123".into(), node).unwrap();

        // reload storage and check that the node was written
        let storage = get_storage(&path, || CircuitState::new()).unwrap();
        // check that the CircuitState data contains the correct node and circuit
        assert!(storage.read().nodes().len() == 1);
        assert!(storage.read().nodes().contains_key("123"));

        assert_eq!(
            storage
                .read()
                .nodes()
                .get("123")
                .unwrap()
                .endpoints()
                .to_vec(),
            vec!["tcp://127.0.0.1:8000".to_string()]
        );

        state.remove_node("123".into()).unwrap();

        // reload storage and check that the node was removed
        let storage = get_storage(&path, || CircuitState::new()).unwrap();
        // Check that state does not have any nodes
        assert!(storage.read().nodes().len() == 0);
    }
}
