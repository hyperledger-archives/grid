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
use crate::grid_db::commits::store::diesel::{schema::commit, CommitStoreError};

#[cfg(feature = "diesel")]
use diesel::{dsl::max, prelude::*};

#[cfg(feature = "diesel")]
pub(in crate::grid_db::commits) trait CommitStoreGetNextCommitNumOperation {
    fn get_next_commit_num(&self) -> Result<i64, CommitStoreError>;
}

#[cfg(feature = "diesel")]
impl<'a, C> CommitStoreGetNextCommitNumOperation for CommitStoreOperations<'a, C>
where
    C: diesel::Connection,
    <C as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    <C as diesel::Connection>::Backend: 'static,
    i64: diesel::deserialize::FromSql<diesel::sql_types::BigInt, C::Backend>,
{
    fn get_next_commit_num(&self) -> Result<i64, CommitStoreError> {
        let commit_num = commit::table
            .select(max(commit::commit_num))
            .first(self.conn)
            .map(|option: Option<i64>| match option {
                Some(num) => num + 1,
                None => 0,
            })
            .map_err(|err| CommitStoreError::OperationError {
                context: "Failed to get next commit num".to_string(),
                source: Box::new(err),
            })?;
        Ok(commit_num)
    }
}
