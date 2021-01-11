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

#[cfg(feature = "diesel")]
pub mod diesel;
mod error;

pub use error::RoleStoreError;

use crate::hex::as_hex;

/// Represents a Grid Role
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Role {
    pub org_id: String,
    pub name: String,
    pub description: String,
    #[serde(serialize_with = "as_hex")]
    #[serde(deserialize_with = "deserialize_hex")]
    #[serde(default)]
    pub permissions: Vec<u8>,
    #[serde(serialize_with = "as_hex")]
    #[serde(deserialize_with = "deserialize_hex")]
    #[serde(default)]
    pub allowed_orgs: Vec<u8>,
    #[serde(serialize_with = "as_hex")]
    #[serde(deserialize_with = "deserialize_hex")]
    #[serde(default)]
    pub inherit_from: Vec<u8>,
    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

pub trait RoleStore: Send + Sync {
    /// Adds a role to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `roles` - The roles to be added
    fn add_roles(&self, roles: Vec<Role>) -> Result<(), RoleStoreError>;

    ///  Lists roles from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `org_id` - The organization id to list roles for
    ///  * `service_id` - The service id to list roles for
    fn list_roles_for_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<Role>, RoleStoreError>;

    /// Fetches a role from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `name` - The role to fetch
    ///  * `service_id` - The service id of the role to fetch
    fn fetch_role(
        &self,
        name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Role>, RoleStoreError>;

    /// Updates a role from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `role` - The role to update
    fn update_role(&self, role: Role) -> Result<(), RoleStoreError>;
}

impl<OS> RoleStore for Box<OS>
where
    OS: RoleStore + ?Sized,
{
    fn add_roles(&self, roles: Vec<Role>) -> Result<(), RoleStoreError> {
        (**self).add_roles(roles)
    }

    fn list_roles_for_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<Role>, RoleStoreError> {
        (**self).list_roles_for_organization(org_id, service_id)
    }

    fn fetch_role(
        &self,
        name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Role>, RoleStoreError> {
        (**self).fetch_role(name, service_id)
    }

    fn update_role(&self, role: Role) -> Result<(), RoleStoreError> {
        (**self).update_role(role)
    }
}
