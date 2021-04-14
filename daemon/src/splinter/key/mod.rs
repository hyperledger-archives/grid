/*
 * Copyright 2020 Cargill Incorporated
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

mod error;

use std::io::Read;
use std::{fs::File, path::Path};

pub use error::KeyError;

pub fn load_scabbard_admin_key(key_dir: &str) -> Result<String, KeyError> {
    let private_key_filename = format!("{}/gridd.priv", key_dir);
    let private_key_path = Path::new(&private_key_filename);
    if !private_key_path.exists() {
        return Err(KeyError(format!(
            "No such private key file: {}",
            private_key_path.display()
        )));
    }
    let private_key = read_key_from_file(private_key_filename)?;

    Ok(private_key)
}

fn read_key_from_file(filename: String) -> Result<String, KeyError> {
    let mut f = File::open(&filename)?;

    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    let key_str = match contents.lines().next() {
        Some(k) => k,
        None => {
            return Err(KeyError(format!("Empty key file: {}", filename)));
        }
    };

    Ok(key_str.to_string())
}
