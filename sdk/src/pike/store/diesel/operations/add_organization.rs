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
    schema::{
        pike_organization, pike_organization_alternate_id, pike_organization_location_assoc,
        pike_organization_metadata,
    },
    PikeStoreError,
};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::pike::store::diesel::models::{
    AlternateIDModel, LocationAssociationModel, NewAlternateIDModel, NewLocationAssociationModel,
    NewOrganizationMetadataModel, NewOrganizationModel, OrganizationMetadataModel,
    OrganizationModel,
};
use diesel::{
    dsl::{insert_into, update},
    prelude::*,
    result::Error as dsl_error,
};

pub(in crate::pike::store::diesel) trait PikeStoreAddOrganizationOperation {
    fn add_organization(
        &self,
        org: NewOrganizationModel,
        locations: Vec<NewLocationAssociationModel>,
        alternate_ids: Vec<NewAlternateIDModel>,
        metadata: Vec<NewOrganizationMetadataModel>,
    ) -> Result<(), PikeStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PikeStoreAddOrganizationOperation for PikeStoreOperations<'a, diesel::pg::PgConnection> {
    fn add_organization(
        &self,
        org: NewOrganizationModel,
        locations: Vec<NewLocationAssociationModel>,
        alternate_ids: Vec<NewAlternateIDModel>,
        metadata: Vec<NewOrganizationMetadataModel>,
    ) -> Result<(), PikeStoreError> {
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let mut query = pike_organization::table.into_boxed().filter(
                pike_organization::org_id
                    .eq(&org.org_id)
                    .and(pike_organization::service_id.eq(&org.service_id))
                    .and(pike_organization::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

            if let Some(service_id) = &org.service_id {
                query = query.filter(pike_organization::service_id.eq(service_id));
            } else {
                query = query.filter(pike_organization::service_id.is_null());
            }

            let duplicate_org = query
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
                    PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            if duplicate_org.is_some() {
                update(pike_organization::table)
                    .filter(
                        pike_organization::org_id
                            .eq(&org.org_id)
                            .and(pike_organization::service_id.eq(&org.service_id))
                            .and(pike_organization::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(pike_organization::end_commit_num.eq(org.start_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            insert_into(pike_organization::table)
                .values(&org)
                .execute(self.conn)
                .map(|_| ())
                .map_err(PikeStoreError::from)?;

            for location in locations {
                let mut query = pike_organization_location_assoc::table
                    .into_boxed()
                    .select(pike_organization_location_assoc::all_columns)
                    .filter(
                        pike_organization_location_assoc::org_id
                            .eq(&org.org_id)
                            .and(
                                pike_organization_location_assoc::location_id
                                    .eq(&location.location_id),
                            )
                            .and(
                                pike_organization_location_assoc::end_commit_num.eq(MAX_COMMIT_NUM),
                            ),
                    );

                if let Some(service_id) = &location.service_id {
                    query =
                        query.filter(pike_organization_location_assoc::service_id.eq(service_id));
                } else {
                    query = query.filter(pike_organization_location_assoc::service_id.is_null());
                }

                let duplicate = query
                    .first::<LocationAssociationModel>(self.conn)
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

                if duplicate.is_some() {
                    update(pike_organization_location_assoc::table)
                        .filter(
                            pike_organization_location_assoc::org_id
                                .eq(&org.org_id)
                                .and(
                                    pike_organization_location_assoc::location_id
                                        .eq(&location.location_id),
                                )
                                .and(
                                    pike_organization_location_assoc::end_commit_num
                                        .eq(MAX_COMMIT_NUM),
                                ),
                        )
                        .set(
                            pike_organization_location_assoc::end_commit_num
                                .eq(location.start_commit_num),
                        )
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PikeStoreError::from)?;
                }

                insert_into(pike_organization_location_assoc::table)
                    .values(location)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            for entry in alternate_ids {
                let mut query = pike_organization_alternate_id::table
                    .into_boxed()
                    .select(pike_organization_alternate_id::all_columns)
                    .filter(
                        pike_organization_alternate_id::org_id
                            .eq(&entry.org_id)
                            .and(
                                pike_organization_alternate_id::alternate_id_type
                                    .eq(&entry.alternate_id_type),
                            )
                            .and(
                                pike_organization_alternate_id::alternate_id
                                    .eq(&entry.alternate_id),
                            )
                            .and(pike_organization_alternate_id::end_commit_num.eq(MAX_COMMIT_NUM)),
                    );

                if let Some(service_id) = &entry.service_id {
                    query = query.filter(pike_organization_alternate_id::service_id.eq(service_id));
                } else {
                    query = query.filter(pike_organization_alternate_id::service_id.is_null());
                }

                let duplicate = query
                    .first::<AlternateIDModel>(self.conn)
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

                if duplicate.is_some() {
                    update(pike_organization_alternate_id::table)
                        .filter(
                            pike_organization_alternate_id::org_id
                                .eq(&entry.org_id)
                                .and(
                                    pike_organization_alternate_id::alternate_id_type
                                        .eq(&entry.alternate_id_type),
                                )
                                .and(
                                    pike_organization_alternate_id::alternate_id
                                        .eq(&entry.alternate_id),
                                )
                                .and(
                                    pike_organization_alternate_id::end_commit_num
                                        .eq(MAX_COMMIT_NUM),
                                ),
                        )
                        .set(
                            pike_organization_alternate_id::end_commit_num
                                .eq(entry.start_commit_num),
                        )
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PikeStoreError::from)?;
                }

                insert_into(pike_organization_alternate_id::table)
                    .values(entry)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            for data in metadata {
                let mut query = pike_organization_metadata::table
                    .into_boxed()
                    .select(pike_organization_metadata::all_columns)
                    .filter(
                        pike_organization_metadata::org_id
                            .eq(&data.org_id)
                            .and(pike_organization_metadata::end_commit_num.eq(MAX_COMMIT_NUM))
                            .and(pike_organization_metadata::key.eq(&data.key)),
                    );

                if let Some(service_id) = &data.service_id {
                    query = query.filter(pike_organization_metadata::service_id.eq(service_id));
                } else {
                    query = query.filter(pike_organization_metadata::service_id.is_null());
                }

                let duplicate = query
                    .first::<OrganizationMetadataModel>(self.conn)
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

                if duplicate.is_some() {
                    update(pike_organization_metadata::table)
                        .filter(
                            pike_organization_metadata::org_id
                                .eq(&data.org_id)
                                .and(pike_organization_metadata::service_id.eq(&data.service_id))
                                .and(pike_organization_metadata::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(pike_organization_metadata::end_commit_num.eq(data.start_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PikeStoreError::from)?;
                }

                insert_into(pike_organization_metadata::table)
                    .values(data)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PikeStoreAddOrganizationOperation
    for PikeStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn add_organization(
        &self,
        org: NewOrganizationModel,
        locations: Vec<NewLocationAssociationModel>,
        alternate_ids: Vec<NewAlternateIDModel>,
        metadata: Vec<NewOrganizationMetadataModel>,
    ) -> Result<(), PikeStoreError> {
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let mut query = pike_organization::table.into_boxed().filter(
                pike_organization::org_id
                    .eq(&org.org_id)
                    .and(pike_organization::service_id.eq(&org.service_id))
                    .and(pike_organization::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

            if let Some(service_id) = &org.service_id {
                query = query.filter(pike_organization::service_id.eq(service_id));
            } else {
                query = query.filter(pike_organization::service_id.is_null());
            }

            let duplicate_org = query
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
                    PikeStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            if duplicate_org.is_some() {
                update(pike_organization::table)
                    .filter(
                        pike_organization::org_id
                            .eq(&org.org_id)
                            .and(pike_organization::service_id.eq(&org.service_id))
                            .and(pike_organization::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(pike_organization::end_commit_num.eq(org.start_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            insert_into(pike_organization::table)
                .values(&org)
                .execute(self.conn)
                .map(|_| ())
                .map_err(PikeStoreError::from)?;

            for location in locations {
                let mut query = pike_organization_location_assoc::table
                    .into_boxed()
                    .select(pike_organization_location_assoc::all_columns)
                    .filter(
                        pike_organization_location_assoc::org_id
                            .eq(&org.org_id)
                            .and(
                                pike_organization_location_assoc::location_id
                                    .eq(&location.location_id),
                            )
                            .and(
                                pike_organization_location_assoc::end_commit_num.eq(MAX_COMMIT_NUM),
                            ),
                    );

                if let Some(service_id) = &location.service_id {
                    query =
                        query.filter(pike_organization_location_assoc::service_id.eq(service_id));
                } else {
                    query = query.filter(pike_organization_location_assoc::service_id.is_null());
                }

                let duplicate = query
                    .first::<LocationAssociationModel>(self.conn)
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

                if duplicate.is_some() {
                    update(pike_organization_location_assoc::table)
                        .filter(
                            pike_organization_location_assoc::org_id
                                .eq(&org.org_id)
                                .and(
                                    pike_organization_location_assoc::location_id
                                        .eq(&location.location_id),
                                )
                                .and(
                                    pike_organization_location_assoc::end_commit_num
                                        .eq(MAX_COMMIT_NUM),
                                ),
                        )
                        .set(
                            pike_organization_location_assoc::end_commit_num
                                .eq(location.start_commit_num),
                        )
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PikeStoreError::from)?;
                }

                insert_into(pike_organization_location_assoc::table)
                    .values(location)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            for entry in alternate_ids {
                let mut query = pike_organization_alternate_id::table
                    .into_boxed()
                    .select(pike_organization_alternate_id::all_columns)
                    .filter(
                        pike_organization_alternate_id::org_id
                            .eq(&entry.org_id)
                            .and(
                                pike_organization_alternate_id::alternate_id_type
                                    .eq(&entry.alternate_id_type),
                            )
                            .and(
                                pike_organization_alternate_id::alternate_id
                                    .eq(&entry.alternate_id),
                            )
                            .and(pike_organization_alternate_id::end_commit_num.eq(MAX_COMMIT_NUM)),
                    );

                if let Some(service_id) = &entry.service_id {
                    query = query.filter(pike_organization_alternate_id::service_id.eq(service_id));
                } else {
                    query = query.filter(pike_organization_alternate_id::service_id.is_null());
                }

                let duplicate = query
                    .first::<AlternateIDModel>(self.conn)
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

                if duplicate.is_some() {
                    update(pike_organization_alternate_id::table)
                        .filter(
                            pike_organization_alternate_id::org_id
                                .eq(&entry.org_id)
                                .and(
                                    pike_organization_alternate_id::alternate_id_type
                                        .eq(&entry.alternate_id_type),
                                )
                                .and(
                                    pike_organization_alternate_id::alternate_id
                                        .eq(&entry.alternate_id),
                                )
                                .and(
                                    pike_organization_alternate_id::end_commit_num
                                        .eq(MAX_COMMIT_NUM),
                                ),
                        )
                        .set(
                            pike_organization_alternate_id::end_commit_num
                                .eq(entry.start_commit_num),
                        )
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PikeStoreError::from)?;
                }

                insert_into(pike_organization_alternate_id::table)
                    .values(entry)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            for data in metadata {
                let mut query = pike_organization_metadata::table
                    .into_boxed()
                    .select(pike_organization_metadata::all_columns)
                    .filter(
                        pike_organization_metadata::org_id
                            .eq(&data.org_id)
                            .and(pike_organization_metadata::end_commit_num.eq(MAX_COMMIT_NUM))
                            .and(pike_organization_metadata::key.eq(&data.key)),
                    );

                if let Some(service_id) = &data.service_id {
                    query = query.filter(pike_organization_metadata::service_id.eq(service_id));
                } else {
                    query = query.filter(pike_organization_metadata::service_id.is_null());
                }

                let duplicate = query
                    .first::<OrganizationMetadataModel>(self.conn)
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

                if duplicate.is_some() {
                    update(pike_organization_metadata::table)
                        .filter(
                            pike_organization_metadata::org_id
                                .eq(&data.org_id)
                                .and(pike_organization_metadata::service_id.eq(&data.service_id))
                                .and(pike_organization_metadata::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(pike_organization_metadata::end_commit_num.eq(data.start_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PikeStoreError::from)?;
                }

                insert_into(pike_organization_metadata::table)
                    .values(data)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            Ok(())
        })
    }
}
