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

pub(super) mod add_associated_agents;
pub(super) mod add_properties;
pub(super) mod add_proposals;
pub(super) mod add_records;
pub(super) mod add_reported_values;
pub(super) mod add_reporters;
pub(super) mod fetch_property_with_data_type;
pub(super) mod fetch_record;
pub(super) mod fetch_reported_value_reporter_to_agent_metadata;
pub(super) mod list_associated_agents;
pub(super) mod list_properties_with_data_type;
pub(super) mod list_proposals;
pub(super) mod list_records;
pub(super) mod list_reported_value_reporter_to_agent_metadata;
pub(super) mod list_reporters;

pub(super) struct TrackAndTraceStoreOperations<'a, C> {
    conn: &'a C,
}

impl<'a, C> TrackAndTraceStoreOperations<'a, C>
where
    C: diesel::Connection,
{
    pub fn new(conn: &'a C) -> Self {
        TrackAndTraceStoreOperations { conn }
    }
}
