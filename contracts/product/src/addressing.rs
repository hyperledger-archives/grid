// Copyright (c) 2019 Target Brands, Inc.
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

const GRID_ADDRESS_LEN: usize = 70;
const GS1_NAMESPACE: &str = "01"; // Indicates GS1 standard
const PRODUCT_NAMESPACE: &str = "02"; // Indicates product under GS1 standard
const GRID_NAMESPACE: &str = "621dee"; // Grid prefix
pub const PIKE_NAMESPACE: &str = "cad11d";
pub const PIKE_AGENT_NAMESPACE: &str = "00";
pub const PIKE_ORG_NAMESPACE: &str = "01";

pub fn get_product_prefix() -> String {
    GRID_NAMESPACE.to_string()
}

pub fn hash(to_hash: &str, num: usize) -> String {
    let mut sha = Sha512::new();
    sha.input_str(to_hash);
    let temp = sha.result_str();
    let hash = temp.get(..num).expect("PANIC! Hashing Out of Bounds Error");
    hash.to_string()
}

pub fn make_product_address(product_id: &str) -> String {
    let grid_product_gs1_prefix = get_product_prefix() + PRODUCT_NAMESPACE + GS1_NAMESPACE;
    let grid_product_gs1_prefix_len = grid_product_gs1_prefix.chars().count();
    let hash_len = GRID_ADDRESS_LEN - grid_product_gs1_prefix_len;

    grid_product_gs1_prefix + &hash(product_id, hash_len)
}

/// Computes the address a Pike Agent is stored at based on its public_key
pub fn compute_agent_address(public_key: &str) -> String {
    let mut sha = Sha512::new();
    sha.input(public_key.as_bytes());

    String::from(PIKE_NAMESPACE) + PIKE_AGENT_NAMESPACE + &sha.result_str()[..62]
}

/// Computes the address a Pike Organization is stored at based on its identifier
pub fn compute_org_address(identifier: &str) -> String {
    let mut sha = Sha512::new();
    sha.input(identifier.as_bytes());

    String::from(PIKE_NAMESPACE) + PIKE_ORG_NAMESPACE + &sha.result_str()[..62]
}
