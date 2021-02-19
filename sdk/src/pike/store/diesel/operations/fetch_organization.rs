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

use super::PikeStoreOperations;
use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::pike::store::diesel::models::OrganizationModel;
use crate::pike::store::diesel::{schema::organization, PikeStoreError};
use crate::pike::store::Organization;
use diesel::{prelude::*, result::Error::NotFound};

pub(in crate::pike::store::diesel) trait PikeStoreFetchOrganizationOperation {
    fn fetch_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Organization>, PikeStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PikeStoreFetchOrganizationOperation for PikeStoreOperations<'a, diesel::pg::PgConnection> {
    fn fetch_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Organization>, PikeStoreError> {
        let mut query = organization::table
            .into_boxed()
            .select(organization::all_columns)
            .filter(
                organization::org_id
                    .eq(&org_id)
                    .and(organization::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(organization::service_id.eq(service_id));
        } else {
            query = query.filter(organization::service_id.is_null());
        }

        query
            .first::<OrganizationModel>(self.conn)
            .map(Organization::from)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| PikeStoreError::InternalError(InternalError::from_source(Box::new(err))))
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PikeStoreFetchOrganizationOperation
    for PikeStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn fetch_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Organization>, PikeStoreError> {
        let mut query = organization::table
            .into_boxed()
            .select(organization::all_columns)
            .filter(
                organization::org_id
                    .eq(&org_id)
                    .and(organization::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(organization::service_id.eq(service_id));
        } else {
            query = query.filter(organization::service_id.is_null());
        }

        query
            .first::<OrganizationModel>(self.conn)
            .map(Organization::from)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| PikeStoreError::InternalError(InternalError::from_source(Box::new(err))))
    }
}
