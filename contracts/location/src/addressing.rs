// Copyright 2020 Cargill Incorporated
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

pub const GRID_NAMESPACE: &str = "621dee";

pub fn compute_gs1_location_address(gln: &str) -> String {
    //621ddee (grid namespace) + 04 (location namesapce) + 01 (gs1 namespace)
    String::from(GRID_NAMESPACE) + "0401000000000000000000000000000000000000000000000" + gln + "00"
}

/// Computes the address a Pike Organization is stored at based on its identifier
pub fn compute_org_address(identifier: &str) -> String {
    let mut sha = Sha512::new();
    sha.input(identifier.as_bytes());

    // 621dee05 (pike namespace) + 01 (organization namespace)
    String::from("621dee0501") + &sha.result_str()[..60]
}

pub fn compute_schema_address(name: &str) -> String {
    let mut sha = Sha512::new();
    sha.input(name.as_bytes());

    String::from(GRID_NAMESPACE) + "01" + &sha.result_str()[..62].to_string()
}
