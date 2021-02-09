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

#[cfg(feature = "diesel")]
pub mod diesel;
mod error;

use std::fmt;

use crate::hex;
use crate::paging::Paging;

pub use error::BatchStoreError;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum BatchStatus {
    Committed,
    Submitted,
    NotSubmitted,
    Rejected,
    Unknown,
}

impl fmt::Display for BatchStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Batch {
    pub id: String,
    pub data: String,
    pub status: BatchStatus,
}

impl Batch {
    pub fn from_bytes(id: &str, bytes: &[u8]) -> Self {
        Batch {
            id: id.to_string(),
            data: hex::to_hex(bytes),
            status: BatchStatus::NotSubmitted,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Clone, Debug)]
pub struct BatchList {
    data: Vec<Batch>,
    paging: Paging,
}

impl BatchList {
    fn new(data: Vec<Batch>, paging: Paging) -> Self {
        Self { data, paging }
    }
}

pub trait BatchStore: Send + Sync {
    fn add_batch(&self, batch: Batch) -> Result<(), BatchStoreError>;

    fn fetch_batch(&self, id: &str) -> Result<Option<Batch>, BatchStoreError>;

    fn list_batches(&self, offset: i64, limit: i64) -> Result<BatchList, BatchStoreError>;

    fn list_batches_with_status(
        &self,
        status: BatchStatus,
        offset: i64,
        limit: i64,
    ) -> Result<BatchList, BatchStoreError>;

    fn update_status(&self, id: &str, status: BatchStatus) -> Result<(), BatchStoreError>;
}
