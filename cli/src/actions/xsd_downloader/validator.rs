/*
 * Copyright 2022 Cargill Incorporated
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

//! Validation logic for the XSD downloader

use std::fmt::Write;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::CliError;

/// Validate the hash of a file an return a useful error message if the hash is invalid
///
/// * `file` - The file to check the hash of
/// * `hash` - The expected sha256 hash
pub fn validate_hash(file: &Path, hash: &str) -> Result<(), CliError> {
    let hash = (0..hash.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hash[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|_| CliError::InternalError(format!("error parsing file hash \"{hash}\"")))?;

    debug!("validating hash of {file}", file = file.to_string_lossy());
    if !file.exists() {
        return Err(CliError::ActionError(format!(
            "file \"{file}\" does not exist",
            file = file.to_string_lossy()
        )));
    }

    let mut hasher = Sha256::new();
    let data = std::fs::read(file).map_err(|err| CliError::InternalError(err.to_string()))?;
    hasher.update(data);
    let sha256 = hasher.finalize();

    if sha256.as_slice() != hash {
        let mut expected = String::new();
        for byte in hash {
            write!(&mut expected, "{:02x}", byte)
                .map_err(|err| CliError::InternalError(err.to_string()))?;
        }

        return Err(CliError::ActionError(format!(
            "expected file to have hash of {expected}, but it was {got:x}",
            got = sha256
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::Write;

    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    const TEST_DATA: &str = "lagomorpha";
    const TEST_HASH: &str = "9b44a9cb40096bf6767dec8e97bdc5a36ead7bc6200025cac801bf445307aba0";

    #[test]
    fn validate_hash_succeeds_on_valid_hash() {
        let temp_dir = TempDir::new().expect("could not create tempdir");
        let file_path = temp_dir.path().join("gs1.zip");

        let mut output = File::create(&file_path).expect("could not create file");
        write!(output, "{}", TEST_DATA).expect("could not write file");

        assert_eq!(
            format!("{:?}", validate_hash(&file_path, TEST_HASH)),
            "Ok(())"
        );
    }

    #[test]
    fn validate_hash_fails_on_invalid_hash() {
        let temp_dir = TempDir::new().expect("could not create tempdir");
        let file_path = temp_dir.path().join("gs1.zip");

        let mut output = File::create(&file_path).expect("could not create file");
        write!(output, "deus ex machina").expect("could not write file");

        assert_eq!(
            format!("{:?}", validate_hash(&file_path, TEST_HASH)), 
            "Err(ActionError(\"expected file to have hash of 9b44a9cb40096bf6767dec8e97bdc5a36ead7bc6200025cac801bf445307aba0, but it was 79909d2507886e03b19dded20d453c048a9e2f05f2e3553e4a399926505df260\"))".to_string()
            );
    }

    #[test]
    fn validate_hash_returns_error_if_file_does_not_exist() {
        let file_path = Path::new("fakefile.zip");
        assert_eq!(
            format!("{:?}", validate_hash(file_path, TEST_HASH)),
            "Err(ActionError(\"file \\\"fakefile.zip\\\" does not exist\"))".to_string()
        );
    }
}
