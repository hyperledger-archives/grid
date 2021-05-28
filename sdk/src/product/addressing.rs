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

pub const GRID_NAMESPACE: &str = "621dee";
pub const PRODUCT_PREFIX: &str = "02";
pub const GRID_PRODUCT_NAMESPACE: &str = "621dee02";

/// Computes the address of a GS1 product based on its GTIN
pub fn compute_gs1_product_address(gtin: &str) -> String {
    // 621ddee (grid namespace) + 02 (product namespace) + 01 (gs1 namespace)
    String::from(GRID_NAMESPACE)
        + PRODUCT_PREFIX
        + "01"
        + "00000000000000000000000000000000000000000000"
        + &format!("{:0>14}", gtin)
        + "00"
}
