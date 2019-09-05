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
pub mod directory;
pub mod handlers;
pub mod service;

use serde_derive::{Deserialize, Serialize};

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::circuit::directory::CircuitDirectory;
use crate::circuit::service::{Service, ServiceId, SplinterNode};
use crate::storage::get_storage;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Circuit {
    #[serde(skip)]
    id: String,
    auth: String,
    members: Vec<String>,
    roster: Roster,
    persistence: String,
    durability: String,
    routes: String,

    #[serde(default = "Circuit::default_management_type")]
    circuit_management_type: String,
}

impl Circuit {
    pub fn builder() -> CircuitBuilder {
        CircuitBuilder::default()
    }

    fn default_management_type() -> String {
        "default".into()
    }

    pub fn new_admin() -> Self {
        Circuit {
            id: "admin".into(),
            auth: "".into(),
            members: vec![],
            roster: Roster::Admin,
            persistence: "".into(),
            durability: "".into(),
            routes: "".into(),
            circuit_management_type: "".into(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn auth(&self) -> &str {
        &self.auth
    }

    pub fn members(&self) -> Members {
        Members { circuit: self }
    }

    pub fn roster(&self) -> &Roster {
        &self.roster
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

    pub fn circuit_management_type(&self) -> &str {
        &self.circuit_management_type
    }
}

#[derive(Default)]
pub struct CircuitBuilder {
    id: Option<String>,
    auth: Option<String>,
    members: Vec<String>,
    roster: Vec<ServiceDefinition>,
    persistence: Option<String>,
    durability: Option<String>,
    routes: Option<String>,

    circuit_management_type: Option<String>,
}

impl CircuitBuilder {
    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);

        self
    }

    pub fn with_members<I: IntoIterator<Item = String>>(mut self, members: I) -> Self {
        self.members.extend(members.into_iter());

        self
    }

    pub fn with_roster<I: IntoIterator<Item = ServiceDefinition>>(mut self, roster: I) -> Self {
        self.roster.extend(roster.into_iter());

        self
    }

    pub fn with_auth(mut self, auth: String) -> Self {
        self.auth = Some(auth);

        self
    }

    pub fn with_persistence(mut self, persistence: String) -> Self {
        self.persistence = Some(persistence);

        self
    }

    pub fn with_durability(mut self, durability: String) -> Self {
        self.durability = Some(durability);

        self
    }

    pub fn with_routes(mut self, id: String) -> Self {
        self.routes = Some(id);

        self
    }

    pub fn with_circuit_management_type(mut self, circuit_management_type: String) -> Self {
        self.circuit_management_type = Some(circuit_management_type);

        self
    }

    pub fn build(self) -> Result<Circuit, CircuitBuildError> {
        if self.members.is_empty() {
            return Err(CircuitBuildError(
                "Circuit requires at least one member".into(),
            ));
        }

        Ok(Circuit {
            id: self
                .id
                .ok_or_else(|| CircuitBuildError("Circuit requires an id".into()))?,
            auth: self.auth.ok_or_else(|| {
                CircuitBuildError("Circuit requires an auth configuration".into())
            })?,

            members: self.members,
            roster: Roster::Standard(self.roster),
            persistence: self.persistence.ok_or_else(|| {
                CircuitBuildError("Circuit requires a persistence setting".into())
            })?,
            routes: self
                .routes
                .ok_or_else(|| CircuitBuildError("Circuit requires a routes setting".into()))?,
            durability: self
                .durability
                .ok_or_else(|| CircuitBuildError("Circuit requires a durability setting".into()))?,
            circuit_management_type: self
                .circuit_management_type
                .unwrap_or_else(Circuit::default_management_type),
        })
    }
}

#[derive(Debug)]
pub struct CircuitBuildError(pub String);

impl std::error::Error for CircuitBuildError {}

impl Display for CircuitBuildError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "unable to build circuit: {}", self.0)
    }
}

pub struct Members<'c> {
    circuit: &'c Circuit,
}

impl<'c> Members<'c> {
    pub fn contains(&self, node_id: &str) -> bool {
        self.circuit
            .members
            .iter()
            .any(|member_id| member_id == node_id)
    }

    pub fn to_vec(&self) -> Vec<String> {
        self.circuit.members.to_vec()
    }
}

