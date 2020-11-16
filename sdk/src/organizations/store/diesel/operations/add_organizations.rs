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

use super::OrganizationStoreOperations;
use crate::organizations::store::diesel::{schema::organization, OrganizationStoreError};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::{ConstraintViolationError, ConstraintViolationType, InternalError};
use crate::organizations::store::diesel::models::{NewOrganizationModel, OrganizationModel};
use diesel::{
    dsl::insert_into,
    prelude::*,
    result::{DatabaseErrorKind, Error as dsl_error},
};

pub(in crate::organizations::store::diesel) trait OrganizationStoreAddOrganizationsOperation {
    fn add_organizations(
        &self,
        orgs: Vec<NewOrganizationModel>,
    ) -> Result<(), OrganizationStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> OrganizationStoreAddOrganizationsOperation
    for OrganizationStoreOperations<'a, diesel::pg::PgConnection>
{
    fn add_organizations(
        &self,
        orgs: Vec<NewOrganizationModel>,
    ) -> Result<(), OrganizationStoreError> {
        for org in orgs {
            let duplicate_org = organization::table
                .filter(
                    organization::org_id
                        .eq(&org.org_id)
                        .and(organization::service_id.eq(&org.service_id))
                        .and(organization::end_commit_num.eq(MAX_COMMIT_NUM)),
                )
                .first::<OrganizationModel>(self.conn)
                .map(Some)
                .or_else(|err| {
                    if err == dsl_error::NotFound {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                })
                .map_err(|err| {
                    OrganizationStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;
            if duplicate_org.is_some() {
                return Err(OrganizationStoreError::ConstraintViolationError(
                    ConstraintViolationError::with_violation_type(ConstraintViolationType::Unique),
                ));
            }

            insert_into(organization::table)
                .values(org)
                .execute(self.conn)
                .map(|_| ())
                .map_err(|err| match err {
                    dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                        OrganizationStoreError::ConstraintViolationError(
                            ConstraintViolationError::from_source_with_violation_type(
                                ConstraintViolationType::Unique,
                                Box::new(err),
                            ),
                        )
                    }
                    dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                        OrganizationStoreError::ConstraintViolationError(
                            ConstraintViolationError::from_source_with_violation_type(
                                ConstraintViolationType::ForeignKey,
                                Box::new(err),
                            ),
                        )
                    }
                    _ => OrganizationStoreError::InternalError(InternalError::from_source(
                        Box::new(err),
                    )),
                })?;
        }

        Ok(())
    }
}

#[cfg(feature = "sqlite")]
impl<'a> OrganizationStoreAddOrganizationsOperation
    for OrganizationStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn add_organizations(
        &self,
        orgs: Vec<NewOrganizationModel>,
    ) -> Result<(), OrganizationStoreError> {
        for org in orgs {
            let duplicate_org = organization::table
                .filter(
                    organization::org_id
                        .eq(&org.org_id)
                        .and(organization::service_id.eq(&org.service_id))
                        .and(organization::end_commit_num.eq(MAX_COMMIT_NUM)),
                )
                .first::<OrganizationModel>(self.conn)
                .map(Some)
                .or_else(|err| {
                    if err == dsl_error::NotFound {
                        Ok(None)
                    } else {
                        Err(err)
                    }
                })
                .map_err(|err| {
                    OrganizationStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;
            if duplicate_org.is_some() {
                return Err(OrganizationStoreError::ConstraintViolationError(
                    ConstraintViolationError::with_violation_type(ConstraintViolationType::Unique),
                ));
            }

            insert_into(organization::table)
                .values(org)
                .execute(self.conn)
                .map(|_| ())
                .map_err(|err| match err {
                    dsl_error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                        OrganizationStoreError::ConstraintViolationError(
                            ConstraintViolationError::from_source_with_violation_type(
                                ConstraintViolationType::Unique,
                                Box::new(err),
                            ),
                        )
                    }
                    dsl_error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
                        OrganizationStoreError::ConstraintViolationError(
                            ConstraintViolationError::from_source_with_violation_type(
                                ConstraintViolationType::ForeignKey,
                                Box::new(err),
                            ),
                        )
                    }
                    _ => OrganizationStoreError::InternalError(InternalError::from_source(
                        Box::new(err),
                    )),
                })?;
        }

        Ok(())
    }
}
