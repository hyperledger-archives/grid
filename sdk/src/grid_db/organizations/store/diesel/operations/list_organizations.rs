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

use super::OrganizationStoreOperations;
use crate::grid_db::error::StoreError;
use crate::grid_db::organizations::store::diesel::models::OrganizationModel;
use crate::grid_db::organizations::store::diesel::schema::organization;
use crate::grid_db::organizations::store::Organization;
use diesel::prelude::*;

pub(in crate::grid_db::organizations) trait OrganizationStoreListOrganizationsOperation {
    fn list_organizations(
        &self,
        service_id: Option<String>,
    ) -> Result<Vec<Organization>, StoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> OrganizationStoreListOrganizationsOperation
    for OrganizationStoreOperations<'a, diesel::pg::PgConnection>
{
    fn list_organizations(
        &self,
        service_id: Option<String>,
    ) -> Result<Vec<Organization>, StoreError> {
        let orgs = organization::table
            .select(organization::all_columns)
            .filter(organization::service_id.eq(service_id))
            .load::<OrganizationModel>(self.conn)
            .map(Some)
            .map_err(|err| StoreError::OperationError {
                context: "Failed to fetch organizations".to_string(),
                source: Some(Box::new(err)),
            })?
            .ok_or_else(|| {
                StoreError::NotFoundError(
                    "Could not get all organizations from storage".to_string(),
                )
            })?
            .into_iter()
            .map(Organization::from)
            .collect();
        Ok(orgs)
    }
}

#[cfg(feature = "sqlite")]
impl<'a> OrganizationStoreListOrganizationsOperation
    for OrganizationStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn list_organizations(
        &self,
        service_id: Option<String>,
    ) -> Result<Vec<Organization>, StoreError> {
        let orgs = organization::table
            .select(organization::all_columns)
            .filter(organization::service_id.eq(service_id))
            .load::<OrganizationModel>(self.conn)
            .map(Some)
            .map_err(|err| StoreError::OperationError {
                context: "Failed to fetch organizations".to_string(),
                source: Some(Box::new(err)),
            })?
            .ok_or_else(|| {
                StoreError::NotFoundError(
                    "Could not get all organizations from storage".to_string(),
                )
            })?
            .into_iter()
            .map(Organization::from)
            .collect();
        Ok(orgs)
    }
}
