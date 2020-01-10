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

use std::fs::{create_dir_all, metadata, OpenOptions};
use std::io::prelude::*;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(not(target_os = "linux"))]
use std::os::unix::fs::MetadataExt;

use clap::ArgMatches;
use sawtooth_sdk::signing;
use whoami;

use crate::error::CliError;

use super::{chown, Action};

const ADMIN_KEY_PATH: &str = "/etc/splinter/keys";

pub struct KeyGenAction;

impl Action for KeyGenAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;

        let key_name = args
            .value_of("key-name")
            .map(String::from)
            .unwrap_or_else(whoami::username);
        let key_dir = if args.is_present("admin") {
            PathBuf::from(ADMIN_KEY_PATH)
        } else {
            dirs::home_dir()
                .map(|mut p| {
                    p.push("splinter/keys");
                    p
                })
                .ok_or_else(|| CliError::EnvironmentError("Home directory not found".into()))?
        };

        create_dir_all(key_dir.as_path()).map_err(|err| {
            CliError::EnvironmentError(format!("Failed to create keys directory: {}", err))
        })?;

        let private_key_path = key_dir.join(&key_name).with_extension("priv");
        let public_key_path = key_dir.join(&key_name).with_extension("pub");

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

/// Creates a public/private key pair.
///
/// Returns the public key in hex, if successful.
pub fn create_key_pair(
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

        let private_key_file = OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o640)
            .open(private_key_path.as_path())
            .map_err(|err| CliError::EnvironmentError(format!("{}", err)))?;

        private_key_file
            .write(private_key.as_hex().as_bytes())
            .map_err(|err| CliError::EnvironmentError(format!("{}", err)))?;
    }

    {
        if public_key_path.exists() {
            info!("overwriting file: {:?}", public_key_path);
        } else {
            info!("writing file: {:?}", public_key_path);
        }

        let public_key_file = OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o644)
            .open(public_key_path.as_path())
            .map_err(|err| CliError::EnvironmentError(format!("{}", err)))?;

        public_key_file
            .write(public_key.as_hex().as_bytes())
            .map_err(|err| CliError::EnvironmentError(format!("{}", err)))?;
    }
    if change_permissions {
        chown(private_key_path.as_path(), key_dir_uid, key_dir_gid)?;
        chown(public_key_path.as_path(), key_dir_uid, key_dir_gid)?;
    }

    Ok(public_key.as_slice().to_vec())
}
