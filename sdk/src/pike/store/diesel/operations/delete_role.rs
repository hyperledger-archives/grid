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

use super::PikeStoreOperations;
use crate::error::InternalError;
use crate::pike::store::diesel::{
    models::RoleStateAddressAssociationModel,
    schema::{
        pike_agent_role_assoc, pike_allowed_orgs, pike_inherit_from, pike_permissions, pike_role,
        pike_role_state_address_assoc,
    },
    PikeStoreError,
};

use crate::commits::MAX_COMMIT_NUM;
use diesel::{dsl::update, prelude::*, result::Error as dsl_error};

pub(in crate::pike) trait PikeStoreDeleteRoleOperation {
    fn delete_role(&self, address: &str, current_commit_num: i64) -> Result<(), PikeStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PikeStoreDeleteRoleOperation for PikeStoreOperations<'a, diesel::pg::PgConnection> {
    fn delete_role(&self, address: &str, current_commit_num: i64) -> Result<(), PikeStoreError> {
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let query = pike_role_state_address_assoc::table.into_boxed().filter(
                pike_role_state_address_assoc::state_address
                    .eq(address)
                    .and(pike_role_state_address_assoc::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

            let duplicate_role = query
                .first::<RoleStateAddressAssociationModel>(self.conn)
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

            if let Some(existing_role) = duplicate_role {
                update(pike_role::table)
                    .filter(
                        pike_role::state_address
                            .eq(&existing_role.state_address)
                            .and(pike_role::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(pike_role::end_commit_num.eq(current_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;

                update(pike_agent_role_assoc::table)
                    .filter(
                        pike_agent_role_assoc::role_name
                            .eq(&existing_role.name)
                            .and(pike_agent_role_assoc::org_id.eq(&existing_role.org_id))
                            .and(pike_agent_role_assoc::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(pike_agent_role_assoc::end_commit_num.eq(current_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;

                update(pike_permissions::table)
                    .filter(
                        pike_permissions::role_name
                            .eq(&existing_role.name)
                            .and(pike_permissions::org_id.eq(&existing_role.org_id))
                            .and(pike_permissions::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(pike_permissions::end_commit_num.eq(current_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;

                update(pike_inherit_from::table)
                    .filter(
                        pike_inherit_from::role_name
                            .eq(&existing_role.name)
                            .and(pike_inherit_from::org_id.eq(&existing_role.org_id))
                            .and(pike_inherit_from::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(pike_inherit_from::end_commit_num.eq(current_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;

                update(pike_inherit_from::table)
                    .filter(
                        pike_inherit_from::role_name
                            .eq(&existing_role.name)
                            .and(pike_inherit_from::inherit_from_org_id.eq(&existing_role.org_id))
                            .and(pike_inherit_from::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(pike_inherit_from::end_commit_num.eq(current_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;

                update(pike_allowed_orgs::table)
                    .filter(
                        pike_allowed_orgs::role_name
                            .eq(&existing_role.name)
                            .and(pike_allowed_orgs::org_id.eq(&existing_role.org_id))
                            .and(pike_allowed_orgs::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(pike_allowed_orgs::end_commit_num.eq(current_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PikeStoreDeleteRoleOperation
    for PikeStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn delete_role(&self, address: &str, current_commit_num: i64) -> Result<(), PikeStoreError> {
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let query = pike_role_state_address_assoc::table.into_boxed().filter(
                pike_role_state_address_assoc::state_address
                    .eq(address)
                    .and(pike_role_state_address_assoc::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

            let duplicate_role = query
                .first::<RoleStateAddressAssociationModel>(self.conn)
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

            if let Some(existing_role) = duplicate_role {
                update(pike_role::table)
                    .filter(
                        pike_role::state_address
                            .eq(&existing_role.state_address)
                            .and(pike_role::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(pike_role::end_commit_num.eq(current_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;

                update(pike_agent_role_assoc::table)
                    .filter(
                        pike_agent_role_assoc::role_name
                            .eq(&existing_role.name)
                            .and(pike_agent_role_assoc::org_id.eq(&existing_role.org_id))
                            .and(pike_agent_role_assoc::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(pike_agent_role_assoc::end_commit_num.eq(current_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;

                update(pike_permissions::table)
                    .filter(
                        pike_permissions::role_name
                            .eq(&existing_role.name)
                            .and(pike_permissions::org_id.eq(&existing_role.org_id))
                            .and(pike_permissions::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(pike_permissions::end_commit_num.eq(current_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;

                update(pike_inherit_from::table)
                    .filter(
                        pike_inherit_from::role_name
                            .eq(&existing_role.name)
                            .and(pike_inherit_from::org_id.eq(&existing_role.org_id))
                            .and(pike_inherit_from::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(pike_inherit_from::end_commit_num.eq(current_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;

                update(pike_inherit_from::table)
                    .filter(
                        pike_inherit_from::role_name
                            .eq(&existing_role.name)
                            .and(pike_inherit_from::inherit_from_org_id.eq(&existing_role.org_id))
                            .and(pike_inherit_from::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(pike_inherit_from::end_commit_num.eq(current_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;

                update(pike_allowed_orgs::table)
                    .filter(
                        pike_allowed_orgs::role_name
                            .eq(&existing_role.name)
                            .and(pike_allowed_orgs::org_id.eq(&existing_role.org_id))
                            .and(pike_allowed_orgs::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .set(pike_allowed_orgs::end_commit_num.eq(current_commit_num))
                    .execute(self.conn)
                    .map(|_| ())
                    .map_err(PikeStoreError::from)?;
            }

            Ok(())
        })
    }
}
