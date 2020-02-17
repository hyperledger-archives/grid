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

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use sawtooth_sdk::signing::secp256k1::Secp256k1PrivateKey;

use super::error::CliError;

/// Load a private key from the local filesystem.
///
/// If the argument is a file path (contains a '/'), this will attempt to load the key file from
/// the specified location. If the argument is not a file path, this will attempt to load the
/// file from the $HOME/.splinter/keys directory; when loading from this directory, the '.prive'
/// file extension is optional.
pub fn load_signing_key(key: &str) -> Result<Secp256k1PrivateKey, CliError> {
    let file_path = determine_key_file_path(key)?;

    let key_file = File::open(file_path).map_err(|err| {
        CliError::action_error_with_source("failed to open private key file", err.into())
    })?;
    let mut key_file_reader = BufReader::new(key_file);

    let mut raw_key_string = String::new();
    key_file_reader
        .read_line(&mut raw_key_string)
        .map_err(|err| {
            CliError::action_error_with_source("failed to read private key file", err.into())
        })?;

    let key_string = raw_key_string.trim();
    if key_string.is_empty() {
        return Err(CliError::action_error("private key file is empty"));
    }

    let signing_key = Secp256k1PrivateKey::from_hex(key_string).map_err(|err| {
        CliError::action_error_with_source("failed to read valid private key from file", err.into())
    })?;

    Ok(signing_key)
}

fn determine_key_file_path(key: &str) -> Result<PathBuf, CliError> {
    if key.contains('/') {
        Ok(PathBuf::from(key))
    } else {
        let mut path = dirs::home_dir().ok_or_else(|| {
            CliError::action_error("failed to load signing key: unable to determine home directory")
        })?;
        path.push(".splinter");
        path.push("keys");
        path.push(key);

        if path.exists() {
            Ok(path)
        } else {
            path.set_extension("priv");
            if path.exists() {
                Ok(path)
            } else {
                Err(CliError::action_error(
                    "failed to load signing key: could not be found",
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::create_dir_all;
    use std::io::Write;
    use std::path::Path;

    use sawtooth_sdk::signing::PrivateKey;
    use serial_test::serial;
    use tempfile::{tempdir, NamedTempFile};

    const MOCK_PRIV_KEY_HEX: &str =
        "d31e395bed0d9b2277b25d57523063d7d6b9db802d80549bc1362875cdcb83c6";

    /// Verify that an error is returned when the key file is not found.
    #[test]
    fn file_not_found() {
        assert!(load_signing_key("non_existent_file.priv").is_err());
    }

    /// Verify that an error is returned when the key file is empty.
    #[test]
    fn file_empty() {
        let (_file, path) = temp_key_file("");
        assert!(load_signing_key(&path).is_err());
    }

    /// Verify that an error is returned when the file does not contain a valid key.
    #[test]
    fn key_in_file_invalid() {
        let (_file, path) = temp_key_file("not_valid_hex");
        assert!(load_signing_key(&path).is_err());
    }

    /// Verify that a key is successfully loaded from a valid key file that is specified with a
    /// full path.
    #[test]
    fn successful_with_path() {
        let (_file, path) = temp_key_file(MOCK_PRIV_KEY_HEX);
        let signing_key = load_signing_key(&path).expect("failed to get key from file");
        assert_eq!(&signing_key.as_hex(), MOCK_PRIV_KEY_HEX);
    }

    /// Verify that a key is successfully loaded from a valid key file in the $HOME/.splinter/keys
    /// directory when the key's file name, without the file extension, is specified.
    #[test]
    #[serial(home_dir)]
    fn successful_from_home_without_extension() {
        run_test_for_key_in_home(MOCK_PRIV_KEY_HEX, |key_path| {
            let file_stem = key_path
                .file_stem()
                .expect("failed to get file stem")
                .to_string_lossy()
                .into_owned();
            let signing_key = load_signing_key(&file_stem).expect("failed to get key");
            assert_eq!(&signing_key.as_hex(), MOCK_PRIV_KEY_HEX);
        })
    }

    /// Verify that a key is successfully loaded from a valid key file in the $HOME/.splinter/keys
    /// directory when the key's file name with the file extension is specified.
    #[test]
    #[serial(home_dir)]
    fn successful_from_home_with_extension() {
        run_test_for_key_in_home(MOCK_PRIV_KEY_HEX, |key_path| {
            let file_name = key_path
                .file_name()
                .expect("failed to get file name")
                .to_string_lossy()
                .into_owned();
            let signing_key = load_signing_key(&file_name).expect("failed to get key");
            assert_eq!(&signing_key.as_hex(), MOCK_PRIV_KEY_HEX);
        })
    }

    /// Create a temporary key file with the given key; return the temp file's handle and the file
    /// path.
    fn temp_key_file(key: &str) -> (NamedTempFile, String) {
        let mut file = NamedTempFile::new().expect("failed to create temp key file");
        writeln!(&mut file, "{}", key).expect("failed to write key to temp file");
        let path = file.path().to_string_lossy().into_owned();
        (file, path)
    }

    /// Create a temporary home directory, write the given key to a temporary file in
    /// $HOME/.splinter/keys, and run the given test, passing in the key file's path. When the test
    /// has been run, reset the home directory to its original value and check the test's result.
    ///
    /// NOTE: the tests that use this method must be run serially because they modify the same
    /// environment variable. This is accomplished by annotating each test with
    /// `#[serial(scar_path)]`.
    fn run_test_for_key_in_home<F>(key: &str, test: F)
    where
        F: Fn(&Path) -> () + std::panic::UnwindSafe,
    {
        let original_home = std::env::var("HOME").expect("failed to get original home dir");

        let temp_home = tempdir().expect("failed to create temp home dir");
        std::env::set_var("HOME", temp_home.path());

        let result = std::panic::catch_unwind(move || {
            let mut keys_dir_path = temp_home.path().to_path_buf();
            keys_dir_path.push(".splinter");
            keys_dir_path.push("keys");
            create_dir_all(&keys_dir_path).expect("failed to create $HOME/.splinter/keys dir");

            let key_file =
                NamedTempFile::new_in(&keys_dir_path).expect("failed to create temp key file");
            writeln!(&key_file, "{}", key).expect("failed to write key to temp file");

            test(key_file.path())
        });

        std::env::set_var("HOME", original_home);

        assert!(result.is_ok())
    }
}
