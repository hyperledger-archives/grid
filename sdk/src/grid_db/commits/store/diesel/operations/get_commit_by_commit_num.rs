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
use crate::grid_db::commits::store::diesel::{
    models::CommitModel, schema::commit, Commit, CommitStoreError,
};

#[cfg(feature = "diesel")]
use diesel::{prelude::*, result::Error::NotFound, sql_types::Text};

#[cfg(feature = "diesel")]
pub(in crate::grid_db::commits) trait CommitStoreGetCommitByCommitNumOperation {
    fn get_commit_by_commit_num(&self, commit_num: i64)
        -> Result<Option<Commit>, CommitStoreError>;
}

#[cfg(feature = "diesel")]
impl<'a, C> CommitStoreGetCommitByCommitNumOperation for CommitStoreOperations<'a, C>
where
    C: diesel::Connection,
    <C as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    <C as diesel::Connection>::Backend: 'static,
    i64: diesel::deserialize::FromSql<diesel::sql_types::BigInt, C::Backend>,
    String: diesel::deserialize::FromSql<diesel::sql_types::Text, C::Backend>,
    Option<String>: diesel::deserialize::FromSql<diesel::sql_types::Nullable<Text>, C::Backend>,
{
    fn get_commit_by_commit_num(
        &self,
        commit_num: i64,
    ) -> Result<Option<Commit>, CommitStoreError> {
        commit::table
            .select(commit::all_columns)
            .filter(commit::commit_num.eq(&commit_num))
            .first::<CommitModel>(self.conn)
            .map(|commit| Some(Commit::from(commit)))
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| CommitStoreError::OperationError {
                context: "Failed to fetch commit".to_string(),
                source: Box::new(err),
            })
    }
}
