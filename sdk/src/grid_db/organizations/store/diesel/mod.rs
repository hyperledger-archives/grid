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
pub(in crate::grid_db) mod schema;

use diesel::r2d2::{ConnectionManager, Pool};
use std::iter::FromIterator;

use super::diesel::models::{NewOrganizationModel, OrganizationModel};
use super::{Organization, OrganizationStore, OrganizationStoreError};
use crate::database::DatabaseError;
use operations::add_organizations::OrganizationStoreAddOrganizationsOperation as _;
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
    fn add_organizations(&self, orgs: Vec<Organization>) -> Result<(), OrganizationStoreError> {
        OrganizationStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .add_organizations(Vec::from_iter(orgs.iter().map(|org| org.clone().into())))
    }

    fn list_organizations(
        &self,
        service_id: Option<String>,
    ) -> Result<Vec<Organization>, OrganizationStoreError> {
        OrganizationStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .list_organizations(service_id)
    }

    fn fetch_organization(
        &self,
        org_id: &str,
        service_id: Option<String>,
    ) -> Result<Option<Organization>, OrganizationStoreError> {
        OrganizationStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .fetch_organization(org_id, service_id)
    }
}

#[cfg(feature = "sqlite")]
impl OrganizationStore for DieselOrganizationStore<diesel::sqlite::SqliteConnection> {
    fn add_organizations(&self, orgs: Vec<Organization>) -> Result<(), OrganizationStoreError> {
        OrganizationStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .add_organizations(Vec::from_iter(orgs.iter().map(|org| org.clone().into())))
    }

    fn list_organizations(
        &self,
        service_id: Option<String>,
    ) -> Result<Vec<Organization>, OrganizationStoreError> {
        OrganizationStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .list_organizations(service_id)
    }

    fn fetch_organization(
        &self,
        org_id: &str,
        service_id: Option<String>,
    ) -> Result<Option<Organization>, OrganizationStoreError> {
        OrganizationStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            DatabaseError::ConnectionError {
                context: "Could not get connection pool".to_string(),
                source: Box::new(err),
            }
        })?)
        .fetch_organization(org_id, service_id)
    }
}

impl From<OrganizationModel> for Organization {
    fn from(org: OrganizationModel) -> Self {
        Self {
            org_id: org.org_id,
            name: org.name,
            address: org.address,
            metadata: org.metadata,
            start_commit_num: org.start_commit_num,
            end_commit_num: org.end_commit_num,
            service_id: org.service_id,
        }
    }
}

impl From<NewOrganizationModel> for Organization {
    fn from(org: NewOrganizationModel) -> Self {
        Self {
            org_id: org.org_id,
            name: org.name,
            address: org.address,
            metadata: org.metadata,
            start_commit_num: org.start_commit_num,
            end_commit_num: org.end_commit_num,
            service_id: org.service_id,
        }
    }
}

impl Into<NewOrganizationModel> for Organization {
    fn into(self) -> NewOrganizationModel {
        NewOrganizationModel {
            org_id: self.org_id,
            name: self.name,
            address: self.address,
            metadata: self.metadata,
            start_commit_num: self.start_commit_num,
            end_commit_num: self.end_commit_num,
            service_id: self.service_id,
        }
    }
}

impl From<DatabaseError> for OrganizationStoreError {
    fn from(err: DatabaseError) -> OrganizationStoreError {
        OrganizationStoreError::ConnectionError(Box::new(err))
    }
}

impl From<diesel::result::Error> for OrganizationStoreError {
    fn from(err: diesel::result::Error) -> OrganizationStoreError {
        OrganizationStoreError::QueryError {
            context: "Diesel query failed".to_string(),
            source: Box::new(err),
        }
    }
}

impl From<diesel::r2d2::PoolError> for OrganizationStoreError {
    fn from(err: diesel::r2d2::PoolError) -> OrganizationStoreError {
        OrganizationStoreError::ConnectionError(Box::new(err))
    }
}
