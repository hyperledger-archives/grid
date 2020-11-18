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
use crate::grid_db::commits::store::diesel::{
    models::CommitModel, schema::commit, CommitStoreError,
};

use diesel::{prelude::*, result::Error::NotFound};

pub(in crate::grid_db::commits) trait CommitStoreGetCurrentCommitIdOperation {
    fn get_current_commit_id(&self) -> Result<Option<String>, CommitStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> CommitStoreGetCurrentCommitIdOperation
    for CommitStoreOperations<'a, diesel::pg::PgConnection>
{
    fn get_current_commit_id(&self) -> Result<Option<String>, CommitStoreError> {
        commit::table
            .select(commit::all_columns)
            .order_by(commit::commit_num.desc())
            .limit(1)
            .first::<CommitModel>(self.conn)
            .map(|commit| Some(commit.commit_id))
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                CommitStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> CommitStoreGetCurrentCommitIdOperation
    for CommitStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn get_current_commit_id(&self) -> Result<Option<String>, CommitStoreError> {
        commit::table
            .select(commit::all_columns)
            .order_by(commit::commit_num.desc())
            .limit(1)
            .first::<CommitModel>(self.conn)
            .map(|commit| Some(commit.commit_id))
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
            .map_err(|err| {
                CommitStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })
    }
}
