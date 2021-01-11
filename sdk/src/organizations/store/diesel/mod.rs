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

pub mod models;
mod operations;
pub(in crate) mod schema;

use diesel::r2d2::{ConnectionManager, Pool};
use std::iter::FromIterator;

use super::diesel::models::{
    AltIDModel, LocationModel, NewAltIDModel, NewLocationModel, NewOrganizationModel,
    OrganizationModel,
};
use super::{AltID, Location, Organization, OrganizationStore, OrganizationStoreError};
use crate::error::{
    ConstraintViolationError, ConstraintViolationType, InternalError,
    ResourceTemporarilyUnavailableError,
};
use operations::add_organization::OrganizationStoreAddOrganizationOperation as _;
use operations::fetch_organization::OrganizationStoreFetchOrganizationOperation as _;
use operations::list_organizations::OrganizationStoreListOrganizationsOperation as _;
use operations::OrganizationStoreOperations;

/// Manages creating organizations in the database
#[derive(Clone)]
pub struct DieselOrganizationStore<C: diesel::Connection + 'static> {
    connection_pool: Pool<ConnectionManager<C>>,
}

impl<C: diesel::Connection> DieselOrganizationStore<C> {
    /// Creates a new DieselOrganizationStore
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: connection pool to the database
    // Allow dead code if diesel feature is not enabled
    #[allow(dead_code)]
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> Self {
        DieselOrganizationStore { connection_pool }
    }
}

#[cfg(feature = "postgres")]
impl OrganizationStore for DieselOrganizationStore<diesel::pg::PgConnection> {
    fn add_organization(&self, org: Organization) -> Result<(), OrganizationStoreError> {
        OrganizationStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            OrganizationStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_organization(
            NewOrganizationModel::from(&org),
            Vec::from_iter(org.locations.iter().map(|loc| NewLocationModel::from(loc))),
            Vec::from_iter(org.alternate_ids.iter().map(|id| NewAltIDModel::from(id))),
        )
    }

    fn list_organizations(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<Organization>, OrganizationStoreError> {
        OrganizationStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            OrganizationStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_organizations(service_id)
    }

    fn fetch_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Organization>, OrganizationStoreError> {
        OrganizationStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            OrganizationStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .fetch_organization(org_id, service_id)
    }
}

#[cfg(feature = "sqlite")]
impl OrganizationStore for DieselOrganizationStore<diesel::sqlite::SqliteConnection> {
    fn add_organization(&self, org: Organization) -> Result<(), OrganizationStoreError> {
        OrganizationStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            OrganizationStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_organization(
            NewOrganizationModel::from(&org),
            Vec::from_iter(org.locations.iter().map(|loc| NewLocationModel::from(loc))),
            Vec::from_iter(org.alternate_ids.iter().map(|id| NewAltIDModel::from(id))),
        )
    }

    fn list_organizations(
        &self,
        service_id: Option<&str>,
    ) -> Result<Vec<Organization>, OrganizationStoreError> {
        OrganizationStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            OrganizationStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_organizations(service_id)
    }

    fn fetch_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Organization>, OrganizationStoreError> {
        OrganizationStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            OrganizationStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .fetch_organization(org_id, service_id)
    }
}

impl From<(OrganizationModel, Vec<LocationModel>, Vec<AltIDModel>)> for Organization {
    fn from(
        (org, locations, alternate_ids): (OrganizationModel, Vec<LocationModel>, Vec<AltIDModel>),
    ) -> Self {
        Self {
            org_id: org.org_id,
            name: org.name,
            locations: locations.iter().map(|l| Location::from(l)).collect(),
            alternate_ids: alternate_ids.iter().map(|a| AltID::from(a)).collect(),
            metadata: org.metadata,
            start_commit_num: org.start_commit_num,
            end_commit_num: org.end_commit_num,
            service_id: org.service_id,
        }
    }
}

impl From<(NewOrganizationModel, Vec<LocationModel>, Vec<AltIDModel>)> for Organization {
    fn from(
        (org, locations, alternate_ids): (
            NewOrganizationModel,
            Vec<LocationModel>,
            Vec<AltIDModel>,
        ),
    ) -> Self {
        Self {
            org_id: org.org_id,
            name: org.name,
            locations: locations.iter().map(|l| Location::from(l)).collect(),
            alternate_ids: alternate_ids.iter().map(|a| AltID::from(a)).collect(),
            metadata: org.metadata,
            start_commit_num: org.start_commit_num,
            end_commit_num: org.end_commit_num,
            service_id: org.service_id,
        }
    }
}

impl From<&Organization> for NewOrganizationModel {
    fn from(org: &Organization) -> NewOrganizationModel {
        NewOrganizationModel {
            org_id: org.org_id.clone(),
            name: org.name.clone(),
            metadata: org.metadata.clone(),
            start_commit_num: org.start_commit_num.clone(),
            end_commit_num: org.end_commit_num.clone(),
            service_id: org.service_id.clone(),
        }
    }
}

impl From<&LocationModel> for Location {
    fn from(location: &LocationModel) -> Location {
        Location {
            location: location.location.clone(),
            org_id: location.org_id.clone(),
            start_commit_num: location.start_commit_num.clone(),
            end_commit_num: location.end_commit_num.clone(),
            service_id: location.service_id.clone(),
        }
    }
}

impl From<&Location> for NewLocationModel {
    fn from(location: &Location) -> NewLocationModel {
        NewLocationModel {
            location: location.location.clone(),
            org_id: location.org_id.clone(),
            start_commit_num: location.start_commit_num.clone(),
            end_commit_num: location.end_commit_num.clone(),
            service_id: location.service_id.clone(),
        }
    }
}

impl From<&AltIDModel> for AltID {
    fn from(id: &AltIDModel) -> AltID {
        AltID {
            alternate_id: id.alternate_id.clone(),
            id_type: id.id_type.clone(),
            org_id: id.org_id.clone(),
            start_commit_num: id.start_commit_num.clone(),
            end_commit_num: id.end_commit_num.clone(),
            service_id: id.service_id.clone(),
        }
    }
}

impl From<&AltID> for NewAltIDModel {
    fn from(id: &AltID) -> NewAltIDModel {
        NewAltIDModel {
            alternate_id: id.alternate_id.clone(),
            id_type: id.id_type.clone(),
            org_id: id.org_id.clone(),
            start_commit_num: id.start_commit_num.clone(),
            end_commit_num: id.end_commit_num.clone(),
            service_id: id.service_id.clone(),
        }
    }
}

impl From<diesel::result::Error> for OrganizationStoreError {
    fn from(err: diesel::result::Error) -> OrganizationStoreError {
        match err {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => OrganizationStoreError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::Unique,
                    Box::new(err),
                ),
            ),
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                _,
            ) => OrganizationStoreError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::ForeignKey,
                    Box::new(err),
                ),
            ),
            _ => OrganizationStoreError::InternalError(InternalError::from_source(Box::new(err))),
        }
    }
}

impl From<diesel::r2d2::PoolError> for OrganizationStoreError {
    fn from(err: diesel::r2d2::PoolError) -> OrganizationStoreError {
        OrganizationStoreError::ResourceTemporarilyUnavailableError(
            ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
        )
    }
}
