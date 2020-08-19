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

use crate::grid_db::commits::store::diesel::models::{CommitModel, NewCommitModel};
#[cfg(feature = "diesel")]
use diesel::{dsl::insert_into, prelude::*, result::Error::NotFound};

#[cfg(feature = "diesel")]
pub(in crate::grid_db::commits) trait CommitStoreAddCommitOperation {
    fn add_commit(&self, commit: &NewCommitModel) -> Result<(), CommitStoreError>;
}

#[cfg(feature = "diesel")]
impl<'a, C> CommitStoreAddCommitOperation for CommitStoreOperations<'a, C>
where
    C: diesel::Connection,
    <C as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    <C as diesel::Connection>::Backend: 'static,
    i64: diesel::deserialize::FromSql<diesel::sql_types::BigInt, C::Backend>,
    String: diesel::deserialize::FromSql<diesel::sql_types::Text, C::Backend>,
{
    fn add_commit(&self, commit: &NewCommitModel) -> Result<(), CommitStoreError> {
        let duplicate_commit = commit::table
            .filter(commit::commit_id.eq(&commit.commit_id))
            .first::<CommitModel>(self.conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| CommitStoreError::QueryError {
                context: "Failed check for existing commit".to_string(),
                source: Box::new(err),
            })?;
        if duplicate_commit.is_some() {
            return Err(CommitStoreError::DuplicateError {
                context: "Commit already exists".to_string(),
                source: None,
            });
        }

        insert_into(commit::table)
            .values(commit)
            .execute(self.conn)
            .map(|_| ())
            .map_err(|err| CommitStoreError::OperationError {
                context: "Failed to add commit".to_string(),
                source: Box::new(err),
            })?;
        Ok(())
    }
}
