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
use crate::grid_db::commits::store::diesel::{
    models::NewCommitModel, schema::commit, CommitEvent, CommitEventError,
};

#[cfg(feature = "diesel")]
use diesel::{dsl::max, prelude::*};

#[cfg(feature = "diesel")]
pub(in crate::grid_db::commits) trait CommitStoreCreateDbCommitFromCommitEventOperation {
    fn create_db_commit_from_commit_event(
        &self,
        event: &CommitEvent,
    ) -> Result<Option<NewCommitModel>, CommitEventError>;
}

#[cfg(feature = "diesel")]
impl<'a, C> CommitStoreCreateDbCommitFromCommitEventOperation for CommitStoreOperations<'a, C>
where
    C: diesel::Connection,
    <C as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    <C as diesel::Connection>::Backend: 'static,
    i64: diesel::deserialize::FromSql<diesel::sql_types::BigInt, C::Backend>,
    String: diesel::deserialize::FromSql<diesel::sql_types::Text, C::Backend>,
{
    fn create_db_commit_from_commit_event(
        &self,
        event: &CommitEvent,
    ) -> Result<Option<NewCommitModel>, CommitEventError> {
        let commit_id = event.id.clone();
        let commit_num = match event.height {
            Some(height_u64) => height_u64.try_into().map_err(|err| {
                CommitEventError::ConnectionError(format!(
                    "failed to convert event height to i64: {}",
                    err
                ))
            })?,
            None => commit::table
                .select(max(commit::commit_num))
                .first(self.conn)
                .map(|option: Option<i64>| match option {
                    Some(num) => num + 1,
                    None => 0,
                })
                .map_err(|err| CommitEventError::OperationError {
                    context: "Failed to get next commit num".to_string(),
                    source: Box::new(err),
                })?,
        };
        let service_id = event.service_id.clone();
        Ok(Some(NewCommitModel {
            commit_id,
            commit_num,
            service_id,
        }))
    }
}
