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

mod location;
mod pike;
mod product;
mod schema;

use crate::protos::{IntoBytes, ProtoConversionError};

pub use location::*;
pub use pike::*;
pub use product::*;
pub use schema::*;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SubmitBatchRequest {
    #[serde(default)]
    pub circuit_id: Option<String>,
    #[serde(default)]
    pub service_id: Option<String>,
    pub batches: Vec<Batch>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Batch {
    pub transactions: Vec<Transaction>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Transaction {
    pub family_name: String,
    pub version: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<String>,
    pub payload: Payload,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Payload {
    Pike(PikePayload),
    Product(ProductPayload),
    Location(LocationPayload),
    Schema(SchemaPayload),
}

impl IntoBytes for Payload {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        match self {
            Payload::Pike(payload) => payload.into_bytes(),
            Payload::Product(payload) => payload.into_bytes(),
            Payload::Location(payload) => payload.into_bytes(),
            Payload::Schema(payload) => payload.into_bytes(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SubmitBatchResponse {
    ids: Vec<String>,
    message: String,
}

impl SubmitBatchResponse {
    pub fn new(ids: Vec<String>) -> Self {
        Self {
            ids,
            message: "Batches submitted successfully".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum BuilderError {
    MissingField(String),
    EmptyVec(String),
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            BuilderError::MissingField(ref s) => write!(f, "MissingField: {}", s),
            BuilderError::EmptyVec(ref s) => write!(f, "EmptyVec: {}", s),
        }
    }
}
