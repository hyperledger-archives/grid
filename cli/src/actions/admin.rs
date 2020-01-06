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
 * -----------------------------------------------------------------------------
 */

use std::fs::OpenOptions;
use std::io::prelude::*;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

use sawtooth_sdk::signing;

use crate::error::CliError;

const DEFAULT_KEY_DIR: &str = "/etc/grid/keys";

pub enum ConflictStrategy {
    Force,
    Skip,
    Error,
}

pub fn do_keygen(
    directory: Option<&str>,
    conflict_strategy: ConflictStrategy,
) -> Result<(), CliError> {
    let key_dir = match directory {
        Some(key_dir) => {
            if !PathBuf::from(key_dir).exists() {
                return Err(CliError::UserError(format!("{} does not exist", key_dir)));
            }
            key_dir
        }
        None => {
            if !PathBuf::from(DEFAULT_KEY_DIR).exists() {
                return Err(CliError::UserError(format!(
                    "{} does not exist; verify that you have gridd installed on this system",
                    DEFAULT_KEY_DIR
                )));
            }
            DEFAULT_KEY_DIR
        }
    };

    let public_key_path: PathBuf = [key_dir, "gridd.pub"].iter().collect();
    let private_key_path: PathBuf = [key_dir, "gridd.priv"].iter().collect();

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
                println!("Admin keys exist; skipping generation");
                return Ok(());
            }
        }
        ConflictStrategy::Error => {
            if public_key_path.exists() || private_key_path.exists() {
                return Err(CliError::UserError(format!(
                    "Key files already exist at {}. Rerun with --force to \
                     overwrite existing files",
                    key_dir
                )));
            }
        }
    }

    let context = signing::create_context("secp256k1")?;

    let private_key = context.new_random_private_key()?;
    let public_key = context.get_public_key(&*private_key)?;

    if public_key_path.exists() {
        println!("Overwriting file: {:?}", public_key_path);
    } else {
        println!("Writing file: {:?}", public_key_path);
    }
    let mut public_key_file = OpenOptions::new()
        .write(true)
        .create(true)
        .mode(0o644)
        .open(public_key_path.as_path())?;

    public_key_file.write_all(public_key.as_hex().as_bytes())?;

    if private_key_path.exists() {
        println!("Overwriting file: {:?}", private_key_path);
    } else {
        println!("Writing file: {:?}", private_key_path);
    }
    let mut private_key_file = OpenOptions::new()
        .write(true)
        .create(true)
        .mode(0o640)
        .open(private_key_path.as_path())?;

    private_key_file.write_all(private_key.as_hex().as_bytes())?;

    Ok(())
}
