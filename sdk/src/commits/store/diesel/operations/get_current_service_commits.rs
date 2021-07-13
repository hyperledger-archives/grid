// Copyright 2018-2021 Cargill Incorporated
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

use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Nullable, Text};

use crate::commits::store::diesel::{Commit, CommitStoreError};
use crate::error::InternalError;

use super::CommitStoreOperations;

#[derive(QueryableByName)]
struct CurrentCommit {
    #[column_name = "commit_id"]
    #[sql_type = "Text"]
    pub commit_id: String,

    #[column_name = "current_commit_number"]
    #[sql_type = "BigInt"]
    pub commit_num: i64,

    #[column_name = "service_id"]
    #[sql_type = "Nullable<Text>"]
    pub service_id: Option<String>,
}

/// Performs the operation to return the current commits recorded for services. It ignores any
/// commits that have no service id (i.e. Sawtooth commits).
pub(in crate::commits) trait CommitStoreGetCurrentSericeCommitsOperation {
    fn get_current_service_commits(&self) -> Result<Vec<Commit>, CommitStoreError>;
}

impl<'a, C> CommitStoreGetCurrentSericeCommitsOperation for CommitStoreOperations<'a, C>
where
    C: diesel::Connection,
    i64: diesel::deserialize::FromSql<diesel::sql_types::BigInt, C::Backend>,
    String: diesel::deserialize::FromSql<diesel::sql_types::Text, C::Backend>,
{
    fn get_current_service_commits(&self) -> Result<Vec<Commit>, CommitStoreError> {
        // A raw query is required, as diesel does not support this style of query as of its 1.4.x
        // release branch.
        sql_query(
            r#"
            WITH current_commits AS
            (
                SELECT service_id, max(commit_num) AS current_commit_number
                FROM commits
                GROUP BY service_id
            )
            SELECT commits.commit_id,
                   commits.commit_num AS current_commit_number,
                   commits.service_id
            FROM current_commits
            INNER JOIN commits
            ON  commits.service_id = current_commits.service_id
            AND commits.commit_num = current_commits.current_commit_number
        "#,
        )
        .load::<CurrentCommit>(self.conn)
        .map(|values| {
            values
                .into_iter()
                .filter_map(
                    |CurrentCommit {
                         commit_id,
                         commit_num,
                         service_id,
                     }| {
                        service_id.map(|service_id| Commit {
                            commit_id,
                            commit_num,
                            service_id: Some(service_id),
                        })
                    },
                )
                .collect::<Vec<_>>()
        })
        .map_err(|err| CommitStoreError::InternalError(InternalError::from_source(Box::new(err))))
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;

    use diesel::insert_into;

    use crate::commits::store::diesel::{models::NewCommitModel, schema::commits};
    use crate::migrations::run_sqlite_migrations;

    #[test]
    fn test_get_current_service_commits() -> Result<(), Box<dyn std::error::Error>> {
        let conn = SqliteConnection::establish(":memory:")?;

        run_sqlite_migrations(&conn)?;

        insert_into(commits::table)
            .values(vec![
                NewCommitModel {
                    commit_id: "first:sawtooth".into(),
                    commit_num: 1,
                    service_id: None,
                },
                // service 1
                NewCommitModel {
                    commit_id: "first:service1".into(),
                    commit_num: 1,
                    service_id: Some("service1".into()),
                },
                NewCommitModel {
                    commit_id: "second:service1".into(),
                    commit_num: 2,
                    service_id: Some("service1".into()),
                },
                // Service 2
                NewCommitModel {
                    commit_id: "first:service2".into(),
                    commit_num: 1,
                    service_id: Some("service2".into()),
                },
            ])
            .execute(&conn)?;

        let ops = CommitStoreOperations::new(&conn);
        let current_service_commits = ops.get_current_service_commits()?;

        assert_eq!(
            vec![
                Commit {
                    commit_id: "second:service1".into(),
                    commit_num: 2,
                    service_id: Some("service1".into()),
                },
                Commit {
                    commit_id: "first:service2".into(),
                    commit_num: 1,
                    service_id: Some("service2".into()),
                },
            ],
            current_service_commits,
        );

        Ok(())
    }
}
