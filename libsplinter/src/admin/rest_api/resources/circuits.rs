// Copyright 2018-2020 Cargill Incorporated
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

use crate::circuit::{AuthorizationType, DurabilityType, PersistenceType, Roster, RouteType};
use crate::rest_api::paging::Paging;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CircuitResponse {
    pub id: String,
    pub auth: AuthorizationType,
    pub members: Vec<String>,
    pub roster: Roster,
    pub persistence: PersistenceType,
    pub durability: DurabilityType,
    pub routes: RouteType,
    pub circuit_management_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ListCircuitsResponse {
    pub data: Vec<CircuitResponse>,
    pub paging: Paging,
}
