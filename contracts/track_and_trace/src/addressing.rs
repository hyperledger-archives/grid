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

use crypto::digest::Digest;
use crypto::sha2::Sha512;

const FAMILY_NAME: &str = "grid_track_and_trace";
const PROPERTY: &str = "ea";
const PROPOSAL: &str = "aa";
const RECORD: &str = "ec";
const GRID_NAMESPACE: &str = "621dee";
const GRID_SCHEMA_NAMESPACE: &str = "01";
const PIKE_NAMESPACE: &str = "cad11d";
const PIKE_AGENT_NAMESPACE: &str = "00";

pub fn get_track_and_trace_prefix() -> String {
    let mut sha = Sha512::new();
    sha.input_str(&FAMILY_NAME);
    sha.result_str()[..6].to_string()
}

pub fn get_grid_prefix() -> String {
    GRID_NAMESPACE.to_string()
}

pub fn get_pike_prefix() -> String {
    PIKE_NAMESPACE.to_string()
}

pub fn hash(to_hash: &str, num: usize) -> String {
    let mut sha = Sha512::new();
    sha.input_str(to_hash);
    let temp = sha.result_str();
    let hash = match temp.get(..num) {
        Some(x) => x,
        None => "",
    };
    hash.to_string()
}

pub fn make_record_address(record_id: &str) -> String {
    get_track_and_trace_prefix() + RECORD + &hash(record_id, 62)
}

pub fn make_property_address(record_id: &str, property_name: &str, page: u32) -> String {
    make_property_address_range(record_id) + &hash(property_name, 22) + &num_to_page_number(page)
}

pub fn make_property_address_range(record_id: &str) -> String {
    get_track_and_trace_prefix() + PROPERTY + &hash(record_id, 36)
}

/// Computes the address a Grid Schema is stored at based on its name
pub fn make_schema_address(name: &str) -> String {
    let mut sha = Sha512::new();
    sha.input(name.as_bytes());
    String::from(GRID_NAMESPACE) + GRID_SCHEMA_NAMESPACE + &sha.result_str()[..62].to_string()
}

/// Computes the address a Pike Agent is stored at based on its public_key
pub fn make_agent_address(public_key: &str) -> String {
    let mut sha = Sha512::new();
    sha.input(public_key.as_bytes());

    String::from(PIKE_NAMESPACE) + PIKE_AGENT_NAMESPACE + &sha.result_str()[..62].to_string()
}

pub fn num_to_page_number(page: u32) -> String {
    format!("{:01$x}", page, 4)
}

pub fn make_proposal_address(record_id: &str, agent_id: &str) -> String {
    get_track_and_trace_prefix() + PROPOSAL + &hash(record_id, 36) + &hash(agent_id, 26)
}
