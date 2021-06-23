// Copyright 2018-2021 Cargill Incorporated
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

use std::{env, path::Path, path::PathBuf};

use cylinder::{
    current_user_key_name, current_user_search_path, load_key, load_key_from_path,
    secp256k1::Secp256k1Context, Context, PrivateKey, Signer,
};

use crate::error::CliError;

// If the `CYLINDER_PATH` environment variable is not set, add `$HOME/.grid/keys`
// to the vector of paths to search. This is for backwards compatibility.
fn grid_user_search_path() -> Vec<PathBuf> {
    match env::var("CYLINDER_PATH") {
        Ok(_) => current_user_search_path(),
        Err(_) => {
            let mut grid_path = match dirs::home_dir() {
                Some(dir) => dir,
                None => Path::new(".").to_path_buf(),
            };
            grid_path.push(".grid");
            grid_path.push("keys");
            let mut paths = current_user_search_path();
            paths.push(grid_path);
            paths
        }
    }
}

fn load_private_key(key_name: Option<String>) -> Result<PrivateKey, CliError> {
    let private_key = if let Some(key_name) = key_name {
        if key_name.contains('/') {
            load_key_from_path(Path::new(&*key_name))
                .map_err(|err| CliError::ActionError(err.to_string()))?
        } else {
            let path = grid_user_search_path();
            load_key(&*key_name, &path)
                .map_err(|err| CliError::ActionError(err.to_string()))?
                .ok_or_else(|| {
                    CliError::ActionError({
                        format!(
                            "No signing key found in {}. Either specify the --key argument or \
                            generate the default key via grid keygen",
                            path.iter()
                                .map(|path| path.as_path().display().to_string())
                                .collect::<Vec<String>>()
                                .join(":")
                        )
                    })
                })?
        }
    } else {
        let path = grid_user_search_path();
        load_key(&current_user_key_name(), &path)
            .map_err(|err| CliError::ActionError(err.to_string()))?
            .ok_or_else(|| {
                CliError::ActionError({
                    format!(
                        "No signing key found in {}. Either specify the --key argument or \
                        generate the default key via grid keygen",
                        path.iter()
                            .map(|path| path.as_path().display().to_string())
                            .collect::<Vec<String>>()
                            .join(":")
                    )
                })
            })?
    };

    Ok(private_key)
}

pub fn load_signer(key_name: Option<String>) -> Result<Box<dyn Signer>, CliError> {
    Ok(Secp256k1Context::new().new_signer(load_private_key(key_name)?))
}
