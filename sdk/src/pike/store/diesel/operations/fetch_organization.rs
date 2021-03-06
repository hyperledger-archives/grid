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
use crate::pike::store::diesel::models::{
    AlternateIDModel, LocationAssociationModel, OrganizationMetadataModel, OrganizationModel,
};
use crate::pike::store::diesel::{
    schema::{
        pike_organization, pike_organization_alternate_id, pike_organization_location_assoc,
        pike_organization_metadata,
    },
    PikeStoreError,
};
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
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let mut query = pike_organization::table
                .into_boxed()
                .select(pike_organization::all_columns)
                .filter(
                    pike_organization::org_id
                        .eq(&org_id)
                        .and(pike_organization::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(pike_organization::service_id.eq(service_id));
            } else {
                query = query.filter(pike_organization::service_id.is_null());
            }

            let org_model = query
                .first::<OrganizationModel>(self.conn)
                .map(Some)
                .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                .map_err(|err| {
                    PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            let mut metadata_query = pike_organization_metadata::table
                .into_boxed()
                .select(pike_organization_metadata::all_columns)
                .filter(
                    pike_organization_metadata::org_id
                        .eq(&org_id)
                        .and(pike_organization_metadata::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                metadata_query =
                    metadata_query.filter(pike_organization_metadata::service_id.eq(service_id));
            } else {
                metadata_query =
                    metadata_query.filter(pike_organization_metadata::service_id.is_null());
            }

            let metadata_models = metadata_query
                .load::<OrganizationMetadataModel>(self.conn)
                .map_err(|err| {
                    PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            let mut query = pike_organization_alternate_id::table
                .into_boxed()
                .select(pike_organization_alternate_id::all_columns)
                .filter(
                    pike_organization_alternate_id::org_id
                        .eq(&org_id)
                        .and(pike_organization_alternate_id::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(pike_organization_alternate_id::service_id.eq(service_id));
            } else {
                query = query.filter(pike_organization_alternate_id::service_id.is_null());
            }

            let alternate_ids = query.load::<AlternateIDModel>(self.conn).map_err(|err| {
                PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            let mut query = pike_organization_location_assoc::table
                .into_boxed()
                .select(pike_organization_location_assoc::all_columns)
                .filter(
                    pike_organization_location_assoc::org_id
                        .eq(&org_id)
                        .and(pike_organization_location_assoc::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(pike_organization_location_assoc::service_id.eq(service_id));
            } else {
                query = query.filter(pike_organization_location_assoc::service_id.is_null());
            }

            let location_models =
                query
                    .load::<LocationAssociationModel>(self.conn)
                    .map_err(|err| {
                        PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                    })?;

            Ok(org_model.map(|org| {
                Organization::from((org, metadata_models, alternate_ids, location_models))
            }))
        })
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
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let mut query = pike_organization::table
                .into_boxed()
                .select(pike_organization::all_columns)
                .filter(
                    pike_organization::org_id
                        .eq(&org_id)
                        .and(pike_organization::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(pike_organization::service_id.eq(service_id));
            } else {
                query = query.filter(pike_organization::service_id.is_null());
            }

            let org_model = query
                .first::<OrganizationModel>(self.conn)
                .map(Some)
                .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                .map_err(|err| {
                    PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            let mut metadata_query = pike_organization_metadata::table
                .into_boxed()
                .select(pike_organization_metadata::all_columns)
                .filter(
                    pike_organization_metadata::org_id
                        .eq(&org_id)
                        .and(pike_organization_metadata::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                metadata_query =
                    metadata_query.filter(pike_organization_metadata::service_id.eq(service_id));
            } else {
                metadata_query =
                    metadata_query.filter(pike_organization_metadata::service_id.is_null());
            }

            let metadata_models = metadata_query
                .load::<OrganizationMetadataModel>(self.conn)
                .map_err(|err| {
                    PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            let mut query = pike_organization_alternate_id::table
                .into_boxed()
                .select(pike_organization_alternate_id::all_columns)
                .filter(
                    pike_organization_alternate_id::org_id
                        .eq(&org_id)
                        .and(pike_organization_alternate_id::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(pike_organization_alternate_id::service_id.eq(service_id));
            } else {
                query = query.filter(pike_organization_alternate_id::service_id.is_null());
            }

            let alternate_ids = query.load::<AlternateIDModel>(self.conn).map_err(|err| {
                PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            let mut query = pike_organization_location_assoc::table
                .into_boxed()
                .select(pike_organization_location_assoc::all_columns)
                .filter(
                    pike_organization_location_assoc::org_id
                        .eq(&org_id)
                        .and(pike_organization_location_assoc::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(pike_organization_location_assoc::service_id.eq(service_id));
            } else {
                query = query.filter(pike_organization_location_assoc::service_id.is_null());
            }

            let location_models =
                query
                    .load::<LocationAssociationModel>(self.conn)
                    .map_err(|err| {
                        PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                    })?;

            Ok(org_model.map(|org| {
                Organization::from((org, metadata_models, alternate_ids, location_models))
            }))
        })
    }
}
