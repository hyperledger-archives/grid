// Copyright 2022 Cargill Incorporated
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

pub mod event;
pub mod monitor;

use futures::future::BoxFuture;
use std::fmt::Debug;

pub type BatchResult<T> = Result<T, BatchError>;

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum BatchError {
    // InternalError uses a String here because the overarching
    // enum must be Send
    InternalError(String),
}

/// BatchStatus represents the minimum batch status information
/// necessary for the polling monitor to run
pub trait BatchStatus: Debug + Clone {
    fn get_id(&self) -> &str;
    fn is_unknown(&self) -> bool;
}

/// BatchId represents the minimum batch id information
/// necessary for the polling monitor to run
pub trait BatchId: Debug + Clone + Sync + Send {
    fn get_id(&self) -> &str;
    fn get_service_id(&self) -> &str;
}

/// Store that allows getting a list of pending batch ids
pub trait PendingBatchStore: Send {
    type Id: BatchId;
    fn get_pending_batch_ids(&self, limit: usize) -> BatchResult<Vec<Self::Id>>;
}

/// Reads the batch statuses from an external source
pub trait BatchStatusReader: Send {
    type Status: BatchStatus;

    fn get_batch_statuses<'a>(
        &'a self,
        service_id: &'a str,
        batch_ids: &'a [String],
    ) -> BoxFuture<'a, BatchResult<Vec<Self::Status>>>;

    fn available_connections(&self) -> usize;
}

/// Updates the batch statuses
pub trait BatchUpdater: Send {
    type Status: BatchStatus;

    fn update_batch_statuses(&self, service_id: &str, batches: &[Self::Status]) -> BatchResult<()>;
}
