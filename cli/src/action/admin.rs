// Copyright 2019 Cargill Incorporated
// Copyright 2018 Intel Corporation
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

use std::collections::BTreeMap;
use std::env;
use std::fs::{metadata, File, OpenOptions};
use std::io::prelude::*;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(not(target_os = "linux"))]
use std::os::unix::fs::MetadataExt;

use clap::ArgMatches;
use sawtooth_sdk::signing;
use serde::{Deserialize, Serialize};
use splinter::keys::{storage::StorageKeyRegistry, KeyInfo, KeyRegistry};

use crate::error::CliError;

use super::{chown, Action};

const DEFAULT_STATE_DIR: &str = "/var/lib/splinter/";
const STATE_DIR_ENV: &str = "SPLINTER_STATE_DIR";

pub struct KeyGenAction;

impl Action for KeyGenAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;

        let key_name = args.value_of("key_name").unwrap_or("splinter");
        let key_dir = args
            .value_of("key_dir")
            .or(Some("."))
            .map(Path::new)
            .unwrap();

        let private_key_path = key_dir.join(key_name).with_extension("priv");
        let public_key_path = key_dir.join(key_name).with_extension("pub");

        create_key_pair(
            &key_dir,
            private_key_path,
            public_key_path,
            args.is_present("force"),
            true,
        )?;

        Ok(())
    }
}

pub struct KeyRegistryGenerationAction;

impl Action for KeyRegistryGenerationAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;

        let registry_spec_path = args
            .value_of("registry_spec_path")
            .or(Some("./key_registry_spec.yaml"))
            .map(Path::new)
            .unwrap();

        let target_dir_path = args
            .value_of("target_dir")
            .map(ToOwned::to_owned)
            .or_else(|| env::var(STATE_DIR_ENV).ok())
            .or_else(|| Some(DEFAULT_STATE_DIR.to_string()))
            .unwrap();
        let target_dir = Path::new(&target_dir_path);

        let registry_target_file = args
            .value_of("registry_file")
            .or(Some("keys.yaml"))
            .unwrap();

        let force_write = args.is_present("force");

        let target_registry_path = target_dir.join(registry_target_file);
        let registry_file_exists = target_registry_path.exists();
        if registry_file_exists {
            if !force_write {
                return Err(CliError::EnvironmentError(format!(
                    "file exists: {}",
                    target_registry_path.display()
                )));
            } else {
                std::fs::remove_file(&target_registry_path).map_err(|err| {
                    CliError::EnvironmentError(format!(
                        "Unable to overwrite {}: {}",
                        target_registry_path.display(),
                        err
                    ))
                })?;
            }
        }

        let registry_spec_file = File::open(&registry_spec_path).map_err(|err| {
            CliError::ActionError(format!(
                "Unable to open key registry spec {}: {}",
                registry_spec_path.display(),
                err
            ))
        })?;

        let registry_spec: KeyRegistrySpec =
            serde_yaml::from_reader(registry_spec_file).map_err(|err| {
                CliError::ActionError(format!(
                    "Unable to read key registry {}: {}",
                    registry_spec_path.display(),
                    err
                ))
            })?;

        let mut key_registry = StorageKeyRegistry::new(
            target_registry_path
                .as_os_str()
                .to_str()
                .ok_or_else(|| {
                    CliError::EnvironmentError(format!(
                        "Key registry output file {} contains invalid characters",
                        target_registry_path.display()
                    ))
                })?
                .to_string(),
        )
        .map_err(|err| {
            CliError::EnvironmentError(format!(
                "Unable to read {}: {}",
                target_registry_path.display(),
                err
            ))
        })?;

        if registry_file_exists {
            info!("overwriting file \"{}\"", target_registry_path.display());
        } else {
            info!("writing file \"{}\"", target_registry_path.display());
        }

        let mut key_infos = vec![];
        for (key_name, key_spec) in registry_spec.keys.into_iter() {
            let private_key_path = target_dir.join(&key_name).with_extension("priv");
            let public_key_path = target_dir.join(&key_name).with_extension("pub");

            let public_key = create_key_pair(
                &target_dir,
                private_key_path,
                public_key_path,
                force_write,
                false,
            )?;

            let mut key_info_builder = KeyInfo::builder(public_key, key_spec.node_id);
            for (meta_key, meta_value) in key_spec.metadata.into_iter() {
                key_info_builder = key_info_builder.with_metadata(meta_key, meta_value);
            }

            key_infos.push(key_info_builder.build());
        }

        key_registry
            .save_keys(key_infos)
            .map_err(|err| CliError::ActionError(format!("Unable to write keys: {}", err)))?;

        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
