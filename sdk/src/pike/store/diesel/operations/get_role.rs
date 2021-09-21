// Copyright 2018-2021 Cargill Incorporated
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

//! Provides the "get role" operation for the `DieselPikeStore`.

use super::PikeStoreOperations;
use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::pike::store::diesel::models::{
    AllowedOrgModel, InheritFromModel, PermissionModel, RoleModel,
};
use crate::pike::store::diesel::{
    schema::{pike_allowed_orgs, pike_inherit_from, pike_permissions, pike_role},
    PikeStoreError,
};
use crate::pike::store::Role;
use diesel::{prelude::*, result::Error::NotFound};

pub(in crate::pike::store::diesel) trait PikeStoreGetRoleOperation {
    fn get_role(
        &self,
        name: &str,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Role>, PikeStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PikeStoreGetRoleOperation for PikeStoreOperations<'a, diesel::pg::PgConnection> {
    fn get_role(
        &self,
        name: &str,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Role>, PikeStoreError> {
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let mut query = pike_role::table
                .into_boxed()
                .select(pike_role::all_columns)
                .filter(
                    pike_role::name
                        .eq(&name)
                        .and(pike_role::org_id.eq(&org_id))
                        .and(pike_role::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(pike_role::service_id.eq(service_id));
            } else {
                query = query.filter(pike_role::service_id.is_null());
            }

            let role = query
                .first::<RoleModel>(self.conn)
                .map(Some)
                .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                .map_err(|err| {
                    PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            let mut query = pike_inherit_from::table
                .into_boxed()
                .select(pike_inherit_from::all_columns)
                .filter(
                    pike_inherit_from::role_name
                        .eq(&name)
                        .and(pike_inherit_from::org_id.eq(&org_id))
                        .and(pike_inherit_from::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(pike_inherit_from::service_id.eq(service_id));
            } else {
                query = query.filter(pike_inherit_from::service_id.is_null());
            }

            let inherit_from = query.load::<InheritFromModel>(self.conn).map_err(|err| {
                PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            let mut query = pike_permissions::table
                .into_boxed()
                .select(pike_permissions::all_columns)
                .filter(
                    pike_permissions::role_name
                        .eq(&name)
                        .and(pike_permissions::org_id.eq(&org_id))
                        .and(pike_permissions::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(pike_permissions::service_id.eq(service_id));
            } else {
                query = query.filter(pike_permissions::service_id.is_null());
            }

            let permissions = query.load::<PermissionModel>(self.conn).map_err(|err| {
                PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            let mut query = pike_allowed_orgs::table
                .into_boxed()
                .select(pike_allowed_orgs::all_columns)
                .filter(
                    pike_allowed_orgs::role_name
                        .eq(&name)
                        .and(pike_allowed_orgs::org_id.eq(&org_id))
                        .and(pike_allowed_orgs::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(pike_allowed_orgs::service_id.eq(service_id));
            } else {
                query = query.filter(pike_allowed_orgs::service_id.is_null());
            }

            let allowed_orgs = query.load::<AllowedOrgModel>(self.conn).map_err(|err| {
                PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            Ok(role.map(|r| Role::from((r, inherit_from, permissions, allowed_orgs))))
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PikeStoreGetRoleOperation for PikeStoreOperations<'a, diesel::sqlite::SqliteConnection> {
    fn get_role(
        &self,
        name: &str,
        org_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Role>, PikeStoreError> {
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let mut query = pike_role::table
                .into_boxed()
                .select(pike_role::all_columns)
                .filter(
                    pike_role::name
                        .eq(&name)
                        .and(pike_role::org_id.eq(&org_id))
                        .and(pike_role::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(pike_role::service_id.eq(service_id));
            } else {
                query = query.filter(pike_role::service_id.is_null());
            }

            let role = query
                .first::<RoleModel>(self.conn)
                .map(Some)
                .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                .map_err(|err| {
                    PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            let mut query = pike_inherit_from::table
                .into_boxed()
                .select(pike_inherit_from::all_columns)
                .filter(
                    pike_inherit_from::role_name
                        .eq(&name)
                        .and(pike_inherit_from::org_id.eq(&org_id))
                        .and(pike_inherit_from::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(pike_inherit_from::service_id.eq(service_id));
            } else {
                query = query.filter(pike_inherit_from::service_id.is_null());
            }

            let inherit_from = query.load::<InheritFromModel>(self.conn).map_err(|err| {
                PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            let mut query = pike_permissions::table
                .into_boxed()
                .select(pike_permissions::all_columns)
                .filter(
                    pike_permissions::role_name
                        .eq(&name)
                        .and(pike_permissions::org_id.eq(&org_id))
                        .and(pike_permissions::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(pike_permissions::service_id.eq(service_id));
            } else {
                query = query.filter(pike_permissions::service_id.is_null());
            }

            let permissions = query.load::<PermissionModel>(self.conn).map_err(|err| {
                PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            let mut query = pike_allowed_orgs::table
                .into_boxed()
                .select(pike_allowed_orgs::all_columns)
                .filter(
                    pike_allowed_orgs::role_name
                        .eq(&name)
                        .and(pike_allowed_orgs::org_id.eq(&org_id))
                        .and(pike_allowed_orgs::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(pike_allowed_orgs::service_id.eq(service_id));
            } else {
                query = query.filter(pike_allowed_orgs::service_id.is_null());
            }

            let allowed_orgs = query.load::<AllowedOrgModel>(self.conn).map_err(|err| {
                PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            Ok(role.map(|r| Role::from((r, inherit_from, permissions, allowed_orgs))))
        })
    }
}
