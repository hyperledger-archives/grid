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
use crate::pike::store::diesel::{
    schema::{agent, role},
    PikeStoreError,
};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::{ConstraintViolationError, ConstraintViolationType, InternalError};
use crate::pike::store::diesel::models::{AgentModel, NewAgentModel, NewRoleModel, RoleModel};
use diesel::{
    dsl::{insert_into, update},
    prelude::*,
    result::{DatabaseErrorKind, Error as dsl_error},
};

pub(in crate::pike::store::diesel) trait PikeStoreAddAgentOperation {
    fn add_agent(
        &self,
        agent: NewAgentModel,
        roles: Vec<NewRoleModel>,
    ) -> Result<(), PikeStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PikeStoreAddAgentOperation for PikeStoreOperations<'a, diesel::pg::PgConnection> {
    fn add_agent(
        &self,
        agent: NewAgentModel,
        roles: Vec<NewRoleModel>,
    ) -> Result<(), PikeStoreError> {
        self.conn
            .build_transaction()
            .read_write()
            .run::<_, PikeStoreError, _>(|| {
                let duplicate_agent = agent::table
                    .filter(
                        agent::public_key
                            .eq(&agent.public_key)
                            .and(agent::service_id.eq(&agent.service_id))
                            .and(agent::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .first::<AgentModel>(self.conn)
                    .map(Some)
                    .or_else(|err| {
                        if err == dsl_error::NotFound {
                            Ok(None)
                        } else {
                            Err(err)
                        }
                    })
                    .map_err(|err| {
                        PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                    })?;

                if duplicate_agent.is_some() {
                    update(agent::table)
                        .filter(
                            agent::public_key
                                .eq(&agent.public_key)
                                .and(agent::service_id.eq(&agent.service_id))
                                .and(agent::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(agent::end_commit_num.eq(agent.start_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(|err| match err {
                            dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                                PikeStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::Unique,
                                        Box::new(err),
                                    ),
                                )
                            }
                            dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                                PikeStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::ForeignKey,
                                        Box::new(err),
                                    ),
                                )
                            }
                            _ => PikeStoreError::InternalError(InternalError::from_source(
                                Box::new(err),
                            )),
                        })?;
                }

                insert_into(agent::table)
                    .values(&agent)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(|err| match err {
                        dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                            PikeStoreError::ConstraintViolationError(
                                ConstraintViolationError::from_source_with_violation_type(
                                    ConstraintViolationType::Unique,
                                    Box::new(err),
                                ),
                            )
                        }
                        dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                            PikeStoreError::ConstraintViolationError(
                                ConstraintViolationError::from_source_with_violation_type(
                                    ConstraintViolationType::ForeignKey,
                                    Box::new(err),
                                ),
                            )
                        }
                        _ => {
                            PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                        }
                    })?;

                for role in roles {
                    let duplicate_role = role::table
                        .filter(
                            role::public_key
                                .eq(&role.public_key)
                                .and(role::role_name.eq(&role.role_name))
                                .and(role::service_id.eq(&role.service_id))
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
                            PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                        })?;

                    if duplicate_role.is_some() {
                        update(role::table)
                            .filter(
                                role::public_key
                                    .eq(&role.public_key)
                                    .and(role::role_name.eq(&role.role_name))
                                    .and(role::service_id.eq(&role.service_id))
                                    .and(role::end_commit_num.eq(MAX_COMMIT_NUM)),
                            )
                            .set(role::end_commit_num.eq(role.start_commit_num))
                            .execute(self.conn)
                            .map(|_| ())
                            .map_err(|err| match err {
                                dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                                    PikeStoreError::ConstraintViolationError(
                                        ConstraintViolationError::from_source_with_violation_type(
                                            ConstraintViolationType::Unique,
                                            Box::new(err),
                                        ),
                                    )
                                }
                                dsl_error::DatabaseError(
                                    DatabaseErrorKind::ForeignKeyViolation,
                                    _,
                                ) => PikeStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::ForeignKey,
                                        Box::new(err),
                                    ),
                                ),
                                _ => PikeStoreError::InternalError(InternalError::from_source(
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
                                PikeStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::Unique,
                                        Box::new(err),
                                    ),
                                )
                            }
                            dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                                PikeStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::ForeignKey,
                                        Box::new(err),
                                    ),
                                )
                            }
                            _ => PikeStoreError::InternalError(InternalError::from_source(
                                Box::new(err),
                            )),
                        })?;
                }

                Ok(())
            })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PikeStoreAddAgentOperation for PikeStoreOperations<'a, diesel::sqlite::SqliteConnection> {
    fn add_agent(
        &self,
        agent: NewAgentModel,
        roles: Vec<NewRoleModel>,
    ) -> Result<(), PikeStoreError> {
        self.conn.immediate_transaction::<_, PikeStoreError, _>(|| {
            let duplicate_agent = agent::table
                .filter(
                    agent::public_key
                        .eq(&agent.public_key)
                        .and(agent::service_id.eq(&agent.service_id))
                        .and(agent::end_commit_num.eq(MAX_COMMIT_NUM)),
                )
                .first::<AgentModel>(self.conn)
                .map(Some)
                .or_else(|err| {
                    if err == dsl_error::NotFound {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                })
                .map_err(|err| {
                    PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            if duplicate_agent.is_some() {
                update(agent::table)
                    .filter(
                        agent::public_key
                            .eq(&agent.public_key)
                            .and(agent::service_id.eq(&agent.service_id))
                            .and(agent::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(agent::end_commit_num.eq(agent.start_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(|err| match err {
                        dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                            PikeStoreError::ConstraintViolationError(
                                ConstraintViolationError::from_source_with_violation_type(
                                    ConstraintViolationType::Unique,
                                    Box::new(err),
                                ),
                            )
                        }
                        dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                            PikeStoreError::ConstraintViolationError(
                                ConstraintViolationError::from_source_with_violation_type(
                                    ConstraintViolationType::ForeignKey,
                                    Box::new(err),
                                ),
                            )
                        }
                        _ => {
                            PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                        }
                    })?;
            }

            insert_into(agent::table)
                .values(&agent)
                .execute(self.conn)
                .map(|_| ())
                .map_err(|err| match err {
                    dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                        PikeStoreError::ConstraintViolationError(
                            ConstraintViolationError::from_source_with_violation_type(
                                ConstraintViolationType::Unique,
                                Box::new(err),
                            ),
                        )
                    }
                    dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                        PikeStoreError::ConstraintViolationError(
                            ConstraintViolationError::from_source_with_violation_type(
                                ConstraintViolationType::ForeignKey,
                                Box::new(err),
                            ),
                        )
                    }
                    _ => PikeStoreError::InternalError(InternalError::from_source(Box::new(err))),
                })?;

            for role in roles {
                let duplicate_role = role::table
                    .filter(
                        role::public_key
                            .eq(&role.public_key)
                            .and(role::role_name.eq(&role.role_name))
                            .and(role::service_id.eq(&role.service_id))
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
                        PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                    })?;

                if duplicate_role.is_some() {
                    update(role::table)
                        .filter(
                            role::public_key
                                .eq(&role.public_key)
                                .and(role::role_name.eq(&role.role_name))
                                .and(role::service_id.eq(&role.service_id))
                                .and(role::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(role::end_commit_num.eq(role.start_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(|err| match err {
                            dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                                PikeStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::Unique,
                                        Box::new(err),
                                    ),
                                )
                            }
                            dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                                PikeStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::ForeignKey,
                                        Box::new(err),
                                    ),
                                )
                            }
                            _ => PikeStoreError::InternalError(InternalError::from_source(
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
                            PikeStoreError::ConstraintViolationError(
                                ConstraintViolationError::from_source_with_violation_type(
                                    ConstraintViolationType::Unique,
                                    Box::new(err),
                                ),
                            )
                        }
                        dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                            PikeStoreError::ConstraintViolationError(
                                ConstraintViolationError::from_source_with_violation_type(
                                    ConstraintViolationType::ForeignKey,
                                    Box::new(err),
                                ),
                            )
                        }
                        _ => {
                            PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                        }
                    })?;
            }

            Ok(())
        })
    }
}