struct KeyRegistrySpec {
    #[serde(flatten)]
    keys: BTreeMap<String, KeySpec>,
}

#[derive(Deserialize, Serialize)]
struct KeySpec {
    node_id: String,
    #[serde(default = "BTreeMap::new")]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    metadata: BTreeMap<String, String>,
}

/// Creates a public/private key pair.
///
/// Returns the public key in hex, if successful.
fn create_key_pair(
    key_dir: &Path,
    private_key_path: PathBuf,
    public_key_path: PathBuf,
    force_create: bool,
    change_permissions: bool,
) -> Result<Vec<u8>, CliError> {
    if !force_create {
        if private_key_path.exists() {
            return Err(CliError::EnvironmentError(format!(
                "file exists: {:?}",
                private_key_path
            )));
        }
        if public_key_path.exists() {
            return Err(CliError::EnvironmentError(format!(
                "file exists: {:?}",
                public_key_path
            )));
        }
    }

    let context = signing::create_context("secp256k1")
        .map_err(|err| CliError::EnvironmentError(format!("{}", err)))?;

    let private_key = context
        .new_random_private_key()
        .map_err(|err| CliError::EnvironmentError(format!("{}", err)))?;
    let public_key = context
        .get_public_key(&*private_key)
        .map_err(|err| CliError::EnvironmentError(format!("{}", err)))?;

    let key_dir_info =
        metadata(key_dir).map_err(|err| CliError::EnvironmentError(format!("{}", err)))?;

    #[cfg(not(target_os = "linux"))]
    let (key_dir_uid, key_dir_gid) = (key_dir_info.uid(), key_dir_info.gid());
    #[cfg(target_os = "linux")]
    let (key_dir_uid, key_dir_gid) = (key_dir_info.st_uid(), key_dir_info.st_gid());

    {
        if private_key_path.exists() {
            info!("overwriting file: {:?}", private_key_path);
        } else {
            info!("writing file: {:?}", private_key_path);
        }

        let mut private_key_file = OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o640)
            .open(private_key_path.as_path())
            .map_err(|err| CliError::EnvironmentError(format!("{}", err)))?;
        write_hex_to_file(&private_key.as_hex(), &mut private_key_file)?;
    }

    {
        if public_key_path.exists() {
            info!("overwriting file: {:?}", public_key_path);
        } else {
            info!("writing file: {:?}", public_key_path);
        }

        let mut public_key_file = OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o644)
            .open(public_key_path.as_path())
            .map_err(|err| CliError::EnvironmentError(format!("{}", err)))?;
        write_hex_to_file(&public_key.as_hex(), &mut public_key_file)?;
    }
    if change_permissions {
        chown(private_key_path.as_path(), key_dir_uid, key_dir_gid)?;
        chown(public_key_path.as_path(), key_dir_uid, key_dir_gid)?;
    }

    Ok(public_key.as_slice().to_vec())
}

/// Write the given hex string to the given file, appending a newline at the end.
fn write_hex_to_file(hex: &str, file: &mut File) -> Result<(), CliError> {
    writeln!(file, "{}", hex).map_err(|err| CliError::EnvironmentError(format!("{}", err)))
}
