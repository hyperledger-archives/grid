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

pub(super) mod add_agent;
pub(super) mod add_organizations;
pub(super) mod fetch_agent;
pub(super) mod fetch_organization;
pub(super) mod list_agents;
pub(super) mod list_organizations;
pub(super) mod update_agent;

pub(super) struct PikeStoreOperations<'a, C> {
    conn: &'a C,
}

impl<'a, C> PikeStoreOperations<'a, C>
where
    C: diesel::Connection,
{
    pub fn new(conn: &'a C) -> Self {
        PikeStoreOperations { conn }
    }
}
