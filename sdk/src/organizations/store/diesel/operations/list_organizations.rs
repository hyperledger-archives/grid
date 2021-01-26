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
use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::organizations::store::diesel::models::{AltIDModel, OrganizationModel};
use crate::organizations::store::diesel::{
    schema::{alternate_identifier, organization},
    OrganizationStoreError,
};
use crate::organizations::store::Organization;

use diesel::prelude::*;

pub(in crate::organizations::store::diesel) trait OrganizationStoreListOrganizationsOperation {
    fn list_organizations(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<Organization>, OrganizationStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> OrganizationStoreListOrganizationsOperation
    for OrganizationStoreOperations<'a, diesel::pg::PgConnection>
{
    fn list_organizations(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<Organization>, OrganizationStoreError> {
        self.conn
            .build_transaction()
            .read_write()
            .run::<_, OrganizationStoreError, _>(|| {
                let mut query = organization::table
                    .into_boxed()
                    .select(organization::all_columns)
                    .filter(organization::end_commit_num.eq(MAX_COMMIT_NUM));

                if let Some(service_id) = service_id {
                    query = query.filter(organization::service_id.eq(service_id));
                } else {
                    query = query.filter(organization::service_id.is_null());
                }

                let org_models = query.load::<OrganizationModel>(self.conn).map_err(|err| {
                    OrganizationStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

                let mut orgs = Vec::new();

                for org in org_models {
                    let mut ids_query = alternate_identifier::table
                        .into_boxed()
                        .select(alternate_identifier::all_columns)
                        .filter(
                            alternate_identifier::org_id
                                .eq(&org.org_id)
                                .and(alternate_identifier::end_commit_num.eq(MAX_COMMIT_NUM)),
                        );

                    if let Some(service_id) = service_id {
                        ids_query =
                            ids_query.filter(alternate_identifier::service_id.eq(service_id));
                    } else {
                        ids_query = ids_query.filter(alternate_identifier::service_id.is_null());
                    }

                    let ids = ids_query.load::<AltIDModel>(self.conn).map_err(|err| {
                        OrganizationStoreError::InternalError(InternalError::from_source(Box::new(
                            err,
                        )))
                    })?;

                    orgs.push(Organization::from((org, ids)));
                }

                Ok(orgs)
            })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> OrganizationStoreListOrganizationsOperation
    for OrganizationStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn list_organizations(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<Organization>, OrganizationStoreError> {
        self.conn
            .immediate_transaction::<_, OrganizationStoreError, _>(|| {
                let mut query = organization::table
                    .into_boxed()
                    .select(organization::all_columns)
                    .filter(organization::end_commit_num.eq(MAX_COMMIT_NUM));

                if let Some(service_id) = service_id {
                    query = query.filter(organization::service_id.eq(service_id));
                } else {
                    query = query.filter(organization::service_id.is_null());
                }

                let org_models = query.load::<OrganizationModel>(self.conn).map_err(|err| {
                    OrganizationStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

                let mut orgs = Vec::new();

                for org in org_models {
                    let mut ids_query = alternate_identifier::table
                        .into_boxed()
                        .select(alternate_identifier::all_columns)
                        .filter(
                            alternate_identifier::org_id
                                .eq(&org.org_id)
                                .and(alternate_identifier::end_commit_num.eq(MAX_COMMIT_NUM)),
                        );

                    if let Some(service_id) = service_id {
                        ids_query =
                            ids_query.filter(alternate_identifier::service_id.eq(service_id));
                    } else {
                        ids_query = ids_query.filter(alternate_identifier::service_id.is_null());
                    }

                    let ids = ids_query.load::<AltIDModel>(self.conn).map_err(|err| {
                        OrganizationStoreError::InternalError(InternalError::from_source(Box::new(
                            err,
                        )))
                    })?;

                    orgs.push(Organization::from((org, ids)));
                }

                Ok(orgs)
            })
    }
}
