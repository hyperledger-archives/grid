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

use super::RoleStoreOperations;
use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::roles::store::diesel::models::RoleModel;
use crate::roles::store::diesel::{schema::role, RoleStoreError};
use crate::roles::store::Role;
use diesel::{prelude::*, result::Error::NotFound};

pub(in crate::roles::store::diesel) trait RoleStoreFetchRoleOperation {
    fn fetch_role(
        &self,
        name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Role>, RoleStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> RoleStoreFetchRoleOperation for RoleStoreOperations<'a, diesel::pg::PgConnection> {
    fn fetch_role(
        &self,
        name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Role>, RoleStoreError> {
        self.conn
            .build_transaction()
            .read_write()
            .run::<_, RoleStoreError, _>(|| {
                let mut query = role::table.into_boxed().select(role::all_columns).filter(
                    role::name
                        .eq(&name)
                        .and(role::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

                if let Some(service_id) = service_id {
                    query = query.filter(role::service_id.eq(service_id));
                } else {
                    query = query.filter(role::service_id.is_null());
                }

                let role = query
                    .first::<RoleModel>(self.conn)
                    .map(Some)
                    .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                    .map_err(|err| {
                        RoleStoreError::InternalError(InternalError::from_source(Box::new(err)))
                    })?;

                Ok(role.map(|role| Role::from(role)))
            })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> RoleStoreFetchRoleOperation for RoleStoreOperations<'a, diesel::sqlite::SqliteConnection> {
    fn fetch_role(
        &self,
        name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Role>, RoleStoreError> {
        self.conn.immediate_transaction::<_, RoleStoreError, _>(|| {
            let mut query = role::table.into_boxed().select(role::all_columns).filter(
                role::name
                    .eq(&name)
                    .and(role::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

            if let Some(service_id) = service_id {
                query = query.filter(role::service_id.eq(service_id));
            } else {
                query = query.filter(role::service_id.is_null());
            }

            let role = query
                .first::<RoleModel>(self.conn)
                .map(Some)
                .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                .map_err(|err| {
                    RoleStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            Ok(role.map(|role| Role::from(role)))
        })
    }
}