impl<'c> IntoIterator for Members<'c> {
    type Item = &'c String;
    type IntoIter = std::slice::Iter<'c, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.circuit.members.iter()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ServiceDefinition {
    service_id: String,
    service_type: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    allowed_nodes: Vec<String>,

    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    #[serde(default = "BTreeMap::new")]
    arguments: BTreeMap<String, String>,
}

impl ServiceDefinition {
    pub fn builder(service_id: String, service_type: String) -> ServiceDefinitionBuilder {
        ServiceDefinitionBuilder {
            service_id,
            service_type,
            allowed_nodes: vec![],
            arguments: BTreeMap::new(),
        }
    }

    pub fn service_id(&self) -> &str {
        &self.service_id
    }

    pub fn service_type(&self) -> &str {
        &self.service_type
    }

    pub fn allowed_nodes(&self) -> &[String] {
        &self.allowed_nodes
    }

    pub fn arguments(&self) -> &BTreeMap<String, String> {
        &self.arguments
    }
}

pub struct ServiceDefinitionBuilder {
    service_id: String,
    service_type: String,
    allowed_nodes: Vec<String>,
    arguments: BTreeMap<String, String>,
}

impl ServiceDefinitionBuilder {
    pub fn with_allowed_nodes<I: IntoIterator<Item = String>>(mut self, node_ids: I) -> Self {
        self.allowed_nodes.extend(node_ids.into_iter());

        self
    }

    pub fn with_arguments<I: IntoIterator<Item = (String, String)>>(
        mut self,
        arguments: I,
    ) -> Self {
        self.arguments.extend(arguments.into_iter());

        self
    }

    pub fn build(self) -> ServiceDefinition {
        ServiceDefinition {
            service_id: self.service_id,
            service_type: self.service_type,
            allowed_nodes: self.allowed_nodes,
            arguments: self.arguments,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Roster {
    Standard(Vec<ServiceDefinition>),
    Admin,
}

impl Roster {
    pub fn contains(&self, service_name: &str) -> bool {
        match self {
            Roster::Standard(roster) => roster
                .iter()
                .any(|service_def| service_def.service_id == service_name),
            Roster::Admin => service_name.starts_with("admin::"),
        }
    }

    pub fn to_vec(&self) -> Vec<ServiceDefinition> {
        match self {
            Roster::Standard(roster) => roster.to_vec(),
            Roster::Admin => Vec::with_capacity(0),
        }
    }

    pub fn iter(&self) -> RosterIter {
        match self {
            Roster::Standard(roster) => RosterIter::Standard(roster.iter()),
            Roster::Admin => RosterIter::Admin,
        }
    }
}

pub enum RosterIter<'r> {
    Standard(std::slice::Iter<'r, ServiceDefinition>),
    Admin,
}

impl<'r> Iterator for RosterIter<'r> {
    type Item = &'r ServiceDefinition;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RosterIter::Standard(ref mut it) => it.next(),
            RosterIter::Admin => None,
        }
    }
}

impl<'r> IntoIterator for &'r Roster {
    type Item = &'r ServiceDefinition;
    type IntoIter = RosterIter<'r>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct SplinterState {
    // location of the persisted state
    storage_location: String,
    // The state that is persisted
    circuit_directory: CircuitDirectory,
    // Service id to Service that contains the node the service is connected to. Not persisted.
    service_directory: HashMap<ServiceId, Service>,
}

impl SplinterState {
    pub fn new(storage_location: String, circuit_directory: CircuitDirectory) -> Self {
        SplinterState {
            storage_location,
            circuit_directory,
            service_directory: HashMap::new(),
        }
    }

    pub fn storage_location(&self) -> &str {
        &self.storage_location
    }

    fn write_circuit_directory(&self) -> Result<(), WriteError> {
        // Replace stored state with the current splinter state
        let mut storage = get_storage(self.storage_location(), || self.circuit_directory.clone())
            .map_err(|err| WriteError::GetStorageError(err.to_string()))?;

        // when this is dropped the new state will be written to storage
        **storage.write() = self.circuit_directory.clone();
        Ok(())
    }

    // ---------- methods to access service directory ----------
    pub fn service_directory(&self) -> &HashMap<ServiceId, Service> {
        &self.service_directory
    }

    pub fn add_service(&mut self, service_id: ServiceId, service: Service) {
        self.service_directory.insert(service_id, service);
    }

    pub fn remove_service(&mut self, service_id: &ServiceId) {
        self.service_directory.remove(service_id);
    }

