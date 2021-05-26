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
use crate::commits::store::diesel::models::{CommitModel, NewCommitModel};
use crate::commits::store::diesel::{schema::commits, CommitStoreError};
use crate::error::{ConstraintViolationError, ConstraintViolationType, InternalError};

use diesel::{dsl::insert_into, prelude::*};

pub(in crate::commits) trait CommitStoreAddCommitOperation {
    fn add_commit(&self, commit: NewCommitModel) -> Result<(), CommitStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> CommitStoreAddCommitOperation for CommitStoreOperations<'a, diesel::pg::PgConnection> {
    fn add_commit(&self, commit: NewCommitModel) -> Result<(), CommitStoreError> {
        self.conn.transaction::<_, CommitStoreError, _>(|| {
            let duplicate_commit = commits::table
                .filter(commits::commit_id.eq(&commit.commit_id))
                .first::<CommitModel>(self.conn)
                .map(Some)
                .optional()
                .map_err(|err| {
                    CommitStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;
            if duplicate_commit.is_some() {
                return Err(CommitStoreError::ConstraintViolationError(
                    ConstraintViolationError::with_violation_type(ConstraintViolationType::Unique),
                ));
            }

            insert_into(commits::table)
                .values(commit)
                .execute(self.conn)
                .map(|_| ())?;
            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> CommitStoreAddCommitOperation
    for CommitStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn add_commit(&self, commit: NewCommitModel) -> Result<(), CommitStoreError> {
        self.conn.transaction::<_, CommitStoreError, _>(|| {
            let duplicate_commit = commits::table
                .filter(commits::commit_id.eq(&commit.commit_id))
                .first::<CommitModel>(self.conn)
                .map(Some)
                .optional()
                .map_err(|err| {
                    CommitStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;
            if duplicate_commit.is_some() {
                return Err(CommitStoreError::ConstraintViolationError(
                    ConstraintViolationError::with_violation_type(ConstraintViolationType::Unique),
                ));
            }

            insert_into(commits::table)
                .values(commit)
                .execute(self.conn)
                .map(|_| ())?;
            Ok(())
        })
    }
}
