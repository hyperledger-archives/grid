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

pub use error::OrganizationStoreError;

use crate::hex::as_hex;

/// Represents a Grid Organization
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Organization {
    pub org_id: String,
    pub name: String,
    pub locations: Vec<Location>,
    pub alternate_ids: Vec<AltID>,
    #[serde(serialize_with = "as_hex")]
    #[serde(deserialize_with = "deserialize_hex")]
    #[serde(default)]
    pub metadata: Vec<u8>,
    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Location {
    pub location: String,
    pub org_id: String,
    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct AltID {
    pub alternate_id: String,
    pub id_type: String,
    pub org_id: String,
    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

pub trait OrganizationStore: Send + Sync {
    /// Adds an organization to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `org` - The organization to be added
    fn add_organization(&self, org: Organization) -> Result<(), OrganizationStoreError>;

    ///  Lists organizations from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `service_id` - The service id to list organizations for
    fn list_organizations(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<Organization>, OrganizationStoreError>;

    /// Fetches an organization from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `org_id` - This organization id to fetch
    ///  * `service_id` - The service id of the organization to fetch
    fn fetch_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Organization>, OrganizationStoreError>;
}

impl<OS> OrganizationStore for Box<OS>
where
    OS: OrganizationStore + ?Sized,
{
    fn add_organization(&self, org: Organization) -> Result<(), OrganizationStoreError> {
        (**self).add_organization(org)
    }

    fn list_organizations(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<Organization>, OrganizationStoreError> {
        (**self).list_organizations(service_id)
    }

    fn fetch_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Organization>, OrganizationStoreError> {
        (**self).fetch_organization(org_id, service_id)
    }
}
