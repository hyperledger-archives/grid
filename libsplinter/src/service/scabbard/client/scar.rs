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

use std::env::{split_paths, var_os};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use bzip2::read::BzDecoder;
use tar::Archive;

use super::Error;

const SCAR_FILE_EXTENSION: &str = "scar";
const SCAR_PATH_ENV_VAR: &str = "SCAR_PATH";
const MANIFEST_FILENAME: &str = "manifest.yaml";
const WASM_FILE_EXTENSION: &str = "wasm";

/// The definition of a Sabre smart contract, including the bytes of the smart contract itself and
/// the associated metadata that is required for submitting the smart contract to scabbard.
#[derive(Debug)]
pub struct SabreSmartContractDefinition {
    pub contract: Vec<u8>,
    pub metadata: SabreSmartContractMetadata,
}

impl SabreSmartContractDefinition {
    /// Load a `SabreSmartContractDefinition` from a .scar file on the local filesystem.
    ///
    /// If the argument is a file path (contains a '/'), this will attempt to load the .scar from
    /// the specified location. If the argument is not a file path, this will attempt to load the
    /// .scar from the directories listed in the SCAR_PATH environment variable. When loading from
    /// a directory in SCAR_PATH, the '.scar' file extension is optional.
    pub fn new_from_scar(scar: &str) -> Result<SabreSmartContractDefinition, Error> {
        let scar_file_path = determine_scar_file_path(scar)?;
        load_smart_contract_from_file(&scar_file_path)
    }
}

