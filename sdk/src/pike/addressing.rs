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

//! Provides addressing functionality for Pike.

use crypto::digest::Digest;
use crypto::sha2::Sha512;

/// Namespace for Pike objects, prefixes addresses
pub const GRID_PIKE_NAMESPACE: &str = "621dee05";

/// Address prefix representing Pike agents
pub const AGENT_PREFIX: &str = "00";
/// Namespace specific to Pike agents
pub const GRID_PIKE_AGENT_NAMESPACE: &str = "621dee0500";

/// Address prefix representing Pike organizations
pub const ORG_PREFIX: &str = "01";
/// Namespace specific to Pike organizations
pub const GRID_PIKE_ORGANIZATION_NAMESPACE: &str = "621dee0501";

/// Address prefix representing Pike roles
pub const ROLE_PREFIX: &str = "02";
/// Namespace specific to Pike roles
pub const GRID_PIKE_ROLE_NAMESPACE: &str = "621dee0502";

/// Address prefix representing Pike alternate IDs
pub const ALTERNATE_ID_INDEX_ENTRY_PREFIX: &str = "03";
/// Namespace specific to Pike alternate IDs
pub const GRID_PIKE_ALTERNATE_ID_INDEX_ENTRY_NAMESPACE: &str = "621dee0503";

/// Computes the address a Pike Agent is stored at based on its public_key
pub fn compute_agent_address(public_key: &str) -> String {
    let mut sha = Sha512::new();
    sha.input(public_key.as_bytes());
    // (pike namespace) + (agent namespace) + hash
    let hash_str =
        String::from(GRID_PIKE_NAMESPACE) + &String::from(AGENT_PREFIX) + &sha.result_str();
    hash_str[..70].to_string()
}

/// Computes the address a Pike Organizaton is stored at based on its org_id
pub fn compute_organization_address(org_id: &str) -> String {
    let mut sha = Sha512::new();
    sha.input(org_id.as_bytes());
    // (pike namespace) + (org namespace) + hash
    let hash_str = String::from(GRID_PIKE_NAMESPACE) + ORG_PREFIX + &sha.result_str();
    hash_str[..70].to_string()
}

/// Computes the address a Pike Role is stored at based on its name
pub fn compute_role_address(name: &str, org_id: &str) -> String {
    let uname = format!("{}.{}", org_id, name);
    let mut sha = Sha512::new();
    sha.input(uname.as_bytes());
    // (pike namespace) + (role namespace) + hash
    let hash_str = String::from(GRID_PIKE_NAMESPACE) + ROLE_PREFIX + &sha.result_str();
    hash_str[..70].to_string()
}

/// Computes the address a Pike Alternate ID Index is stored at based on its name
pub fn compute_alternate_id_index_entry_address(id_type: &str, id: &str) -> String {
    let uname = format!("{}:{}", id_type, id);
    let mut sha = Sha512::new();
    sha.input(uname.as_bytes());
    // (pike namespace) + (alternate ID namespace) + hash
    let hash_str =
        String::from(GRID_PIKE_NAMESPACE) + ALTERNATE_ID_INDEX_ENTRY_PREFIX + &sha.result_str();
    hash_str[..70].to_string()
}
