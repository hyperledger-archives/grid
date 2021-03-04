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

use crate::{pike::store::Role, rest_api::resources::paging::v1::Paging};

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleSlice {
    pub org_id: String,
    pub name: String,
    pub description: String,
    pub active: bool,
    pub permissions: Vec<String>,
    pub allowed_organizations: Vec<String>,
    pub inherit_from: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

impl From<Role> for RoleSlice {
    fn from(role: Role) -> Self {
        let permissions = role.permissions.into_iter().map(String::from).collect();

        let allowed_organizations = role
            .allowed_organizations
            .into_iter()
            .map(String::from)
            .collect();

        let inherit_from = role.inherit_from.into_iter().map(String::from).collect();

        Self {
            org_id: role.org_id.clone(),
            name: role.name.clone(),
            description: role.description.clone(),
            active: role.active,
            permissions,
            allowed_organizations,
            inherit_from,
            service_id: role.service_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleListSlice {
    pub data: Vec<RoleSlice>,
    pub paging: Paging,
}
