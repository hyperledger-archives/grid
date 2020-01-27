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
use serde_json::Value;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct GetNodeError(pub String);

impl Error for GetNodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl fmt::Display for GetNodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn get_node_id(splinterd_url: String) -> Result<String, GetNodeError> {
    let uri = format!("{}/status", splinterd_url);

    let body: Value = reqwest::blocking::get(&uri)
        .map_err(|err| GetNodeError(format!("Failed to get set up request: {}", err)))?
        .json()
        .map_err(|err| GetNodeError(format!("Failed to parse response body: {}", err)))?;

    let node_id_val = body
        .get("node_id")
        .ok_or_else(|| GetNodeError("Node status response did not contain a node ID".into()))?;

    let node_id = node_id_val
        .as_str()
        .ok_or_else(|| GetNodeError("Node status returned an invalid ID".into()))?;

    Ok(node_id.to_string())
}
