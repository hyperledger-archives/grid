/*
 * Copyright 2019 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use byteorder::{NetworkEndian, ReadBytesExt};
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql, WriteTuple};
use diesel::sql_types;
use serde_json::Value as JsonValue;
use std::io::Write;

use super::schema::{
    agent, block, grid_property_definition, grid_property_value, grid_schema, organization,
};

#[derive(Insertable, Queryable)]
#[table_name = "block"]
pub struct Block {
    pub block_id: String,
    pub block_num: i64,
    pub state_root_hash: String,
}

#[derive(Insertable, Debug)]
#[table_name = "agent"]
pub struct NewAgent {
    pub public_key: String,
    pub org_id: String,
    pub active: bool,
    pub roles: Vec<String>,
    pub metadata: Vec<JsonValue>,

    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_block_num: i64,
    pub end_block_num: i64,
}

#[derive(Queryable, Debug)]
pub struct Agent {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub public_key: String,
    pub org_id: String,
    pub active: bool,
    pub roles: Vec<String>,
    pub metadata: Vec<JsonValue>,

    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_block_num: i64,
    pub end_block_num: i64,
}

#[derive(Insertable, Debug)]
#[table_name = "organization"]
pub struct NewOrganization {
    pub org_id: String,
    pub name: String,
    pub address: String,
    pub metadata: Vec<JsonValue>,

    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_block_num: i64,
    pub end_block_num: i64,
}

#[derive(Queryable, Debug)]
pub struct Organization {
    ///  This is the record id for the slowly-changing-dimensions table.
    pub id: i64,
    pub org_id: String,
    pub name: String,
    pub address: String,
    pub metadata: Vec<JsonValue>,

    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_block_num: i64,
    pub end_block_num: i64,
}

#[derive(Clone, Insertable, Debug)]
#[table_name = "grid_schema"]
pub struct NewGridSchema {
    pub start_block_num: i64,
    pub end_block_num: i64,
    pub name: String,
    pub description: String,
    pub owner: String,
}

#[allow(dead_code)]
#[derive(Queryable, Debug)]
pub struct GridSchema {
    pub id: i64,
    pub start_block_num: i64,
    pub end_block_num: i64,
    pub name: String,
    pub description: String,
    pub owner: String,
}

#[derive(Clone, Insertable, Debug)]
#[table_name = "grid_property_definition"]
pub struct NewGridPropertyDefinition {
    pub start_block_num: i64,
    pub end_block_num: i64,
    pub name: String,
    pub schema_name: String,
    pub data_type: String,
    pub required: bool,
    pub description: String,
    pub number_exponent: i64,
    pub enum_options: Vec<String>,
    pub struct_properties: Vec<String>,
}

#[allow(dead_code)]
#[derive(Queryable, Debug)]
pub struct GridPropertyDefinition {
    pub id: i64,
    pub start_block_num: i64,
    pub end_block_num: i64,
    pub name: String,
    pub schema_name: String,
    pub data_type: String,
    pub required: bool,
    pub description: String,
    pub number_exponent: i64,
    pub enum_options: Vec<String>,
    pub struct_properties: Vec<String>,
}

#[derive(Insertable, Debug)]
#[table_name = "grid_property_value"]
pub struct NewGridPropertyValue {
    pub start_block_num: i64,
    pub end_block_num: i64,
    pub name: String,
    pub data_type: String,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub struct_values: Option<Vec<String>>,
    pub lat_long_value: Option<LatLongValue>,
}

#[allow(dead_code)]
#[derive(Queryable, Debug)]
pub struct GridPropertyValue {
    pub id: i64,
    pub start_block_num: i64,
    pub end_block_num: i64,
    pub name: String,
    pub data_type: String,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub struct_values: Option<Vec<String>>,
    pub lat_long_value: Option<LatLongValue>,
}

#[derive(SqlType, QueryId, Debug, Clone, Copy)]
#[postgres(type_name = "latlong")]
pub struct LatLong;

#[derive(Debug, PartialEq, FromSqlRow, AsExpression)]
#[sql_type = "LatLong"]
pub struct LatLongValue(pub i64, pub i64);

impl ToSql<LatLong, Pg> for LatLongValue {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        WriteTuple::<(sql_types::BigInt, sql_types::BigInt)>::write_tuple(&(self.0, self.1), out)
    }
}

impl FromSql<LatLong, Pg> for LatLongValue {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let mut bytes = not_none!(bytes);
        let num_elements = bytes.read_i32::<NetworkEndian>()?;
        if num_elements != 2 {
            return Err(format!("Expected a tuple of 2 elements, got {}", num_elements,).into());
        }
        let (_, mut bytes) = bytes.split_at(std::mem::size_of::<i32>() * 2);
        let lat = bytes.read_i64::<NetworkEndian>()?;
        let (_, mut bytes) = bytes.split_at(std::mem::size_of::<i32>() * 2);
        let long = bytes.read_i64::<NetworkEndian>()?;
        Ok(LatLongValue(lat, long))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserializing_lat_long_value_from_bytes() {
        let lat_long_bytes = [
            0, 0, 0, 2, 0, 0, 0, 20, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 8,
            255, 255, 255, 255, 255, 255, 255, 255,
        ];
        let lat_long = LatLongValue::from_sql(Some(&lat_long_bytes))
            .expect("Failed to deserialize LatLongValue");

        assert_eq!(lat_long, LatLongValue(0, -1));
    }
}
