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
use crate::roles::store::diesel::{schema::role, RoleStoreError};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::{ConstraintViolationError, ConstraintViolationType, InternalError};
use crate::roles::store::diesel::models::{NewRoleModel, RoleModel};
use diesel::{
    dsl::insert_into,
    prelude::*,
    result::{DatabaseErrorKind, Error as dsl_error},
};

pub(in crate::roles::store::diesel) trait RoleStoreAddRolesOperation {
    fn add_roles(&self, roles: Vec<NewRoleModel>) -> Result<(), RoleStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> RoleStoreAddRolesOperation for RoleStoreOperations<'a, diesel::pg::PgConnection> {
    fn add_roles(&self, roles: Vec<NewRoleModel>) -> Result<(), RoleStoreError> {
        for r in roles {
            let duplicate_role = role::table
                .filter(
                    role::name
                        .eq(&r.name)
                        .and(role::org_id.eq(&r.org_id))
                        .and(role::service_id.eq(&r.service_id))
                        .and(role::end_commit_num.eq(MAX_COMMIT_NUM)),
                )
                .first::<RoleModel>(self.conn)
                .map(Some)
                .or_else(|err| {
                    if err == dsl_error::NotFound {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                })
                .map_err(|err| {
                    RoleStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;
            if duplicate_role.is_some() {
                return Err(RoleStoreError::ConstraintViolationError(
                    ConstraintViolationError::with_violation_type(ConstraintViolationType::Unique),
                ));
            }

            insert_into(role::table)
                .values(&r)
                .execute(self.conn)
                .map(|_| ())
                .map_err(|err| match err {
                    dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                        RoleStoreError::ConstraintViolationError(
                            ConstraintViolationError::from_source_with_violation_type(
                                ConstraintViolationType::Unique,
                                Box::new(err),
                            ),
                        )
                    }
                    dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                        RoleStoreError::ConstraintViolationError(
                            ConstraintViolationError::from_source_with_violation_type(
                                ConstraintViolationType::ForeignKey,
                                Box::new(err),
                            ),
                        )
                    }
                    _ => RoleStoreError::InternalError(InternalError::from_source(Box::new(err))),
                })?;
        }

        Ok(())
    }
}

#[cfg(feature = "sqlite")]
impl<'a> RoleStoreAddRolesOperation for RoleStoreOperations<'a, diesel::sqlite::SqliteConnection> {
    fn add_roles(&self, roles: Vec<NewRoleModel>) -> Result<(), RoleStoreError> {
        for r in roles {
            let duplicate_role = role::table
                .filter(
                    role::name
                        .eq(&r.name)
                        .and(role::org_id.eq(&r.org_id))
                        .and(role::service_id.eq(&r.service_id))
                        .and(role::end_commit_num.eq(MAX_COMMIT_NUM)),
                )
                .first::<RoleModel>(self.conn)
                .map(Some)
                .or_else(|err| {
                    if err == dsl_error::NotFound {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                })
                .map_err(|err| {
                    RoleStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;
            if duplicate_role.is_some() {
                return Err(RoleStoreError::ConstraintViolationError(
                    ConstraintViolationError::with_violation_type(ConstraintViolationType::Unique),
                ));
            }

            insert_into(role::table)
                .values(&r)
                .execute(self.conn)
                .map(|_| ())
                .map_err(|err| match err {
                    dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                        RoleStoreError::ConstraintViolationError(
                            ConstraintViolationError::from_source_with_violation_type(
                                ConstraintViolationType::Unique,
                                Box::new(err),
                            ),
                        )
                    }
                    dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                        RoleStoreError::ConstraintViolationError(
                            ConstraintViolationError::from_source_with_violation_type(
                                ConstraintViolationType::ForeignKey,
                                Box::new(err),
                            ),
                        )
                    }
                    _ => RoleStoreError::InternalError(InternalError::from_source(Box::new(err))),
                })?;
        }

        Ok(())
    }
}
