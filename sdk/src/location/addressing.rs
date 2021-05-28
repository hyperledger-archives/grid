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
pub const LOCATION_PREFIX: &str = "04";
pub const GRID_LOCATION_NAMESPACE: &str = "621dee04";

pub fn compute_gs1_location_address(gln: &str) -> String {
    //621ddee (grid namespace) + 04 (location namespace) + 01 (gs1 namespace)
    String::from(GRID_NAMESPACE)
        + LOCATION_PREFIX
        + "01000000000000000000000000000000000000000000000"
        + gln
        + "00"
}
