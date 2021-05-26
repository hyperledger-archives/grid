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
use crate::commits::store::diesel::{schema::chain_record, schema::commits};
use crate::commits::store::CommitStoreError;
use crate::commits::MAX_COMMIT_NUM;

use diesel::{
    dsl::{delete, update},
    prelude::*,
};

/// This database operation takes one argument: `commit_num`. This argument is
/// the height of the commit that will be replaced. The commit is only replaced
/// if it is not a duplicate.
///
/// If a fork is detected when attempting to add a commit, and the existing
/// commit in the store is not a duplicate of the commit that is being added,
/// the existing commit and anything with a `start_commit_num` greater than
/// the height of the commit being added is removed. Then, the end_commit_num`
/// for any record that is greater than the height of the commit being added is
/// set to `MAX_COMMIT_NUM`, meaning it is the current record. Finally, the
/// commit being replaced is deleted. This allows for the database handler to
/// call the operation to insert the record at the correct height.

pub(in crate::commits) trait CommitStoreResolveForkOperation {
    fn resolve_fork(&self, commit_num: i64) -> Result<(), CommitStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> CommitStoreResolveForkOperation for CommitStoreOperations<'a, diesel::pg::PgConnection> {
    fn resolve_fork(&self, commit_num: i64) -> Result<(), CommitStoreError> {
        self.conn.transaction::<_, CommitStoreError, _>(|| {
            delete(chain_record::table)
                .filter(chain_record::start_commit_num.ge(commit_num))
                .execute(self.conn)
                .map(|_| ())?;

            update(chain_record::table)
                .filter(chain_record::end_commit_num.ge(commit_num))
                .set(chain_record::end_commit_num.eq(MAX_COMMIT_NUM))
                .execute(self.conn)
                .map(|_| ())?;

            delete(commits::table)
                .filter(commits::commit_num.ge(commit_num))
                .execute(self.conn)
                .map(|_| ())?;

            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> CommitStoreResolveForkOperation
    for CommitStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn resolve_fork(&self, commit_num: i64) -> Result<(), CommitStoreError> {
        self.conn.transaction::<_, CommitStoreError, _>(|| {
            delete(chain_record::table)
                .filter(chain_record::start_commit_num.ge(commit_num))
                .execute(self.conn)
                .map(|_| ())?;

            update(chain_record::table)
                .filter(chain_record::end_commit_num.ge(commit_num))
                .set(chain_record::end_commit_num.eq(MAX_COMMIT_NUM))
                .execute(self.conn)
                .map(|_| ())?;

            delete(commits::table)
                .filter(commits::commit_num.ge(commit_num))
                .execute(self.conn)
                .map(|_| ())?;

            Ok(())
        })
    }
}