    // ---------- methods to access circuit directory ----------
    pub fn add_node(&mut self, id: String, node: SplinterNode) -> Result<(), WriteError> {
        self.circuit_directory.add_node(id, node);
        self.write_circuit_directory()?;
        Ok(())
    }

    pub fn add_circuit(&mut self, name: String, circuit: Circuit) -> Result<(), WriteError> {
        self.circuit_directory.add_circuit(name, circuit);
        self.write_circuit_directory()?;
        Ok(())
    }

    pub fn remove_node(&mut self, id: &str) -> Result<(), WriteError> {
        self.circuit_directory.remove_node(id);
        self.write_circuit_directory()?;
        Ok(())
    }

    pub fn remove_circuit(&mut self, name: &str) -> Result<(), WriteError> {
        self.circuit_directory.remove_circuit(name);
        self.write_circuit_directory()?;
        Ok(())
    }

    pub fn nodes(&self) -> &BTreeMap<String, SplinterNode> {
        &self.circuit_directory.nodes()
    }

    pub fn node(&self, node_id: &str) -> Option<&SplinterNode> {
        self.circuit_directory.node(node_id)
    }

    pub fn circuits(&self) -> &BTreeMap<String, Circuit> {
        &self.circuit_directory.circuits()
    }

    pub fn circuit(&self, circuit_name: &str) -> Option<&Circuit> {
        self.circuit_directory.circuit(circuit_name)
    }

    pub fn has_circuit(&self, circuit_name: &str) -> bool {
        self.circuit_directory.has_circuit(circuit_name)
    }
}

#[derive(Debug)]
pub enum WriteError {
    GetStorageError(String),
}

impl Error for WriteError {}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WriteError::GetStorageError(msg) => write!(f, "Unable to get storage: {}", msg),
        }
    }
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

    impl Into<ServiceDefinition> for &str {
        fn into(self) -> ServiceDefinition {
            ServiceDefinition::builder(self.to_string(), "test_type".into())
                .with_allowed_nodes(vec!["*".into()])
                .with_arguments(vec![("test-key".into(), "test-value".into())])
                .build()
        }
    }

    #[test]
    fn test_circuit_write_file() {
        // create temp directoy
        let temp_dir = TempDir::new("test_circuit_write_file").unwrap();
        let temp_dir = temp_dir.path().to_path_buf();

        // setup empty state filename
        let path = setup_storage(temp_dir);
        let mut storage = get_storage(&path, CircuitDirectory::new).unwrap();
        let circuit_directory = storage.write().clone();
        let mut state = SplinterState::new(path.to_string(), circuit_directory);

        // Check that SplinterState does not have any circuits
        assert!(state.circuits().len() == 0);

        let circuit = Circuit::builder()
            .with_id("alpha".into())
            .with_auth("trust".into())
            .with_members(vec!["123".into()])
            .with_roster(vec!["abc".into(), "def".into()])
            .with_persistence("any".into())
            .with_durability("none".into())
            .with_routes("require_direct".into())
            .with_circuit_management_type("test_app".into())
            .build()
            .expect("Should have built a correct circuit");

        // add circuit to splinter state
        state.add_circuit("alpha".into(), circuit).unwrap();

        // reload storage and check that the circuit was written
        let storage = get_storage(&path, CircuitDirectory::new).unwrap();
        std::io::Write::write_all(
            &mut std::io::stderr(),
            &serde_yaml::to_vec(&**storage.read()).unwrap(),
        )
        .unwrap();

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
            vec!["abc".into(), "def".into()]
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
        let storage = get_storage(&path, CircuitDirectory::new).unwrap();

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
        let mut storage = get_storage(&path, CircuitDirectory::new).unwrap();
        let circuit_directory = storage.write().clone();
        let mut state = SplinterState::new(path.to_string(), circuit_directory);

        // Check that SplinterState does not have any nodes
        assert!(state.nodes().len() == 0);

        let node = SplinterNode::new("123".into(), vec!["tcp://127.0.0.1:8000".into()]);
        state.add_node("123".into(), node).unwrap();

        // reload storage and check that the node was written
        let storage = get_storage(&path, CircuitDirectory::new).unwrap();
        // check that the CircuitDirectory data contains the correct node and circuit
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
        let storage = get_storage(&path, CircuitDirectory::new).unwrap();
        // Check that state does not have any nodes
        assert!(storage.read().nodes().len() == 0);
    }
}
