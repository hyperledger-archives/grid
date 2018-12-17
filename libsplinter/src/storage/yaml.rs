// Copyright 2018 Cargill Incorporated
// Copyright 2018 Bitwise IO, Inc.
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

use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};

use atomicwrites::{AllowOverwrite, AtomicFile};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_yaml::{from_str, to_string};

use super::{Storage, StorageReadGuard, StorageWriteGuard};

/// A yaml read guard
pub struct YamlStorageReadGuard<'a, T: Serialize + DeserializeOwned + 'a> {
    storage: &'a YamlStorage<T>,
}

impl<'a, T: Serialize + DeserializeOwned> YamlStorageReadGuard<'a, T> {
    fn new(storage: &'a YamlStorage<T>) -> Self {
        Self { storage }
    }
}

impl<'a, T: Serialize + DeserializeOwned + 'a> Deref for YamlStorageReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.storage.data
    }
}

impl<'a, T: 'a + Serialize + DeserializeOwned + fmt::Display> fmt::Display
    for YamlStorageReadGuard<'a, T>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<'a, T: 'a + Serialize + DeserializeOwned> StorageReadGuard<'a, T>
    for YamlStorageReadGuard<'a, T>
{
}

/// A yaml write guard
pub struct YamlStorageWriteGuard<'a, T: Serialize + DeserializeOwned + 'a> {
    storage: &'a mut YamlStorage<T>,
}

impl<'a, T: Serialize + DeserializeOwned> YamlStorageWriteGuard<'a, T> {
    fn new(storage: &'a mut YamlStorage<T>) -> Self {
        Self { storage }
    }
}

impl<'a, T: Serialize + DeserializeOwned> Drop for YamlStorageWriteGuard<'a, T> {
    fn drop(&mut self) {
        self.storage
            .file
            .write(|f| {
                f.write_all(
                    to_string(&self.storage.data)
                        .expect("Couldn't convert value to string!")
                        .as_bytes(),
                )
            })
            .expect("File write failed while dropping YamlStorageWriteGuard!");
    }
}

impl<'a, T: Serialize + DeserializeOwned + 'a> Deref for YamlStorageWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.storage.data
    }
}

impl<'a, T: Serialize + DeserializeOwned + 'a> DerefMut for YamlStorageWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.storage.data
    }
}

impl<'a, T: 'a + Serialize + DeserializeOwned + fmt::Display> fmt::Display
    for YamlStorageWriteGuard<'a, T>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<'a, T: 'a + Serialize + DeserializeOwned> StorageWriteGuard<'a, T>
    for YamlStorageWriteGuard<'a, T>
{
}

// A Yaml Storage implementation
///
/// File writes are atomic
pub struct YamlStorage<T: Serialize + DeserializeOwned> {
    data: T,
    file: AtomicFile,
}

impl<T: Serialize + DeserializeOwned> YamlStorage<T> {
    pub fn new<P: Into<String>, F: Fn() -> T>(path: P, default: F) -> Result<Self, String> {
        let path = path.into();

        let file = AtomicFile::new(path, AllowOverwrite);

        // Read the file first, to see if there's any existing data
        let data = match File::open(file.path()) {
            Ok(mut f) => {
                let mut contents = String::new();

                f.read_to_string(&mut contents)
                    .map_err(|err| format!("Couldn't read file: {}", err))?;

                from_str(&contents).map_err(|err| format!("Couldn't read file: {}", err))?
            }
            Err(_) => {
                let data = default();
                let data_string =
                    to_string(&data).map_err(|err| format!("File write failed: {}", err))?;

                file.write(|f| f.write_all(data_string.as_bytes()))
                    .map_err(|err| format!("File write failed: {}", err))?;

                data
            }
        };

        // Then open the file again and truncate, preparing it to be written to
        Ok(Self { data, file })
    }
}

impl<T: fmt::Display + Serialize + DeserializeOwned> fmt::Display for YamlStorage<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (*self).data.fmt(f)
    }
}

