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

use super::TrackAndTraceStoreOperations;
use crate::track_and_trace::store::diesel::{schema::property, TrackAndTraceStoreError};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::{ConstraintViolationError, ConstraintViolationType, InternalError};
use crate::track_and_trace::store::diesel::models::{NewPropertyModel, PropertyModel};

use diesel::{
    dsl::{insert_into, update},
    prelude::*,
    result::{DatabaseErrorKind, Error as dsl_error},
};

pub(in crate::track_and_trace::store::diesel) trait TrackAndTraceStoreAddPropertiesOperation {
    fn add_properties(
        &self,
        properties: Vec<NewPropertyModel>,
    ) -> Result<(), TrackAndTraceStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> TrackAndTraceStoreAddPropertiesOperation
    for TrackAndTraceStoreOperations<'a, diesel::pg::PgConnection>
{
    fn add_properties(
        &self,
        properties: Vec<NewPropertyModel>,
    ) -> Result<(), TrackAndTraceStoreError> {
        self.conn
            .build_transaction()
            .read_write()
            .run::<_, TrackAndTraceStoreError, _>(|| {
                for prop in properties {
                    let duplicate = property::table
                        .filter(
                            property::name
                                .eq(&prop.name)
                                .and(property::record_id.eq(&prop.record_id))
                                .and(property::end_commit_num.eq(MAX_COMMIT_NUM))
                                .and(property::service_id.eq(&prop.service_id)),
                        )
                        .first::<PropertyModel>(self.conn)
                        .map(Some)
                        .or_else(|err| {
                            if err == dsl_error::NotFound {
                                Ok(None)
                            } else {
                                Err(err)
                            }
                        })
                        .map_err(|err| {
                            TrackAndTraceStoreError::InternalError(InternalError::from_source(
                                Box::new(err),
                            ))
                        })?;

                    if duplicate.is_some() {
                        update(property::table)
                            .filter(
                                property::name
                                    .eq(&prop.name)
                                    .and(property::record_id.eq(&prop.record_id))
                                    .and(property::end_commit_num.eq(MAX_COMMIT_NUM))
                                    .and(property::service_id.eq(&prop.service_id)),
                            )
                            .set(property::end_commit_num.eq(&prop.start_commit_num))
                            .execute(self.conn)
                            .map(|_| ())
                            .map_err(|err| match err {
                                dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                                    TrackAndTraceStoreError::ConstraintViolationError(
                                        ConstraintViolationError::from_source_with_violation_type(
                                            ConstraintViolationType::Unique,
                                            Box::new(err),
                                        ),
                                    )
                                }
                                dsl_error::DatabaseError(
                                    DatabaseErrorKind::ForeignKeyViolation,
                                    _,
                                ) => TrackAndTraceStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::ForeignKey,
                                        Box::new(err),
                                    ),
                                ),
                                _ => TrackAndTraceStoreError::InternalError(
                                    InternalError::from_source(Box::new(err)),
                                ),
                            })?;
                    }

                    insert_into(property::table)
                        .values(&prop)
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(|err| match err {
                            dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                                TrackAndTraceStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::Unique,
                                        Box::new(err),
                                    ),
                                )
                            }
                            dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                                TrackAndTraceStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::ForeignKey,
                                        Box::new(err),
                                    ),
                                )
                            }
                            _ => TrackAndTraceStoreError::InternalError(
                                InternalError::from_source(Box::new(err)),
                            ),
                        })?;
                }

                Ok(())
            })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> TrackAndTraceStoreAddPropertiesOperation
    for TrackAndTraceStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn add_properties(
        &self,
        properties: Vec<NewPropertyModel>,
    ) -> Result<(), TrackAndTraceStoreError> {
        self.conn
            .immediate_transaction::<_, TrackAndTraceStoreError, _>(|| {
                for prop in properties {
                    let duplicate = property::table
                        .filter(
                            property::name
                                .eq(&prop.name)
                                .and(property::record_id.eq(&prop.record_id))
                                .and(property::end_commit_num.eq(MAX_COMMIT_NUM))
                                .and(property::service_id.eq(&prop.service_id)),
                        )
                        .first::<PropertyModel>(self.conn)
                        .map(Some)
                        .or_else(|err| {
                            if err == dsl_error::NotFound {
                                Ok(None)
                            } else {
                                Err(err)
                            }
                        })
                        .map_err(|err| {
                            TrackAndTraceStoreError::InternalError(InternalError::from_source(
                                Box::new(err),
                            ))
                        })?;

                    if duplicate.is_some() {
                        update(property::table)
                            .filter(
                                property::name
                                    .eq(&prop.name)
                                    .and(property::record_id.eq(&prop.record_id))
                                    .and(property::end_commit_num.eq(MAX_COMMIT_NUM))
                                    .and(property::service_id.eq(&prop.service_id)),
                            )
                            .set(property::end_commit_num.eq(&prop.start_commit_num))
                            .execute(self.conn)
                            .map(|_| ())
                            .map_err(|err| match err {
                                dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                                    TrackAndTraceStoreError::ConstraintViolationError(
                                        ConstraintViolationError::from_source_with_violation_type(
                                            ConstraintViolationType::Unique,
                                            Box::new(err),
                                        ),
                                    )
                                }
                                dsl_error::DatabaseError(
                                    DatabaseErrorKind::ForeignKeyViolation,
                                    _,
                                ) => TrackAndTraceStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::ForeignKey,
                                        Box::new(err),
                                    ),
                                ),
                                _ => TrackAndTraceStoreError::InternalError(
                                    InternalError::from_source(Box::new(err)),
                                ),
                            })?;
                    }

                    insert_into(property::table)
                        .values(&prop)
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(|err| match err {
                            dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                                TrackAndTraceStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::Unique,
                                        Box::new(err),
                                    ),
                                )
                            }
                            dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                                TrackAndTraceStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::ForeignKey,
                                        Box::new(err),
                                    ),
                                )
                            }
                            _ => TrackAndTraceStoreError::InternalError(
                                InternalError::from_source(Box::new(err)),
                            ),
                        })?;
                }

                Ok(())
            })
    }
}
