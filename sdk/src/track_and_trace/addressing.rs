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

use crypto::digest::Digest;
use crypto::sha2::Sha512;

pub const TRACK_AND_TRACE_NAMESPACE: &str = "a43b46";
pub const PROPERTY: &str = "ea";
pub const PROPOSAL: &str = "aa";
pub const RECORD: &str = "ec";
pub const TRACK_AND_TRACE_PROPERTY_NAMESPACE: &str = "a43b46ea";
pub const TRACK_AND_TRACE_PROPOSAL_NAMESPACE: &str = "a43b46aa";
pub const TRACK_AND_TRACE_RECORD_NAMESPACE: &str = "a43b46ec";

fn hash(to_hash: &str, num: usize) -> String {
    let mut sha = Sha512::new();
    sha.input_str(to_hash);
    let temp = sha.result_str();
    let hash = temp.get(..num).unwrap_or("");
    hash.to_string()
}

pub fn make_record_address(record_id: &str) -> String {
    String::from(TRACK_AND_TRACE_NAMESPACE) + RECORD + &hash(record_id, 62)
}

pub fn make_property_address(record_id: &str, property_name: &str, page: u32) -> String {
    make_property_address_range(record_id) + &hash(property_name, 22) + &num_to_page_number(page)
}

pub fn make_property_address_range(record_id: &str) -> String {
    String::from(TRACK_AND_TRACE_NAMESPACE) + PROPERTY + &hash(record_id, 36)
}

pub fn make_proposal_address(record_id: &str, agent_id: &str) -> String {
    String::from(TRACK_AND_TRACE_NAMESPACE) + PROPOSAL + &hash(record_id, 36) + &hash(agent_id, 26)
}

pub fn num_to_page_number(page: u32) -> String {
    format!("{:01$x}", page, 4)
}
