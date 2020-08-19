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
use crate::grid_db::commits::store::diesel::{schema::chain_record, schema::commit};
use crate::grid_db::commits::MAX_COMMIT_NUM;

use crate::grid_db::commits::store::CommitStoreError;

#[cfg(feature = "diesel")]
use diesel::{
    dsl::{delete, update},
    prelude::*,
};

#[cfg(feature = "diesel")]
pub(in crate::grid_db::commits) trait CommitStoreResolveForkOperation {
    fn resolve_fork(&self, commit_num: i64) -> Result<(), CommitStoreError>;
}

#[cfg(feature = "diesel")]
impl<'a, C> CommitStoreResolveForkOperation for CommitStoreOperations<'a, C>
where
    C: diesel::Connection,
    <C as diesel::Connection>::Backend: diesel::backend::SupportsDefaultKeyword,
    <C as diesel::Connection>::Backend: 'static,
    i64: diesel::deserialize::FromSql<diesel::sql_types::BigInt, C::Backend>,
{
    fn resolve_fork(&self, commit_num: i64) -> Result<(), CommitStoreError> {
        delete(chain_record::table)
            .filter(chain_record::start_commit_num.ge(commit_num))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|err| CommitStoreError::OperationError {
                context: "Failed to resolve fork".to_string(),
                source: Box::new(err),
            })?;

        update(chain_record::table)
            .filter(chain_record::end_commit_num.ge(commit_num))
            .set(chain_record::end_commit_num.eq(MAX_COMMIT_NUM))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|err| CommitStoreError::OperationError {
                context: "Failed to resolve fork".to_string(),
                source: Box::new(err),
            })?;

        delete(commit::table)
            .filter(commit::commit_num.ge(commit_num))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|err| CommitStoreError::OperationError {
                context: "Failed to resolve fork".to_string(),
                source: Box::new(err),
            })?;

        Ok(())
    }
}
