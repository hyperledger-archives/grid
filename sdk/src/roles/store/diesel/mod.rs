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

use super::diesel::models::{NewRoleModel, RoleModel};
use super::{Role, RoleStore, RoleStoreError};
use crate::error::{
    ConstraintViolationError, ConstraintViolationType, InternalError,
    ResourceTemporarilyUnavailableError,
};
use operations::add_roles::RoleStoreAddRolesOperation as _;
use operations::fetch_role::RoleStoreFetchRoleOperation as _;
use operations::list_roles_for_organization::RoleStoreListRolesForOrganizationOperation as _;
use operations::update_role::RoleStoreUpdateRoleOperation as _;
use operations::RoleStoreOperations;

/// Manages creating roles in the database
#[derive(Clone)]
pub struct DieselRoleStore<C: diesel::Connection + 'static> {
    connection_pool: Pool<ConnectionManager<C>>,
}

impl<C: diesel::Connection> DieselRoleStore<C> {
    /// Creates a new DieselRoleStore
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: connection pool to the database
    // Allow dead code if diesel feature is not enabled
    #[allow(dead_code)]
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> Self {
        DieselRoleStore { connection_pool }
    }
}

#[cfg(feature = "postgres")]
impl RoleStore for DieselRoleStore<diesel::pg::PgConnection> {
    fn add_roles(&self, roles: Vec<Role>) -> Result<(), RoleStoreError> {
        RoleStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            RoleStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_roles(Vec::from_iter(roles.iter().map(|role| role.clone().into())))
    }

    fn list_roles_for_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<Role>, RoleStoreError> {
        RoleStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            RoleStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_roles_for_organization(org_id, service_id)
    }

    fn fetch_role(
        &self,
        name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Role>, RoleStoreError> {
        RoleStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            RoleStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .fetch_role(name, service_id)
    }

    fn update_role(&self, role: Role) -> Result<(), RoleStoreError> {
        RoleStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            RoleStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .update_role(role.clone().into())
    }
}

#[cfg(feature = "sqlite")]
impl RoleStore for DieselRoleStore<diesel::sqlite::SqliteConnection> {
    fn add_roles(&self, roles: Vec<Role>) -> Result<(), RoleStoreError> {
        RoleStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            RoleStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_roles(Vec::from_iter(roles.iter().map(|role| role.clone().into())))
    }

    fn list_roles_for_organization(
        &self,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Vec<Role>, RoleStoreError> {
        RoleStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            RoleStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_roles_for_organization(org_id, service_id)
    }

    fn fetch_role(
        &self,
        name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Role>, RoleStoreError> {
        RoleStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            RoleStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .fetch_role(name, service_id)
    }

    fn update_role(&self, role: Role) -> Result<(), RoleStoreError> {
        RoleStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            RoleStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .update_role(role.clone().into())
    }
}

impl From<RoleModel> for Role {
    fn from(role: RoleModel) -> Self {
        Self {
            org_id: role.org_id,
            name: role.name,
            description: role.description,
            permissions: role.permissions,
            allowed_orgs: role.allowed_orgs,
            inherit_from: role.inherit_from,
            start_commit_num: role.start_commit_num,
            end_commit_num: role.end_commit_num,
            service_id: role.service_id,
        }
    }
}

impl From<&RoleModel> for Role {
    fn from(role: &RoleModel) -> Self {
        Self {
            org_id: role.org_id.clone(),
            name: role.name.clone(),
            description: role.description.clone(),
            permissions: role.permissions.clone(),
            allowed_orgs: role.allowed_orgs.clone(),
            inherit_from: role.inherit_from.clone(),
            start_commit_num: role.start_commit_num.clone(),
            end_commit_num: role.end_commit_num.clone(),
            service_id: role.service_id.clone(),
        }
    }
}

impl From<NewRoleModel> for Role {
    fn from(role: NewRoleModel) -> Self {
        Self {
            org_id: role.org_id,
            name: role.name,
            description: role.description,
            permissions: role.permissions,
            allowed_orgs: role.allowed_orgs,
            inherit_from: role.inherit_from,
            start_commit_num: role.start_commit_num,
            end_commit_num: role.end_commit_num,
            service_id: role.service_id,
        }
    }
}

impl Into<NewRoleModel> for Role {
    fn into(self) -> NewRoleModel {
        NewRoleModel {
            org_id: self.org_id,
            name: self.name,
            description: self.description,
            permissions: self.permissions,
            allowed_orgs: self.allowed_orgs,
            inherit_from: self.inherit_from,
            start_commit_num: self.start_commit_num,
            end_commit_num: self.end_commit_num,
            service_id: self.service_id,
        }
    }
}

impl From<diesel::result::Error> for RoleStoreError {
    fn from(err: diesel::result::Error) -> RoleStoreError {
        match err {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => RoleStoreError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::Unique,
                    Box::new(err),
                ),
            ),
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                _,
            ) => RoleStoreError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::ForeignKey,
                    Box::new(err),
                ),
            ),
            _ => RoleStoreError::InternalError(InternalError::from_source(Box::new(err))),
        }
    }
}

impl From<diesel::r2d2::PoolError> for RoleStoreError {
    fn from(err: diesel::r2d2::PoolError) -> RoleStoreError {
        RoleStoreError::ResourceTemporarilyUnavailableError(
            ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
        )
    }
}
