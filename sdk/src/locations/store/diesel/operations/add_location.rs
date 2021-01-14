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

use super::LocationStoreOperations;
use crate::locations::store::diesel::{
    schema::{location, location_attribute},
    LocationStoreError,
};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::{ConstraintViolationError, ConstraintViolationType, InternalError};
use crate::locations::store::diesel::models::{
    LocationAttributeModel, LocationModel, NewLocationAttributeModel, NewLocationModel,
};
use diesel::{
    dsl::{insert_into, update},
    prelude::*,
    result::{DatabaseErrorKind, Error as dsl_error},
};

pub(in crate::locations::store::diesel) trait LocationStoreAddLocationOperation {
    fn add_location(
        &self,
        location: NewLocationModel,
        attributes: Vec<NewLocationAttributeModel>,
        current_commit_num: i64,
    ) -> Result<(), LocationStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> LocationStoreAddLocationOperation
    for LocationStoreOperations<'a, diesel::pg::PgConnection>
{
    fn add_location(
        &self,
        location: NewLocationModel,
        attributes: Vec<NewLocationAttributeModel>,
        current_commit_num: i64,
    ) -> Result<(), LocationStoreError> {
        self.conn
            .build_transaction()
            .read_write()
            .run::<_, LocationStoreError, _>(|| {
                let duplicate_loc = location::table
                    .filter(
                        location::location_id
                            .eq(&location.location_id)
                            .and(location::service_id.eq(&location.service_id))
                            .and(location.end_commit_num.eq(&MAX_COMMIT_NUM)),
                    )
                    .first::<LocationModel>(self.conn)
                    .map(Some)
                    .or_else(|err| {
                        if err == dsl_error::NotFound {
                            Ok(None)
                        } else {
                            Err(err)
                        }
                    })
                    .map_err(|err| {
                        LocationStoreError::InternalError(InternalError::from_source(Box::new(err)))
                    })?;

                if duplicate_loc.is_some() {
                    update(location::table)
                        .filter(
                            location::location_id
                                .eq(&location.location_id)
                                .and(location::service_id.eq(&location.service_id))
                                .and(location::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(location::end_commit_num.eq(current_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(|err| match err {
                            dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                                LocationStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::Unique,
                                        Box::new(err),
                                    ),
                                )
                            }
                            dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                                LocationStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::ForeignKey,
                                        Box::new(err),
                                    ),
                                )
                            }
                            _ => LocationStoreError::InternalError(InternalError::from_source(
                                Box::new(err),
                            )),
                        })?;
                }

                insert_into(location::table)
                    .values(&location)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(|err| match err {
                        dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                            LocationStoreError::ConstraintViolationError(
                                ConstraintViolationError::from_source_with_violation_type(
                                    ConstraintViolationType::Unique,
                                    Box::new(err),
                                ),
                            )
                        }
                        dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                            LocationStoreError::ConstraintViolationError(
                                ConstraintViolationError::from_source_with_violation_type(
                                    ConstraintViolationType::ForeignKey,
                                    Box::new(err),
                                ),
                            )
                        }
                        _ => LocationStoreError::InternalError(InternalError::from_source(
                            Box::new(err),
                        )),
                    })?;

                for attr in attributes {
                    let duplicate_attr = location_attribute::table
                        .filter(
                            location_attribute::location_id
                                .eq(&attr.location_id)
                                .and(location_attribute::property_name.eq(&attr.property_name))
                                .and(location_attribute::service_id.eq(&attr.service_id))
                                .and(location_attribute::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .first::<LocationAttributeModel>(self.conn)
                        .map(Some)
                        .or_else(|err| {
                            if err == dsl_error::NotFound {
                                Ok(None)
                            } else {
                                Err(err)
                            }
                        })
                        .map_err(|err| {
                            LocationStoreError::InternalError(InternalError::from_source(Box::new(
                                err,
                            )))
                        })?;

                    if duplicate_attr.is_some() {
                        update(location_attribute::table)
                            .filter(
                                location_attribute::location_id
                                    .eq(&attr.location_id)
                                    .and(location_attribute::property_name.eq(&attr.property_name))
                                    .and(location_attribute::service_id.eq(&attr.service_id))
                                    .and(location_attribute::end_commit_num.eq(MAX_COMMIT_NUM)),
                            )
                            .set(location_attribute::end_commit_num.eq(current_commit_num))
                            .execute(self.conn)
                            .map(|_| ())
                            .map_err(|err| match err {
                                dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                                    LocationStoreError::ConstraintViolationError(
                                        ConstraintViolationError::from_source_with_violation_type(
                                            ConstraintViolationType::Unique,
                                            Box::new(err),
                                        ),
                                    )
                                }
                                dsl_error::DatabaseError(
                                    DatabaseErrorKind::ForeignKeyViolation,
                                    _,
                                ) => LocationStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::ForeignKey,
                                        Box::new(err),
                                    ),
                                ),
                                _ => LocationStoreError::InternalError(InternalError::from_source(
                                    Box::new(err),
                                )),
                            })?;
                    }