/// The metadata of a smart contract that needs to be included in the Sabre transaction.
#[derive(Debug, Deserialize, Serialize)]
pub struct SabreSmartContractMetadata {
    pub name: String,
    pub version: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

fn determine_scar_file_path(scar: &str) -> Result<PathBuf, Error> {
    if scar.contains('/') {
        Ok(PathBuf::from(scar))
    } else {
        let scar_paths = var_os(SCAR_PATH_ENV_VAR).ok_or_else(|| {
            Error::new(&format!(
                "cannot find scar file: {} not set",
                SCAR_PATH_ENV_VAR
            ))
        })?;
        split_paths(&scar_paths)
            .find_map(|mut path| {
                path.push(scar);
                if path.exists() {
                    Some(path)
                } else {
                    path.set_extension(SCAR_FILE_EXTENSION);
                    if path.exists() {
                        Some(path)
                    } else {
                        None
                    }
                }
            })
            .ok_or_else(|| Error::new(&format!("{} not found in {}", scar, SCAR_PATH_ENV_VAR)))
    }
}

fn load_smart_contract_from_file(file_path: &Path) -> Result<SabreSmartContractDefinition, Error> {
    let scar_file = File::open(file_path).map_err(|err| {
        Error::new_with_source(
            &format!("failed to open file {}", file_path.display()),
            err.into(),
        )
    })?;
    let mut archive = Archive::new(BzDecoder::new(scar_file));
    let archive_entries = archive
        .entries()
        .map_err(|err| Error::new_with_source("failed to read scar file", err.into()))?;

    let mut metadata = None;
    let mut contract = None;

    for entry in archive_entries {
        let mut entry = entry.map_err(|err| {
            Error::new_with_source(
                "invalid scar file: failed to read archive entry",
                err.into(),
            )
        })?;
        let path = entry
            .path()
            .map_err(|err| {
                Error::new_with_source(
                    "invalid scar file: failed to get path of archive entry",
                    err.into(),
                )
            })?
            .into_owned();
        if path_is_manifest(&path) {
            metadata = Some(serde_yaml::from_reader(entry).map_err(|err| {
                Error::new_with_source("invalid scar file: manifest.yaml invalid", err.into())
            })?);
        } else if path_is_wasm(&path) {
            let mut contract_bytes = vec![];
            entry.read_to_end(&mut contract_bytes).map_err(|err| {
                Error::new_with_source(
                    "invalid scar file: failed to read smart contract",
                    err.into(),
                )
            })?;
            contract = Some(contract_bytes);
        }
    }

    Ok(SabreSmartContractDefinition {
        metadata: metadata
            .ok_or_else(|| Error::new("invalid scar file: manifest.yaml not found"))?,
        contract: contract
            .ok_or_else(|| Error::new("invalid scar file: smart contract not found"))?,
    })
}

fn path_is_manifest(path: &std::path::Path) -> bool {
    path.file_name()
        .map(|file_name| file_name == MANIFEST_FILENAME)
        .unwrap_or(false)
}

fn path_is_wasm(path: &std::path::Path) -> bool {
    match path.extension() {
        Some(extension) => extension == WASM_FILE_EXTENSION,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;
    use std::path::Path;

    use bzip2::write::BzEncoder;
    use bzip2::Compression;
    use serde::Serialize;
    use serial_test::serial;
    use tar::Builder;
    use tempdir::TempDir;

    const MOCK_CONTRACT_BYTES: &[u8] = &[0x00, 0x01, 0x02, 0x03];
    const MOCK_CONTRACT_FILENAME: &str = "mock.wasm";
    const MOCK_SCAR_FILENAME: &str = "mock.scar";

    // The tests in this module must run serially because some tests modify environment variable(s)
    // that are used by all tests. Each test is annotated with `#[serial(scar_path)]` to enforce
    // this.

    /// Verify that a .scar file can be loaded by providing the name + extension of a .scar file
    /// that is located in one of the paths specified by the SCAR_PATH environment variable.
    /// Example: `mock.scar` -> `/path/to/mock.scar`, SCAR_PATH contains `/path/to`
    #[test]
    #[serial(scar_path)]
    fn load_smart_contract_from_path_with_file_extension_successful() {
        let setup = UploadTestSetup::new().build();
        SabreSmartContractDefinition::new_from_scar(&setup.scar)
            .expect("failed to perform upload action");
    }

    /// Verify that a .scar file can be loaded by providing the name of a .scar file, without a
    /// file extension, that is located in one of the paths specified by the SCAR_PATH environment
    /// variable.
    /// Example: `mock` -> `/path/to/mock.scar`, SCAR_PATH contains `/path/to`
    #[test]
    #[serial(scar_path)]
    fn load_smart_contract_from_path_without_file_extension_successful() {
        let setup = UploadTestSetup::new().with_scar_without_extension().build();
        SabreSmartContractDefinition::new_from_scar(&setup.scar)
            .expect("failed to perform upload action");
    }

    /// Verify that a .scar file can be loaded by providing a full path to the .scar file.
    /// Example: `/path/to/mock.scar`
    #[test]
    #[serial(scar_path)]
    fn load_smart_contract_from_file_successful() {
        let setup = UploadTestSetup::new().with_scar_from_file().build();
        SabreSmartContractDefinition::new_from_scar(&setup.scar)
            .expect("failed to perform upload action");
    }

    /// Verify that an error is returned when attempting to load a non-existent .scar file.
    #[test]
    #[serial(scar_path)]
    fn load_smart_contract_file_not_found() {
        let setup = UploadTestSetup::new()
            .with_scar("/non_existent_dir/mock.scar".into())
            .build();
        assert!(SabreSmartContractDefinition::new_from_scar(&setup.scar).is_err());
    }

    /// Verify that an error is returned when attempting to load a .scar file from SCAR_PATH, but
    /// SCAR_PATH is not set.
    #[test]
    #[serial(scar_path)]
    fn load_smart_contract_path_not_set() {
        let setup = UploadTestSetup::new().with_scar_path_env_var(None).build();
        assert!(SabreSmartContractDefinition::new_from_scar(&setup.scar).is_err());
    }

    /// Verify that an error is returned when attempting to load a .scar file from SCAR_PATH, but
    /// the specified .scar file cannout be found in SCAR_PATH.
    #[test]
    #[serial(scar_path)]
    fn load_smart_contract_not_found_in_path() {
        let setup = UploadTestSetup::new()
            .with_scar_path_env_var(Some("".into()))
            .build();
        assert!(SabreSmartContractDefinition::new_from_scar(&setup.scar).is_err());
    }

    /// Verify that an error is returned when attempting to load a .scar file that does not contain
    /// a `manifest.yaml` file.
    #[test]
    #[serial(scar_path)]
    fn load_smart_contract_manifest_not_found() {
        let setup = UploadTestSetup::new()
            .with_manifest::<SabreSmartContractMetadata>(None)
            .build();
        assert!(SabreSmartContractDefinition::new_from_scar(&setup.scar).is_err());
    }

    /// Verify that an error is returned when attempting to load a .scar file whose `manifest.yaml`
    /// is invalidly formatted.
    #[test]
    #[serial(scar_path)]
    fn load_smart_contract_manifest_invalid() {
        let setup = UploadTestSetup::new().with_manifest(Some("")).build();
        assert!(SabreSmartContractDefinition::new_from_scar(&setup.scar).is_err());
    }

    /// Verify that an error is returned when attempting to load a .scar file that does not contain
    /// a .wasm smart contract.
    #[test]
    #[serial(scar_path)]
    fn load_smart_contract_contract_not_found() {
        let setup = UploadTestSetup::new().set_contract(false).build();
        assert!(SabreSmartContractDefinition::new_from_scar(&setup.scar).is_err());
    }

    /// Builder for setting up the test environment. By default, the builder will create a valid
    /// environment for loading a .scar file from SCAR_PATH with the filename + extension of the
    /// .scar file.
    struct UploadTestSetup {
        temp_dir: TempDir,
        set_contract: bool,
        manifest: Option<Vec<u8>>,
        scar_path_env_var: Option<String>,
        scar: String,
    }

    impl UploadTestSetup {
        fn new() -> Self {
            let temp_dir = new_temp_dir();
            let scar_path_env_var = temp_dir.path().to_string_lossy().into_owned();
            let scar = MOCK_SCAR_FILENAME.into();
            Self {
                temp_dir,
                set_contract: true,
                manifest: Some(
                    serde_yaml::to_vec(&get_mock_smart_contract_metadata())
                        .expect("failed to serialize manifest"),
                ),
                scar_path_env_var: Some(scar_path_env_var),
                scar,
            }
        }

        fn set_contract(mut self, set_contract: bool) -> Self {
            self.set_contract = set_contract;
            self
        }

        fn with_manifest<T: Serialize>(mut self, manifest: Option<T>) -> Self {
            self.manifest = manifest.map(|manifest| {
                serde_yaml::to_vec(&manifest).expect("failed to serialize manifest")
            });
            self
        }

        fn with_scar_path_env_var(mut self, scar_path_env_var: Option<String>) -> Self {
            self.scar_path_env_var = scar_path_env_var;
            self
        }

        fn with_scar_from_file(mut self) -> Self {
            self.scar = self
                .temp_dir
                .path()
                .join(MOCK_SCAR_FILENAME)
                .to_string_lossy()
                .into_owned();
            self
        }

        fn with_scar_without_extension(mut self) -> Self {
            self.scar = MOCK_SCAR_FILENAME
                .split(".")
                .next()
                .expect("failed to get stem from mock scar filename")
                .into();
            self
        }

        fn with_scar(mut self, scar: String) -> Self {
            self.scar = scar;
            self
        }

        fn build(self) -> SetupHandle {
            match self.scar_path_env_var {
                Some(scar_path_env_var) => std::env::set_var(SCAR_PATH_ENV_VAR, scar_path_env_var),
                None => std::env::remove_var(SCAR_PATH_ENV_VAR),
            }

            add_mock_scar_to_dir(self.temp_dir.path(), self.manifest, self.set_contract);

            SetupHandle {
                _temp_dir: self.temp_dir,
                scar: self.scar,
            }
        }
    }

    /// This handle is used to keep the temp directory open (since it is removed when dropped) and
    /// to provide the value of the `scar` argument for testing.
    struct SetupHandle {
        _temp_dir: TempDir,
        scar: String,
    }

    fn new_temp_dir() -> TempDir {
        let thread_id = format!("{:?}", std::thread::current().id());
        TempDir::new(&thread_id).expect("failed to create temp dir")
    }

    /// Add a mock .scar file to the given directory, with the given manifest file (as bytes) and
    /// with or without a mock contract (as specified by `add_contract`).
    fn add_mock_scar_to_dir(dir: &Path, manifest: Option<Vec<u8>>, add_contract: bool) {
        let scar_file_path = dir.join(MOCK_SCAR_FILENAME);
        let scar = File::create(scar_file_path.as_path()).expect("failed to create scar file");
        let mut scar_builder = Builder::new(BzEncoder::new(scar, Compression::Default));

        if let Some(manifest) = manifest {
            let manifest_file_path = dir.join(MANIFEST_FILENAME);
            let mut manifest_file =
                File::create(manifest_file_path.as_path()).expect("failed to create manifest file");
            manifest_file
                .write_all(manifest.as_slice())
                .expect("failed to write manifest file");
            scar_builder
                .append_path_with_name(manifest_file_path, MANIFEST_FILENAME)
                .expect("failed to add manifest to scar file");
        }

        if add_contract {
            let contract_file_path = dir.join(MOCK_CONTRACT_FILENAME);
            let mut contract_file =
                File::create(contract_file_path.as_path()).expect("failed to create contract file");
            contract_file
                .write_all(MOCK_CONTRACT_BYTES)
                .expect("failed to write contract file");
            scar_builder
                .append_path_with_name(contract_file_path, MOCK_CONTRACT_FILENAME)
                .expect("failed to add contract to scar file");
        }

        scar_builder.finish().expect("failed to write scar file");
    }

    fn get_mock_smart_contract_metadata() -> SabreSmartContractMetadata {
        SabreSmartContractMetadata {
            name: "mock".into(),
            version: "1.0".into(),
            inputs: vec!["abcdef".into()],
            outputs: vec!["012345".into()],
        }
    }
}
