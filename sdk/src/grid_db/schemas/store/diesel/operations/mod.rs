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

pub(super) mod add_schema;
pub(super) mod fetch_schema;
pub(super) mod get_property_definition_by_name;
pub(super) mod list_property_definitions;
pub(super) mod list_property_definitions_with_schema_name;
pub(super) mod list_schemas;

pub(super) struct SchemaStoreOperations<'a, C> {
    conn: &'a C,
}

impl<'a, C> SchemaStoreOperations<'a, C>
where
    C: diesel::Connection,
{
    pub fn new(conn: &'a C) -> Self {
        SchemaStoreOperations { conn }
    }
}
