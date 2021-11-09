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
            .mode(0o600)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempdir::TempDir;

    /// Validate that create_key_pair skips when the keypairs already exist
    #[test]
    fn create_keypair_mode_skip_skips_existing_keypairs() {
        let temp_dir = TempDir::new("test_files_exist").expect("Failed to create temp dir");

        let public_key_path = temp_dir.path().join("public_key");
        let private_key_path = temp_dir.path().join("private_key");

        File::create(public_key_path.clone())
            .expect("Failed to create file")
            .write_all(b"test-pubkey")
            .expect("Could not write file");

        File::create(private_key_path.clone())
            .expect("Failed to create file")
            .write_all(b"test-privkey")
            .expect("Could not write file");

        create_key_pair(
            temp_dir.path(),
            private_key_path,
            public_key_path.clone(),
            ConflictStrategy::Skip,
            true,
        )
        .expect("Could not create keypair");

        let mut contents = String::new();
        File::open(public_key_path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        assert_eq!(contents, "test-pubkey");
    }

    /// Validate that create_key_pair fails if the public key exists, but the private key is
    /// missing
    #[test]
    fn create_keypair_mode_skip_fails_missing_private_key() {
        let temp_dir = TempDir::new("test_files_exist").expect("Failed to create temp dir");

        let public_key_path = temp_dir.path().join("public_key");
        let private_key_path = temp_dir.path().join("private_key");

        File::create(public_key_path.clone())
            .expect("Failed to create file")
            .write_all(b"test-privkey")
            .expect("Could not write file");

        let result = create_key_pair(
            temp_dir.path(),
            private_key_path.clone(),
            public_key_path.clone(),
            ConflictStrategy::Skip,
            true,
        );

        assert!(result.is_err());

        let expected = format!(
            "{} already exists without a corresponding private key. \
                     Rerun with --force to overwrite existing files",
            public_key_path.as_path().display(),
        );

        match result.unwrap_err() {
            CliError::UserError(message) => {
                assert_eq!(message, expected);
            }
            clierror => panic!(
                "received unexpected result {}, expected {}",
                clierror, expected
            ),
        }
    }

    /// Validate that create_key_pair fails if the private key exists, but the public key is
    /// missing
    #[test]
    fn create_keypair_mode_skip_fails_missing_public_key() {
        let temp_dir = TempDir::new("test_files_exist").expect("Failed to create temp dir");

        let public_key_path = temp_dir.path().join("public_key");
        let private_key_path = temp_dir.path().join("private_key");

        File::create(private_key_path.clone())
            .expect("Failed to create file")
            .write_all(b"test-privkey")
            .expect("Could not write file");

        let result = create_key_pair(
            temp_dir.path(),
            private_key_path.clone(),
            public_key_path.clone(),
            ConflictStrategy::Skip,
            true,
        );

        assert!(result.is_err());

        let expected = format!(
            "{} already exists without a corresponding public key. \
                     Rerun with --force to overwrite existing files",
            private_key_path.as_path().display(),
        );

        match result.unwrap_err() {
            CliError::UserError(message) => {
                assert_eq!(message, expected);
            }
            clierror => panic!(
                "received unexpected result {}, expected CliError::UserError({})",
                clierror, expected
            ),
        }
    }

    /// Validate that create_key_pair writes new keys successfully on a success condition
    #[test]
    fn create_keypair_mode_skip_successfully_writes_new_keys() {
        let temp_dir = TempDir::new("test_files_exist").expect("Failed to create temp dir");

        let public_key_path = temp_dir.path().join("public_key");
        let private_key_path = temp_dir.path().join("private_key");

        create_key_pair(
            temp_dir.path(),
            private_key_path.clone(),
            public_key_path.clone(),
            ConflictStrategy::Skip,
            true,
        )
        .unwrap();

        assert!(public_key_path.exists());
        assert!(private_key_path.exists());
    }

    /// Validate that create_key_pair force option will overwrite an existing pubkey
    #[test]
    fn create_keypair_mode_force_returns_different_pubkey() {
        let temp_dir = TempDir::new("test_files_exist").expect("Failed to create temp dir");

        let public_key_path = temp_dir.path().join("public_key");
        let private_key_path = temp_dir.path().join("private_key");

        let public_key_content = "test-pubkey";

        File::create(public_key_path.clone())
            .expect("Failed to create file")
            .write_all(public_key_content.as_bytes())
            .expect("Could not write file");

        File::create(private_key_path.clone())
            .expect("Failed to create file")
            .write_all(b"test-privkey")
            .expect("Could not write file");

        create_key_pair(
            temp_dir.path(),
            private_key_path.clone(),
            public_key_path.clone(),
            ConflictStrategy::Force,
            true,
        )
        .unwrap();

        let mut contents = String::new();
        File::open(public_key_path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        assert_ne!(
            contents, public_key_content,
            "result must not be equal to existing pubkey"
        );
    }

    /// Validate that create_key_pair force successfully creates both a private and public key
    #[test]
    fn create_keypair_mode_force_successfully_writes_new_keys() {
        let temp_dir = TempDir::new("test_files_exist").expect("Failed to create temp dir");

        let public_key_path = temp_dir.path().join("public_key");
        let private_key_path = temp_dir.path().join("private_key");

        create_key_pair(
            temp_dir.path(),
            private_key_path.clone(),
            public_key_path.clone(),
            ConflictStrategy::Force,
            true,
        )
        .unwrap();

        assert!(public_key_path.exists());
        assert!(private_key_path.exists());
    }

    /// Validate that create_key_pair Error fails on existing keypairs
    #[test]
    fn create_keypair_mode_fail_fails_on_existing_keypairs() {
        let temp_dir = TempDir::new("test_files_exist").expect("Failed to create temp dir");

        let public_key_path = temp_dir.path().join("public_key");
        let private_key_path = temp_dir.path().join("private_key");

        let public_key_content = b"test-privkey";

        File::create(public_key_path.clone())
            .expect("Failed to create file")
            .write_all(public_key_content)
            .expect("Could not write file");

        File::create(private_key_path.clone())
            .expect("Failed to create file")
            .write_all(b"test-privkey")
            .expect("Could not write file");

        let result = create_key_pair(
            temp_dir.path(),
            private_key_path.clone(),
            public_key_path.clone(),
            ConflictStrategy::Error,
            true,
        );

        assert!(
            result.is_err(),
            "result must be an error if one of the keypairs exists"
        );

        let expected = format!(
            "Key files already exist: {} and {}. Rerun with --force to \
                     overwrite existing files",
            public_key_path.as_path().display(),
            private_key_path.as_path().display(),
        );

        match result.unwrap_err() {
            CliError::UserError(message) => {
                assert_eq!(message, expected);
            }
            clierror => panic!(
                "received unexpected result {}, expected CliError::UserError({})",
                clierror, expected
            ),
        }
    }

    /// Validate that create_key_pair Error succeeds if there are no existing keypairs
    #[test]
    fn create_keypair_mode_fail_successfully_writes_new_keys() {
        let temp_dir = TempDir::new("test_files_exist").expect("Failed to create temp dir");

        let public_key_path = temp_dir.path().join("public_key");
        let private_key_path = temp_dir.path().join("private_key");

        create_key_pair(
            temp_dir.path(),
            private_key_path.clone(),
            public_key_path.clone(),
            ConflictStrategy::Error,
            true,
        )
        .unwrap();

        assert!(public_key_path.exists());
        assert!(private_key_path.exists());
    }

    /// Validate that generate_keys successfully generates a public and private key in the correct
    /// path
    #[test]
    fn generate_keys_succeeds() {
        let temp_dir = TempDir::new("test_files_exist").expect("Failed to create temp dir");

        let public_key_path = temp_dir.path().join("grid_key.pub");
        let private_key_path = temp_dir.path().join("grid_key.priv");

        generate_keys(
            "grid_key".to_string(),
            ConflictStrategy::Error,
            temp_dir.path().to_path_buf(),
        )
        .unwrap();

        assert!(public_key_path.exists());
        assert!(private_key_path.exists());
    }
}
