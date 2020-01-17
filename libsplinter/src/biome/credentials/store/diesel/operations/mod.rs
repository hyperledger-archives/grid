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

pub(super) mod add_credentials;
pub(super) mod fetch_credential_by_id;
pub(super) mod fetch_credential_by_username;
pub(super) mod fetch_username;
pub(super) mod get_usernames;
pub(super) mod update_credentials;

pub(super) struct CredentialsStoreOperations<'a, C> {
    conn: &'a C,
}

impl<'a, C> CredentialsStoreOperations<'a, C>
where
    C: diesel::Connection,
{
    pub fn new(conn: &'a C) -> Self {
        CredentialsStoreOperations { conn }
    }
}
