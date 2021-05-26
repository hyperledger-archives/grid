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
