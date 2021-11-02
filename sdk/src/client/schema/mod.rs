// Copyright 2021 Cargill Incorporated
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

use crate::error::ClientError;
use crate::protocol::schema::state::DataType as StateDataType;

use super::Client;

#[cfg(feature = "client-reqwest")]
pub mod reqwest;

pub struct Schema {
    pub name: String,
    pub description: String,
    pub owner: String,
    pub properties: Vec<PropertyDefinition>,
}

pub struct PropertyDefinition {
    pub name: String,
    pub schema_name: String,
    pub data_type: DataType,
    pub required: bool,
    pub description: String,
    pub number_exponent: i64,
    pub enum_options: Vec<String>,
    pub struct_properties: Vec<PropertyDefinition>,
}

#[derive(Debug)]
pub enum DataType {
    Bytes,
    Boolean,
    Number,
    String,
    Enum,
    Struct,
    LatLong,
}

impl From<DataType> for StateDataType {
    fn from(data_type: DataType) -> Self {
        match data_type {
            DataType::Bytes => StateDataType::Bytes,
            DataType::Boolean => StateDataType::Boolean,
            DataType::Number => StateDataType::Number,
            DataType::String => StateDataType::String,
            DataType::Enum => StateDataType::Enum,
            DataType::Struct => StateDataType::Struct,
            DataType::LatLong => StateDataType::LatLong,
        }
    }
}

pub trait SchemaClient: Client {
    /// Fetches a single schema based on name
    ///
    /// # Arguments
    ///
    /// * `name` - the name of the schema (identifier)
    /// * `service_id` - optional - the service ID to fetch the schema from
    fn get_schema(&self, name: String, service_id: Option<&str>) -> Result<Schema, ClientError>;

    /// Fetches a list of schemas for the organization
    ///
    /// # Arguments
    ///
    /// * `service_id` - optional - the service ID to fetch the schemas from
    fn list_schemas(&self, service_id: Option<&str>) -> Result<Vec<Schema>, ClientError>;
}
