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

use super::CommitStoreOperations;
use crate::error::InternalError;
use crate::grid_db::commits::store::diesel::{schema::commit, CommitStoreError};

use diesel::{dsl::max, prelude::*};

pub(in crate::grid_db::commits) trait CommitStoreGetNextCommitNumOperation {
    fn get_next_commit_num(&self) -> Result<i64, CommitStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> CommitStoreGetNextCommitNumOperation
    for CommitStoreOperations<'a, diesel::pg::PgConnection>
{
    fn get_next_commit_num(&self) -> Result<i64, CommitStoreError> {
        let commit_num = commit::table
            .select(max(commit::commit_num))
            .first(self.conn)
            .map(|option: Option<i64>| match option {
                Some(num) => num + 1,
                None => 0,
            })
            .map_err(|err| {
                CommitStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;
        Ok(commit_num)
    }
}

#[cfg(feature = "sqlite")]
impl<'a> CommitStoreGetNextCommitNumOperation
    for CommitStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn get_next_commit_num(&self) -> Result<i64, CommitStoreError> {
        let commit_num = commit::table
            .select(max(commit::commit_num))
            .first(self.conn)
            .map(|option: Option<i64>| match option {
                Some(num) => num + 1,
                None => 0,
            })
            .map_err(|err| {
                CommitStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;
        Ok(commit_num)
    }
}
