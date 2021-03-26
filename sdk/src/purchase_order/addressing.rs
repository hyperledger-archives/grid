// Copyright 2021 Cargill Incorporated
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

//! Provides constants and functions for computing Merkle addresses.

use crypto::digest::Digest;
use crypto::sha2::Sha512;

pub const GRID_PURCHASE_ORDER_NAMESPACE: &str = "621dee06";

pub const PO_PREFIX: &str = "00";
pub const ALTERNATE_ID_PREFIX: &str = "01";

pub const GRID_PURCHASE_ORDER_PO_NAMESPACE: &str = "621dee0600";
pub const GRID_PURCHASE_ORDER_ALT_ID_NAMESPACE: &str = "621dee0601";

/// Computes the Merkle address of a Purchase Order based on its UUID.
pub fn compute_purchase_order_address(uuid: &str) -> String {
    let mut sha = Sha512::new();
    sha.input(uuid.as_bytes());
    let hash_str = String::from(GRID_PURCHASE_ORDER_NAMESPACE) + PO_PREFIX + &sha.result_str();
    hash_str[..70].to_string()
}

/// Computes the Merkle address of a Alternate ID based on its type and id.
pub fn compute_alternate_id_address(
    org_id: &str,
    alternate_id_type: &str,
    alternate_id: &str,
) -> String {
    let mut sha = Sha512::new();
    sha.input(org_id.as_bytes());
    sha.input(b":");
    sha.input(alternate_id_type.as_bytes());
    sha.input(b":");
    sha.input(alternate_id.as_bytes());
    let hash_str =
        String::from(GRID_PURCHASE_ORDER_NAMESPACE) + ALTERNATE_ID_PREFIX + &sha.result_str();
    hash_str[..70].to_string()
}
