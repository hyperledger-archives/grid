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

pub(super) mod add_commit;
pub(super) mod create_db_commit_from_commit_event;
pub(super) mod get_commit_by_commit_num;
pub(super) mod get_current_commit_id;
#[cfg(feature = "commit-store-service-commits")]
pub(super) mod get_current_service_commits;
pub(super) mod get_next_commit_num;
pub(super) mod resolve_fork;

pub(super) struct CommitStoreOperations<'a, C> {
    conn: &'a C,
}

impl<'a, C> CommitStoreOperations<'a, C>
where
    C: diesel::Connection,
{
    pub fn new(conn: &'a C) -> Self {
        CommitStoreOperations { conn }
    }
}
