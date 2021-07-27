/*
 * Copyright 2018 Intel Corporation
 * Copyright 2019 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * ------------------------------------------------------------------------------
 */

use std::fs::{create_dir_all, metadata, OpenOptions};
use std::io::prelude::*;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
#[cfg(not(target_os = "linux"))]
use std::os::unix::fs::MetadataExt;

use crate::error::CliError;
use cylinder::{secp256k1::Secp256k1Context, Context};

use super::chown;

pub enum ConflictStrategy {
    Force,
    Skip,
    Error,
}

pub fn create_key_pair(
    key_dir: &Path,
    private_key_path: PathBuf,
    public_key_path: PathBuf,
    conflict_strategy: ConflictStrategy,
    change_permissions: bool,
) -> Result<(), CliError> {
    match conflict_strategy {
        ConflictStrategy::Force => (),
        ConflictStrategy::Skip => {
            if public_key_path.exists() && !private_key_path.exists() {
                return Err(CliError::UserError(format!(
                    "{} already exists without a corresponding private key. \
                     Rerun with --force to overwrite existing files",
                    public_key_path.as_path().display(),
                )));
            }

            if !public_key_path.exists() && private_key_path.exists() {
                return Err(CliError::UserError(format!(
                    "{} already exists without a corresponding public key. \
                     Rerun with --force to overwrite existing files",
                    private_key_path.as_path().display(),
                )));
            }

            if public_key_path.exists() && private_key_path.exists() {
                debug!("keys files exist; skipping generation");
                return Ok(());
            }
        }
        ConflictStrategy::Error => {
            if public_key_path.exists() || private_key_path.exists() {
                return Err(CliError::UserError(format!(
                    "Key files already exist: {} and {}. Rerun with --force to \
                     overwrite existing files",
                    public_key_path.as_path().display(),
                    private_key_path.as_path().display(),
                )));
            }
        }
    }

    let context = Secp256k1Context::new();

    let private_key = context.new_random_private_key();
    let public_key = context
        .get_public_key(&private_key)
        .map_err(|err| CliError::UserError(format!("Failed to get public key: {}", err)))?;

    let key_dir_info = metadata(key_dir).map_err(|err| {
        CliError::UserError(format!(
            "Failed to read key directory '{}': {}",
            key_dir.display(),
            err
        ))
    })?;

    #[cfg(not(target_os = "linux"))]
    let (key_dir_uid, key_dir_gid) = (key_dir_info.uid(), key_dir_info.gid());
    #[cfg(target_os = "linux")]
    let (key_dir_uid, key_dir_gid) = (key_dir_info.st_uid(), key_dir_info.st_gid());

    {
        if private_key_path.exists() {
            info!(
                "Overwriting private key file: {}",
                private_key_path.display()
            );
        } else {
            info!("Writing private key file: {}", private_key_path.display());
        }

        let private_key_file = OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o640)
            .open(private_key_path.as_path())
            .map_err(|err| {
                CliError::UserError(format!(
                    "Failed to open private key file '{}': {}",
                    private_key_path.display(),
                    err
                ))
            })?;

        writeln!(&private_key_file, "{}", private_key.as_hex()).map_err(|err| {
            CliError::UserError(format!(
                "Failed to write to private key file '{}': {}",
                private_key_path.display(),
                err
            ))
        })?;
    }

    {
        if public_key_path.exists() {
            info!("Overwriting public key file: {}", public_key_path.display());
        } else {
            info!("Writing public key file: {}", public_key_path.display());
        }

        let public_key_file = OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o644)
            .open(public_key_path.as_path())
            .map_err(|err| {
                CliError::UserError(format!(
                    "Failed to open public key file '{}': {}",
                    public_key_path.display(),
                    err
                ))
            })?;

        writeln!(&public_key_file, "{}", public_key.as_hex()).map_err(|err| {
            CliError::UserError(format!(
                "Failed to write to public key file '{}': {}",
                public_key_path.display(),
                err
            ))
        })?;
    }
    if change_permissions {
        chown(private_key_path.as_path(), key_dir_uid, key_dir_gid)?;
        chown(public_key_path.as_path(), key_dir_uid, key_dir_gid)?;
    }

    Ok(())
}

/// Generates a public/private key pair that can be used to sign transactions.
/// If no directory is provided, the keys are created in the default directory
///
///   $HOME/.grid/keys/
///
/// If no key_name is provided the key name is set to USER environment variable.
pub fn generate_keys(
    key_name: String,
    conflict_strategy: ConflictStrategy,
    key_dir: PathBuf,
) -> Result<(), CliError> {
    create_dir_all(key_dir.as_path())
        .map_err(|err| CliError::UserError(format!("Failed to create keys directory: {}", err)))?;

    let private_key_path = key_dir.join(&key_name).with_extension("priv");
    let public_key_path = key_dir.join(&key_name).with_extension("pub");

    create_key_pair(
        &key_dir,
        private_key_path,
        public_key_path,
        conflict_strategy,
        true,
    )?;

    Ok(())
}
