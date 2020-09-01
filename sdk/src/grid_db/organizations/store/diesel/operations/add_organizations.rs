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
use crate::grid_db::organizations::store::diesel::models::{
    NewOrganizationModel, OrganizationModel,
};
use crate::grid_db::organizations::store::diesel::schema::organization;
use diesel::{dsl::insert_into, prelude::*, result::Error::NotFound};

pub(in crate::grid_db::organizations) trait OrganizationStoreAddOrganizationsOperation {
    fn add_organizations(&self, orgs: Vec<NewOrganizationModel>) -> Result<(), StoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> OrganizationStoreAddOrganizationsOperation
    for OrganizationStoreOperations<'a, diesel::pg::PgConnection>
{
    fn add_organizations(&self, orgs: Vec<NewOrganizationModel>) -> Result<(), StoreError> {
        for org in orgs {
            let duplicate_org = organization::table
                .filter(organization::org_id.eq(&org.org_id))
                .first::<OrganizationModel>(self.conn)
                .map(Some)
                .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                .map_err(|err| StoreError::QueryError {
                    context: "Failed check for existing organization".to_string(),
                    source: Box::new(err),
                })?;
            if duplicate_org.is_some() {
                return Err(StoreError::DuplicateError {
                    context: "Organization already exists".to_string(),
                    source: None,
                });
            }

            insert_into(organization::table)
                .values(org)
                .execute(self.conn)
                .map(|_| ())
                .map_err(|err| StoreError::OperationError {
                    context: "Failed to add organization".to_string(),
                    source: Some(Box::new(err)),
                })?;
        }

        Ok(())
    }
}

#[cfg(feature = "sqlite")]
impl<'a> OrganizationStoreAddOrganizationsOperation
    for OrganizationStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn add_organizations(&self, orgs: Vec<NewOrganizationModel>) -> Result<(), StoreError> {
        for org in orgs {
            let duplicate_org = organization::table
                .filter(organization::org_id.eq(&org.org_id))
                .first::<OrganizationModel>(self.conn)
                .map(Some)
                .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                .map_err(|err| StoreError::QueryError {
                    context: "Failed check for existing organization".to_string(),
                    source: Box::new(err),
                })?;
            if duplicate_org.is_some() {
                return Err(StoreError::DuplicateError {
                    context: "Organization already exists".to_string(),
                    source: None,
                });
            }

            insert_into(organization::table)
                .values(org)
                .execute(self.conn)
                .map(|_| ())
                .map_err(|err| StoreError::OperationError {
                    context: "Failed to add organization".to_string(),
                    source: Some(Box::new(err)),
                })?;
        }

        Ok(())
    }
}
