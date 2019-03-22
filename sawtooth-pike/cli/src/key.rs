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


//! Contains functions which assist with signing key management

use std::env;
use std::io::prelude::*;
use std::fs::File;

use users::get_current_username;

use sawtooth_sdk::signing::secp256k1::Secp256k1PrivateKey;

use error::CliError;

/// Return a signing key loaded from the user's environment
///
/// This method attempts to load the user's key from a file.  The filename
/// is constructed by appending ".priv" to the key's name.  If the name argument
/// is None, then the USER environment variable is used in its place.
///
/// The directory containing the keys is determined using the HOME
/// environment variable:
///
///   $HOME/.sawtooth/keys/
///
/// # Arguments
///
/// * `name` - The name of the signing key, which is used to construct the
///            key's filename
///
/// # Errors
///
/// If a signing error occurs, a CliError::SigningError is returned.
///
/// If a HOME or USER environment variable is required but cannot be
/// retrieved from the environment, a CliError::VarError is returned.
pub fn load_signing_key(name: Option<&str>) -> Result<Secp256k1PrivateKey, CliError> {
    let username: String = name.map(|s| String::from(s))
        .ok_or_else(|| env::var("USER"))
        .or_else(|_| get_current_username().ok_or(0))
        .map_err(|_| {
            CliError::UserError(String::from(
                "Could not load signing key: unable to determine username",
            ))
        })?;

    let private_key_filename = env::home_dir()
        .ok_or(CliError::UserError(String::from(
            "Could not load signing key: unable to determine home directory",
        )))
        .and_then(|mut p| {
            p.push(".sawtooth");
            p.push("keys");
            p.push(format!("{}.priv", &username));
            Ok(p)
        })?;

    if !private_key_filename.as_path().exists() {
        return Err(CliError::UserError(format!(
            "No such key file: {}",
            private_key_filename.display()
        )));
    }

    let mut f = File::open(&private_key_filename)?;

    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    let key_str = match contents.lines().next() {
        Some(k) => k,
        None => {
            return Err(CliError::UserError(format!(
                "Empty key file: {}",
                private_key_filename.display()
            )))
        }
    };

    Ok(Secp256k1PrivateKey::from_hex(&key_str)?)
}