impl<T: Serialize + DeserializeOwned> Storage for YamlStorage<T> {
    type S = T;

    fn read<'a>(&'a self) -> Box<StorageReadGuard<'a, T, Target = T> + 'a> {
        Box::new(YamlStorageReadGuard::new(self))
    }

    fn write<'a>(&'a mut self) -> Box<StorageWriteGuard<'a, T, Target = T> + 'a> {
        Box::new(YamlStorageWriteGuard::new(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use circuits::circuit::Circuit;
    use circuits::circuit_state::CircuitState;
    use circuits::service::SplinterNode;
    use std::path::PathBuf;
    use tempdir::TempDir;

    /* Creates a state file that looks like the following:
        ---
        nodes:
          123:
            endpoints:
              - "tcp://1.2.3.4:1234"
              - "inproc://127.0.0.1:100001"
        circuits:
          alpha:
            auth: trust
            members:
              - "123"
            services:
              - abc
              - def
            persistence: any
            durability: none
            routes: require_direct
    */
    fn set_up_mock_state_file(mut temp_dir: PathBuf) -> String {
        // Create mock state
        let mut state = CircuitState::new();
        let node = SplinterNode::new("123".into(), vec!["tcp://127.0.0.1:8000".into()]);
        state.add_node("123".into(), node);

        let circuit = Circuit::new(
            "alpha".into(),
            "trust".into(),
            vec!["123".into()],
            vec!["abc".into(), "def".into()],
            "any".into(),
            "none".into(),
            "require_direct".into(),
        );
        state.add_circuit("alpha".into(), circuit);

        let state_string = serde_yaml::to_string(&state).unwrap();

        // Creat the temp file
        temp_dir.push("circuits.yaml");
        let path = temp_dir.to_str().unwrap().to_string();

        // Write out the mock state file to the temp directory
        let mut file = File::create(path.to_string()).unwrap();
        file.write_all(state_string.as_bytes()).unwrap();
        path
    }

    /* Creates a state file that looks like the following:
        ---
        nodes:
        circuits:
    */
    fn setup_empty_state_file(mut temp_dir: PathBuf) -> String {
        // Create empty CircuitState object
        let state = CircuitState::new();

        let state_string = serde_yaml::to_string(&state).unwrap();

        // Creat the temp file
        temp_dir.push("circuits.yaml");
        let path = temp_dir.to_str().unwrap().to_string();

        // Write out the mock state file to the temp directory
        let mut file = File::create(path.to_string()).unwrap();
        file.write_all(state_string.as_bytes()).unwrap();
        path
    }

    #[test]
    /* Test that an empty state is properly loaded and returns a YamlStorage with CircuitState object that
       contains no nodes or circuits. The empty state file looks like the following:

       ---
       nodes:
       circuits:
    */
    fn test_load_empty_state() {
        // create temp directoy
        let temp_dir = TempDir::new("test_empty_state").unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();

        // setup empty state file
        let path = setup_empty_state_file(temp_dir_path);

        // load empty state file into the yaml storage
        let storage = YamlStorage::new(path, || CircuitState::new()).unwrap();

        // check that state does not have any nodes or circuits
        assert!(storage.data.nodes().is_empty());
        assert!(storage.data.circuits().is_empty());
    }

    #[test]
    // Test that if the state file does not exist, it is created as an empty state.
    fn test_load_no_state() {
        // create temp directoy
        let temp_dir = TempDir::new("test_load_no_state").unwrap();
        let mut temp_dir_path = temp_dir.path().to_path_buf();
        temp_dir_path.push("circuits.yaml");
        let path = temp_dir_path.to_str().unwrap().to_string();

        // create state file empty state when file does not exist
        let storage = YamlStorage::new(path, || CircuitState::new()).unwrap();

        // check that state does not have any nodes or circuits
        assert!(storage.data.nodes().is_empty());
        assert!(storage.data.circuits().is_empty());
    }

    #[test]
    /* Test that CircuitState object is properly loaded into YamlStorage from a state yaml file that
        looks like the following:

       ---
       nodes:
         123:
           endpoints:
             - "tcp://1.2.3.4:1234"
             - "inproc://127.0.0.1:100001"
       circuits:
         alpha:
           auth: trust
           members:
             - "123"
           services:
             - abc
             - def
           persistence: any
           durability: none
           routes: require_direct
    */
    fn test_load_state() {
        // create temp directoy
        let temp_dir = TempDir::new("test_load_state").unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();

        // setup mock state file
        let path = set_up_mock_state_file(temp_dir_path);

        // load state file into yaml storage
        let storage = YamlStorage::new(path, || CircuitState::new()).unwrap();

        // check that the CircuitState data contains the correct node and circuit
        assert_eq!(storage.data.nodes().len(), 1);
        assert_eq!(storage.data.circuits().len(), 1);
        assert!(storage.data.nodes().contains_key("123"));
        assert!(storage.data.circuits().contains_key("alpha"));

        assert_eq!(
            storage
                .data
                .nodes()
                .get("123")
                .unwrap()
                .endpoints()
                .to_vec(),
            vec!["tcp://127.0.0.1:8000".to_string()]
        );

        assert_eq!(
            storage
                .data
                .circuits()
                .get("alpha")
                .unwrap()
                .roster()
                .to_vec(),
            vec!["abc".to_string(), "def".to_string()]
        );

        assert_eq!(
            storage
                .data
                .circuits()
                .get("alpha")
                .unwrap()
                .members()
                .to_vec(),
            vec!["123".to_string()],
        );
    }

    #[test]
    // Using the mock state file as a starting point, test that a new node can be properly
    // added to the state file. CircuitState is then loaded into yaml storage and verified that the
    // added node is there.
    fn test_write_node_state() {
        // create temp directoy
        let temp_dir = TempDir::new("test_write_node").unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();

        // setup mock state file
        let path = set_up_mock_state_file(temp_dir_path);
        {
            // load state file into yaml storage
            let mut storage = YamlStorage::new(path.clone(), || CircuitState::new()).unwrap();

            // add new node to state
            let node = SplinterNode::new("123".into(), vec!["tcp://127.0.0.1:5000".into()]);
            storage.write().add_node("777".into(), node);

            //drop storage
        }

        // load state file into yaml storage
        let storage = YamlStorage::new(path, || CircuitState::new()).unwrap();
        // check that the CircuitState data contains the new node
        assert_eq!(storage.data.nodes().len(), 2);
        assert_eq!(storage.data.circuits().len(), 1);
        assert!(storage.data.nodes().contains_key("123"));
        assert!(storage.data.nodes().contains_key("777"));

        assert_eq!(
            storage
                .data
                .nodes()
                .get("123")
                .unwrap()
                .endpoints()
                .to_vec(),
            vec!["tcp://127.0.0.1:8000".to_string()]
        );

        assert_eq!(
            storage
                .data
                .nodes()
                .get("777")
                .unwrap()
                .endpoints()
                .to_vec(),
            vec!["tcp://127.0.0.1:5000".to_string()]
        );
    }

    #[test]
    // Using the mock state file as a starting point, test that node 123 can be properly
    // removed to the state file. CircuitState is then loaded into yaml storage and verified that node
    // 123 has been removed. Verify that circuit alpha is still there.
    fn test_remove_node_from_state() {
        // create temp directoy
        let temp_dir = TempDir::new("test_write_circuit").unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        // setup mock state file
        let path = set_up_mock_state_file(temp_dir_path);
        {
            // load state file into yaml storage
            let mut storage = YamlStorage::new(path.clone(), || CircuitState::new()).unwrap();

            storage.write().remove_node("123".into());

            // drop storage
        }
        // load state file into yaml storage
        let storage = YamlStorage::new(path.clone(), || CircuitState::new()).unwrap();

        // check that the CircuitState data does not contain node 123
        assert_eq!(storage.data.nodes().len(), 0);
        assert_eq!(storage.data.circuits().len(), 1);
        assert!(!storage.data.nodes().contains_key("123"));
        assert!(storage.data.circuits().contains_key("alpha"));

        assert_eq!(
            storage
                .data
                .circuits()
                .get("alpha")
                .unwrap()
                .roster()
                .to_vec(),
            vec!["abc".to_string(), "def".to_string()]
        );

        assert_eq!(
            storage
                .data
                .circuits()
                .get("alpha")
                .unwrap()
                .members()
                .to_vec(),
            vec!["123".to_string()],
        );
    }

    #[test]
    // Using the mock state file as a starting point, test that a new circuit can be properly
    // added to the state file. CircuitState is then loaded into yaml storage and verified that the
    // added circuit is there.
    fn test_write_circuit_state() {
        // create temp directoy
        let temp_dir = TempDir::new("test_write_circuit").unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        // setup mock state file
        let path = set_up_mock_state_file(temp_dir_path);
        {
            // load state file into yaml storage
            let mut storage = YamlStorage::new(path.clone(), || CircuitState::new()).unwrap();
            let circuit = Circuit::new(
                "beta".into(),
                "trust".into(),
                vec!["456".into(), "789".into()],
                vec!["qwe".into(), "rty".into(), "uio".into()],
                "any".into(),
                "none".into(),
                "require_direct".into(),
            );
            storage.write().add_circuit("beta".into(), circuit);

            //drop storage
        }

        // load state file into yaml storage
        let storage = YamlStorage::new(path.clone(), || CircuitState::new()).unwrap();

        // check that the CircuitState data contains the new circuit
        assert_eq!(storage.data.circuits().len(), 2);
        assert!(storage.data.circuits().contains_key("alpha"));
        assert!(storage.data.circuits().contains_key("beta"));

        assert_eq!(
            storage
                .data
                .circuits()
                .get("alpha")
                .unwrap()
                .roster()
                .to_vec(),
            vec!["abc".to_string(), "def".to_string()]
        );

        assert_eq!(
            storage
                .data
                .circuits()
                .get("alpha")
                .unwrap()
                .members()
                .to_vec(),
            vec!["123".to_string()],
        );

        assert_eq!(
            storage
                .data
                .circuits()
                .get("beta")
                .unwrap()
                .roster()
                .to_vec(),
            vec!["qwe".to_string(), "rty".to_string(), "uio".to_string()]
        );

        assert_eq!(
            storage
                .data
                .circuits()
                .get("beta")
                .unwrap()
                .members()
                .to_vec(),
            vec!["456".to_string(), "789".to_string()],
        );
    }

    #[test]
    // Using the mock state file as a starting point, test that circuit alpha can be properly
    // removed to the state file. CircuitState is then loaded into yaml storage and verified that circuit
    // alpha has been removed. Verify that node 123 is still there.
    fn test_remove_circuit_from_state() {
        // create temp directoy
        let temp_dir = TempDir::new("test_write_circuit").unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        // setup mock state file
        let path = set_up_mock_state_file(temp_dir_path);
        {
            // load state file into yaml storage
            let mut storage = YamlStorage::new(path.clone(), || CircuitState::new()).unwrap();
            storage.write().remove_circuit("alpha".into());

            // drop storage
        }

        // load state file into yaml storage
        let storage = YamlStorage::new(path.clone(), || CircuitState::new()).unwrap();

        // check that the CircuitState data does not contain cirucit alpha
        assert_eq!(storage.data.nodes().len(), 1);
        assert_eq!(storage.data.circuits().len(), 0);
        assert!(storage.data.nodes().contains_key("123"));
        assert!(!storage.data.circuits().contains_key("alpha"));

        assert_eq!(
            storage
                .data
                .nodes()
                .get("123")
                .unwrap()
                .endpoints()
                .to_vec(),
            vec!["tcp://127.0.0.1:8000".to_string()]
        );
    }

}