                    insert_into(location_attribute::table)
                        .values(&attr)
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(|err| match err {
                            dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                                LocationStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::Unique,
                                        Box::new(err),
                                    ),
                                )
                            }
                            dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                                LocationStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::ForeignKey,
                                        Box::new(err),
                                    ),
                                )
                            }
                            _ => LocationStoreError::InternalError(InternalError::from_source(
                                Box::new(err),
                            )),
                        })?;
                }

                Ok(())
            })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> LocationStoreAddLocationOperation
    for LocationStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn add_location(
        &self,
        location: NewLocationModel,
        attributes: Vec<NewLocationAttributeModel>,
        current_commit_num: i64,
    ) -> Result<(), LocationStoreError> {
        self.conn
            .immediate_transaction::<_, LocationStoreError, _>(|| {
                let duplicate_loc = location::table
                    .filter(
                        location::location_id
                            .eq(&location.location_id)
                            .and(location::service_id.eq(&location.service_id))
                            .and(location.end_commit_num.eq(&MAX_COMMIT_NUM)),
                    )
                    .first::<LocationModel>(self.conn)
                    .map(Some)
                    .or_else(|err| {
                        if err == dsl_error::NotFound {
                            Ok(None)
                        } else {
                            Err(err)
                        }
                    })
                    .map_err(|err| {
                        LocationStoreError::InternalError(InternalError::from_source(Box::new(err)))
                    })?;

                if duplicate_loc.is_some() {
                    update(location::table)
                        .filter(
                            location::location_id
                                .eq(&location.location_id)
                                .and(location::service_id.eq(&location.service_id))
                                .and(location::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(location::end_commit_num.eq(current_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(|err| match err {
                            dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                                LocationStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::Unique,
                                        Box::new(err),
                                    ),
                                )
                            }
                            dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                                LocationStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::ForeignKey,
                                        Box::new(err),
                                    ),
                                )
                            }
                            _ => LocationStoreError::InternalError(InternalError::from_source(
                                Box::new(err),
                            )),
                        })?;
                }

                insert_into(location::table)
                    .values(&location)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(|err| match err {
                        dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                            LocationStoreError::ConstraintViolationError(
                                ConstraintViolationError::from_source_with_violation_type(
                                    ConstraintViolationType::Unique,
                                    Box::new(err),
                                ),
                            )
                        }
                        dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                            LocationStoreError::ConstraintViolationError(
                                ConstraintViolationError::from_source_with_violation_type(
                                    ConstraintViolationType::ForeignKey,
                                    Box::new(err),
                                ),
                            )
                        }
                        _ => LocationStoreError::InternalError(InternalError::from_source(
                            Box::new(err),
                        )),
                    })?;

                for attr in attributes {
                    let duplicate_attr = location_attribute::table
                        .filter(
                            location_attribute::location_id
                                .eq(&attr.location_id)
                                .and(location_attribute::property_name.eq(&attr.property_name))
                                .and(location_attribute::service_id.eq(&attr.service_id))
                                .and(location_attribute::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .first::<LocationAttributeModel>(self.conn)
                        .map(Some)
                        .or_else(|err| {
                            if err == dsl_error::NotFound {
                                Ok(None)
                            } else {
                                Err(err)
                            }
                        })
                        .map_err(|err| {
                            LocationStoreError::InternalError(InternalError::from_source(Box::new(
                                err,
                            )))
                        })?;

                    if duplicate_attr.is_some() {
                        update(location_attribute::table)
                            .filter(
                                location_attribute::location_id
                                    .eq(&attr.location_id)
                                    .and(location_attribute::property_name.eq(&attr.property_name))
                                    .and(location_attribute::service_id.eq(&attr.service_id))
                                    .and(location_attribute::end_commit_num.eq(MAX_COMMIT_NUM)),
                            )
                            .set(location_attribute::end_commit_num.eq(current_commit_num))
                            .execute(self.conn)
                            .map(|_| ())
                            .map_err(|err| match err {
                                dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                                    LocationStoreError::ConstraintViolationError(
                                        ConstraintViolationError::from_source_with_violation_type(
                                            ConstraintViolationType::Unique,
                                            Box::new(err),
                                        ),
                                    )
                                }
                                dsl_error::DatabaseError(
                                    DatabaseErrorKind::ForeignKeyViolation,
                                    _,
                                ) => LocationStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::ForeignKey,
                                        Box::new(err),
                                    ),
                                ),
                                _ => LocationStoreError::InternalError(InternalError::from_source(
                                    Box::new(err),
                                )),
                            })?;
                    }

                    insert_into(location_attribute::table)
                        .values(&attr)
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(|err| match err {
                            dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                                LocationStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::Unique,
                                        Box::new(err),
                                    ),
                                )
                            }
                            dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                                LocationStoreError::ConstraintViolationError(
                                    ConstraintViolationError::from_source_with_violation_type(
                                        ConstraintViolationType::ForeignKey,
                                        Box::new(err),
                                    ),
                                )
                            }
                            _ => LocationStoreError::InternalError(InternalError::from_source(
                                Box::new(err),
                            )),
                        })?;
                }

                Ok(())
            })
    }
}
