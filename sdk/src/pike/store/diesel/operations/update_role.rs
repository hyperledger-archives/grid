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
    schema::{pike_allowed_orgs, pike_inherit_from, pike_permissions, pike_role},
    PikeStoreError,
};

use crate::commits::MAX_COMMIT_NUM;
use crate::error::InternalError;
use crate::pike::store::diesel::models::{
    AllowedOrgModel, InheritFromModel, NewAllowedOrgModel, NewInheritFromModel, NewPermissionModel,
    NewRoleModel, PermissionModel, RoleModel,
};
use diesel::{
    dsl::{insert_into, update},
    prelude::*,
    result::Error as dsl_error,
};

pub(in crate::pike::store::diesel) trait PikeStoreUpdateRoleOperation {
    fn update_role(
        &self,
        role: NewRoleModel,
        inherit_from: Vec<NewInheritFromModel>,
        permissions: Vec<NewPermissionModel>,
        allowed_orgs: Vec<NewAllowedOrgModel>,
    ) -> Result<(), PikeStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PikeStoreUpdateRoleOperation for PikeStoreOperations<'a, diesel::pg::PgConnection> {
    fn update_role(
        &self,
        role: NewRoleModel,
        inherit_from: Vec<NewInheritFromModel>,
        permissions: Vec<NewPermissionModel>,
        allowed_orgs: Vec<NewAllowedOrgModel>,
    ) -> Result<(), PikeStoreError> {
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let mut query = pike_role::table.into_boxed().filter(
                pike_role::name
                    .eq(&role.name)
                    .and(pike_role::org_id.eq(&role.org_id))
                    .and(pike_role::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

            if let Some(service_id) = &role.service_id {
                query = query.filter(pike_role::service_id.eq(service_id));
            } else {
                query = query.filter(pike_role::service_id.is_null());
            }

            let duplicate_role = query
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
                update(pike_role::table)
                    .filter(
                        pike_role::name
                            .eq(&role.name)
                            .and(pike_role::org_id.eq(&role.org_id))
                            .and(pike_role::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(pike_role::end_commit_num.eq(role.start_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            insert_into(pike_role::table)
                .values(&role)
                .execute(self.conn)
                .map(|_| ())
                .map_err(PikeStoreError::from)?;

            for i in inherit_from {
                let mut query = pike_inherit_from::table.into_boxed().filter(
                    pike_inherit_from::role_name
                        .eq(&role.name)
                        .and(pike_inherit_from::role_name.eq(&i.role_name))
                        .and(pike_inherit_from::org_id.eq(&role.org_id))
                        .and(pike_inherit_from::org_id.eq(&i.org_id))
                        .and(pike_inherit_from::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

                if let Some(service_id) = &i.service_id {
                    query = query.filter(pike_inherit_from::service_id.eq(service_id));
                } else {
                    query = query.filter(pike_inherit_from::service_id.is_null());
                }

                let duplicate = query
                    .first::<InheritFromModel>(self.conn)
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
                    update(pike_inherit_from::table)
                        .filter(
                            pike_inherit_from::role_name
                                .eq(&role.name)
                                .and(pike_inherit_from::role_name.eq(&i.role_name))
                                .and(pike_inherit_from::org_id.eq(&role.org_id))
                                .and(pike_inherit_from::org_id.eq(&i.org_id))
                                .and(pike_inherit_from::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(pike_inherit_from::end_commit_num.eq(i.start_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PikeStoreError::from)?;
                }

                insert_into(pike_inherit_from::table)
                    .values(&i)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            for p in permissions {
                let mut query = pike_permissions::table.into_boxed().filter(
                    pike_permissions::role_name
                        .eq(&role.name)
                        .and(pike_permissions::role_name.eq(&p.role_name))
                        .and(pike_permissions::org_id.eq(&role.org_id))
                        .and(pike_permissions::org_id.eq(&p.org_id))
                        .and(pike_permissions::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

                if let Some(service_id) = &p.service_id {
                    query = query.filter(pike_permissions::service_id.eq(service_id));
                } else {
                    query = query.filter(pike_permissions::service_id.is_null());
                }

                let duplicate = query
                    .first::<PermissionModel>(self.conn)
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
                    update(pike_permissions::table)
                        .filter(
                            pike_permissions::role_name
                                .eq(&role.name)
                                .and(pike_permissions::role_name.eq(&p.role_name))
                                .and(pike_permissions::org_id.eq(&role.org_id))
                                .and(pike_permissions::org_id.eq(&p.org_id))
                                .and(pike_permissions::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(pike_permissions::end_commit_num.eq(p.start_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PikeStoreError::from)?;
                }

                insert_into(pike_permissions::table)
                    .values(&p)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            for a in allowed_orgs {
                let mut query = pike_allowed_orgs::table.into_boxed().filter(
                    pike_allowed_orgs::role_name
                        .eq(&role.name)
                        .and(pike_allowed_orgs::role_name.eq(&a.role_name))
                        .and(pike_allowed_orgs::org_id.eq(&role.org_id))
                        .and(pike_allowed_orgs::org_id.eq(&a.org_id))
                        .and(pike_allowed_orgs::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

                if let Some(service_id) = &a.service_id {
                    query = query.filter(pike_allowed_orgs::service_id.eq(service_id));
                } else {
                    query = query.filter(pike_allowed_orgs::service_id.is_null());
                }

                let duplicate = query
                    .first::<AllowedOrgModel>(self.conn)
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
                    update(pike_allowed_orgs::table)
                        .filter(
                            pike_allowed_orgs::role_name
                                .eq(&role.name)
                                .and(pike_allowed_orgs::role_name.eq(&a.role_name))
                                .and(pike_allowed_orgs::org_id.eq(&role.org_id))
                                .and(pike_allowed_orgs::org_id.eq(&a.org_id))
                                .and(pike_allowed_orgs::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(pike_allowed_orgs::end_commit_num.eq(a.start_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PikeStoreError::from)?;
                }

                insert_into(pike_allowed_orgs::table)
                    .values(&a)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PikeStoreUpdateRoleOperation
    for PikeStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn update_role(
        &self,
        role: NewRoleModel,
        inherit_from: Vec<NewInheritFromModel>,
        permissions: Vec<NewPermissionModel>,
        allowed_orgs: Vec<NewAllowedOrgModel>,
    ) -> Result<(), PikeStoreError> {
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let mut query = pike_role::table.into_boxed().filter(
                pike_role::name
                    .eq(&role.name)
                    .and(pike_role::org_id.eq(&role.org_id))
                    .and(pike_role::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

            if let Some(service_id) = &role.service_id {
                query = query.filter(pike_role::service_id.eq(service_id));
            } else {
                query = query.filter(pike_role::service_id.is_null());
            }

            let duplicate_role = query
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
                update(pike_role::table)
                    .filter(
                        pike_role::name
                            .eq(&role.name)
                            .and(pike_role::org_id.eq(&role.org_id))
                            .and(pike_role::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(pike_role::end_commit_num.eq(role.start_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            insert_into(pike_role::table)
                .values(&role)
                .execute(self.conn)
                .map(|_| ())
                .map_err(PikeStoreError::from)?;

            for i in inherit_from {
                let mut query = pike_inherit_from::table.into_boxed().filter(
                    pike_inherit_from::role_name
                        .eq(&role.name)
                        .and(pike_inherit_from::role_name.eq(&i.role_name))
                        .and(pike_inherit_from::org_id.eq(&role.org_id))
                        .and(pike_inherit_from::org_id.eq(&i.org_id))
                        .and(pike_inherit_from::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

                if let Some(service_id) = &i.service_id {
                    query = query.filter(pike_inherit_from::service_id.eq(service_id));
                } else {
                    query = query.filter(pike_inherit_from::service_id.is_null());
                }

                let duplicate = query
                    .first::<InheritFromModel>(self.conn)
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
                    update(pike_inherit_from::table)
                        .filter(
                            pike_inherit_from::role_name
                                .eq(&role.name)
                                .and(pike_inherit_from::role_name.eq(&i.role_name))
                                .and(pike_inherit_from::org_id.eq(&role.org_id))
                                .and(pike_inherit_from::org_id.eq(&i.org_id))
                                .and(pike_inherit_from::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(pike_inherit_from::end_commit_num.eq(i.start_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PikeStoreError::from)?;
                }

                insert_into(pike_inherit_from::table)
                    .values(&i)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            for p in permissions {
                let mut query = pike_permissions::table.into_boxed().filter(
                    pike_permissions::role_name
                        .eq(&role.name)
                        .and(pike_permissions::role_name.eq(&p.role_name))
                        .and(pike_permissions::org_id.eq(&role.org_id))
                        .and(pike_permissions::org_id.eq(&p.org_id))
                        .and(pike_permissions::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

                if let Some(service_id) = &p.service_id {
                    query = query.filter(pike_permissions::service_id.eq(service_id));
                } else {
                    query = query.filter(pike_permissions::service_id.is_null());
                }

                let duplicate = query
                    .first::<PermissionModel>(self.conn)
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
                    update(pike_permissions::table)
                        .filter(
                            pike_permissions::role_name
                                .eq(&role.name)
                                .and(pike_permissions::role_name.eq(&p.role_name))
                                .and(pike_permissions::org_id.eq(&role.org_id))
                                .and(pike_permissions::org_id.eq(&p.org_id))
                                .and(pike_permissions::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(pike_permissions::end_commit_num.eq(p.start_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PikeStoreError::from)?;
                }

                insert_into(pike_permissions::table)
                    .values(&p)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            for a in allowed_orgs {
                let mut query = pike_allowed_orgs::table.into_boxed().filter(
                    pike_allowed_orgs::role_name
                        .eq(&role.name)
                        .and(pike_allowed_orgs::role_name.eq(&a.role_name))
                        .and(pike_allowed_orgs::org_id.eq(&role.org_id))
                        .and(pike_allowed_orgs::org_id.eq(&a.org_id))
                        .and(pike_allowed_orgs::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

                if let Some(service_id) = &a.service_id {
                    query = query.filter(pike_allowed_orgs::service_id.eq(service_id));
                } else {
                    query = query.filter(pike_allowed_orgs::service_id.is_null());
                }

                let duplicate = query
                    .first::<AllowedOrgModel>(self.conn)
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
                    update(pike_allowed_orgs::table)
                        .filter(
                            pike_allowed_orgs::role_name
                                .eq(&role.name)
                                .and(pike_allowed_orgs::role_name.eq(&a.role_name))
                                .and(pike_allowed_orgs::org_id.eq(&role.org_id))
                                .and(pike_allowed_orgs::org_id.eq(&a.org_id))
                                .and(pike_allowed_orgs::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .set(pike_allowed_orgs::end_commit_num.eq(a.start_commit_num))
                        .execute(self.conn)
                        .map(|_| ())
                        .map_err(PikeStoreError::from)?;
                }

                insert_into(pike_allowed_orgs::table)
                    .values(&a)
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            Ok(())
        })
    }
}
