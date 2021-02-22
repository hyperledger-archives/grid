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
use crate::paging::Paging;
use crate::pike::store::diesel::models::{OrganizationMetadataModel, OrganizationModel};
use crate::pike::store::diesel::{
    schema::{pike_organization, pike_organization_metadata},
    PikeStoreError,
};
use crate::pike::store::{Organization, OrganizationList, OrganizationMetadata};

use diesel::prelude::*;

pub(in crate::pike::store::diesel) trait PikeStoreListOrganizationsOperation {
    fn list_organizations(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<OrganizationList, PikeStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PikeStoreListOrganizationsOperation for PikeStoreOperations<'a, diesel::pg::PgConnection> {
    fn list_organizations(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<OrganizationList, PikeStoreError> {
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let mut query = pike_organization::table
                .into_boxed()
                .select(pike_organization::all_columns)
                .filter(pike_organization::end_commit_num.eq(MAX_COMMIT_NUM));

            if let Some(service_id) = service_id {
                query = query.filter(pike_organization::service_id.eq(service_id));
            } else {
                query = query.filter(pike_organization::service_id.is_null());
            }

            let org_models = query.load::<OrganizationModel>(self.conn).map_err(|err| {
                PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            let mut count_query = pike_organization::table
                .into_boxed()
                .select(pike_organization::all_columns);

            if let Some(service_id) = service_id {
                count_query = count_query.filter(pike_organization::service_id.eq(service_id));
            } else {
                count_query = count_query.filter(pike_organization::service_id.is_null());
            }

            let total = count_query.count().get_result(self.conn)?;

            let mut orgs = Vec::new();

            for org in org_models {
                let mut query = pike_organization_metadata::table
                    .into_boxed()
                    .select(pike_organization_metadata::all_columns)
                    .filter(
                        pike_organization_metadata::org_id
                            .eq(&org.org_id)
                            .and(pike_organization_metadata::end_commit_num.eq(MAX_COMMIT_NUM)),
                    );

                if let Some(service_id) = service_id {
                    query = query.filter(pike_organization_metadata::service_id.eq(service_id));
                } else {
                    query = query.filter(pike_organization_metadata::service_id.is_null());
                }

                let metadata_models =
                    query
                        .load::<OrganizationMetadataModel>(self.conn)
                        .map_err(|err| {
                            PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                        })?;

                let metadata = metadata_models
                    .iter()
                    .map(OrganizationMetadata::from)
                    .collect();

                orgs.push(Organization::from((org, metadata)));
            }

            Ok(OrganizationList::new(
                orgs,
                Paging::new(offset, limit, total),
            ))
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PikeStoreListOrganizationsOperation
    for PikeStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn list_organizations(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<OrganizationList, PikeStoreError> {
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let mut query = pike_organization::table
                .into_boxed()
                .select(pike_organization::all_columns)
                .filter(pike_organization::end_commit_num.eq(MAX_COMMIT_NUM));

            if let Some(service_id) = service_id {
                query = query.filter(pike_organization::service_id.eq(service_id));
            } else {
                query = query.filter(pike_organization::service_id.is_null());
            }

            let org_models = query.load::<OrganizationModel>(self.conn).map_err(|err| {
                PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            let mut count_query = pike_organization::table
                .into_boxed()
                .select(pike_organization::all_columns);

            if let Some(service_id) = service_id {
                count_query = count_query.filter(pike_organization::service_id.eq(service_id));
            } else {
                count_query = count_query.filter(pike_organization::service_id.is_null());
            }

            let total = count_query.count().get_result(self.conn)?;

            let mut orgs = Vec::new();

            for org in org_models {
                let mut query = pike_organization_metadata::table
                    .into_boxed()
                    .select(pike_organization_metadata::all_columns)
                    .filter(
                        pike_organization_metadata::org_id
                            .eq(&org.org_id)
                            .and(pike_organization_metadata::end_commit_num.eq(MAX_COMMIT_NUM)),
                    );

                if let Some(service_id) = service_id {
                    query = query.filter(pike_organization_metadata::service_id.eq(service_id));
                } else {
                    query = query.filter(pike_organization_metadata::service_id.is_null());
                }

                let metadata_models =
                    query
                        .load::<OrganizationMetadataModel>(self.conn)
                        .map_err(|err| {
                            PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                        })?;

                let metadata = metadata_models
                    .iter()
                    .map(OrganizationMetadata::from)
                    .collect();

                orgs.push(Organization::from((org, metadata)));
            }

            Ok(OrganizationList::new(
                orgs,
                Paging::new(offset, limit, total),
            ))
        })
    }
}
