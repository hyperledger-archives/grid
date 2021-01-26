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
    dsl::{insert_into, update},
    prelude::*,
    result::{DatabaseErrorKind, Error as dsl_error},
};

pub(in crate::roles::store::diesel) trait RoleStoreUpdateRoleOperation {
    fn update_role(&self, role: NewRoleModel) -> Result<(), RoleStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> RoleStoreUpdateRoleOperation for RoleStoreOperations<'a, diesel::pg::PgConnection> {
    fn update_role(&self, role: NewRoleModel) -> Result<(), RoleStoreError> {
        self.conn
            .build_transaction()
            .read_write()
            .run::<_, RoleStoreError, _>(|| {
                let r = role::table
                    .filter(
                        role::name
                            .eq(&role.name)
                            .and(role::service_id.eq(&role.service_id)),
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

                if r.is_some() {
                    update(role::table)
                        .filter(
                            role::name
                                .eq(&role.name)
                                .and(role::org_id.eq(&role.org_id))
                                .and(role::service_id.eq(&role.service_id))
                                .and(role::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(role::end_commit_num.eq(&role.start_commit_num))
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
                            _ => RoleStoreError::InternalError(InternalError::from_source(
                                Box::new(err),
                            )),
                        })?;
                }

                insert_into(role::table)
                    .values(&role)
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
                        _ => {
                            RoleStoreError::InternalError(InternalError::from_source(Box::new(err)))
                        }
                    })?;

                Ok(())
            })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> RoleStoreUpdateRoleOperation
    for RoleStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn update_role(&self, role: NewRoleModel) -> Result<(), RoleStoreError> {
        self.conn.immediate_transaction::<_, RoleStoreError, _>(|| {
            let r = role::table
                .filter(
                    role::name
                        .eq(&role.name)
                        .and(role::service_id.eq(&role.service_id)),
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

            if r.is_some() {
                update(role::table)
                    .filter(
                        role::name
                            .eq(&role.name)
                            .and(role::org_id.eq(&role.org_id))
                            .and(role::service_id.eq(&role.service_id))
                            .and(role::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(role::end_commit_num.eq(&role.start_commit_num))
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
                        _ => {
                            RoleStoreError::InternalError(InternalError::from_source(Box::new(err)))
                        }
                    })?;
            }

            insert_into(role::table)
                .values(&role)
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

            Ok(())
        })
    }
}
