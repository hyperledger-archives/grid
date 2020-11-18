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

use std::convert::TryInto;

use super::CommitStoreOperations;
use crate::error::InternalError;
use crate::grid_db::commits::store::diesel::{
    schema::commit, Commit, CommitEvent, CommitEventError,
};

use diesel::{dsl::max, prelude::*};

pub(in crate::grid_db::commits) trait CommitStoreCreateDbCommitFromCommitEventOperation {
    fn create_db_commit_from_commit_event(
        &self,
        event: &CommitEvent,
    ) -> Result<Option<Commit>, CommitEventError>;
}

#[cfg(feature = "postgres")]
impl<'a> CommitStoreCreateDbCommitFromCommitEventOperation
    for CommitStoreOperations<'a, diesel::pg::PgConnection>
{
    fn create_db_commit_from_commit_event(
        &self,
        event: &CommitEvent,
    ) -> Result<Option<Commit>, CommitEventError> {
        let commit_id = event.id.clone();
        let commit_num = match event.height {
            Some(height_u64) => height_u64.try_into().map_err(|err| {
                CommitEventError::InternalError(InternalError::from_source(Box::new(err)))
            })?,
            None => commit::table
                .select(max(commit::commit_num))
                .first(self.conn)
                .map(|option: Option<i64>| match option {
                    Some(num) => num + 1,
                    None => 0,
                })
                .map_err(|err| {
                    CommitEventError::InternalError(InternalError::from_source(Box::new(err)))
                })?,
        };
        let service_id = event.service_id.clone();
        Ok(Some(Commit {
            commit_id,
            commit_num,
            service_id,
        }))
    }
}

#[cfg(feature = "sqlite")]
impl<'a> CommitStoreCreateDbCommitFromCommitEventOperation
    for CommitStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn create_db_commit_from_commit_event(
        &self,
        event: &CommitEvent,
    ) -> Result<Option<Commit>, CommitEventError> {
        let commit_id = event.id.clone();
        let commit_num = match event.height {
            Some(height_u64) => height_u64.try_into().map_err(|err| {
                CommitEventError::InternalError(InternalError::from_source(Box::new(err)))
            })?,
            None => commit::table
                .select(max(commit::commit_num))
                .first(self.conn)
                .map(|option: Option<i64>| match option {
                    Some(num) => num + 1,
                    None => 0,
                })
                .map_err(|err| {
                    CommitEventError::InternalError(InternalError::from_source(Box::new(err)))
                })?,
        };
        let service_id = event.service_id.clone();
        Ok(Some(Commit {
            commit_id,
            commit_num,
            service_id,
        }))
    }
}
