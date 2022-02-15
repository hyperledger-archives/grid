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
pub mod stream;

pub use monitor::create_polling_monitor;

use futures::future::BoxFuture;
use std::fmt::Debug;

pub type BatchResult<T> = Result<T, BatchError>;

// BatchError must be Send
#[derive(Debug, Clone)]
pub enum BatchError {
    InternalError(String),
}

pub trait BatchStatus: Debug {
    fn get_id(&self) -> &str;
    fn is_unknown(&self) -> bool;
}

pub trait BatchId: Debug + Clone {
    fn get_id(&self) -> &str;
    fn get_service_id(&self) -> &str;
}

pub trait PendingBatchStore<T: BatchId> {
    fn get_pending_batch_ids(&self, limit: usize) -> BatchResult<Vec<T>>;
}

pub trait BatchStatusReader<T: BatchStatus> {
    fn get_batch_statuses<'a>(
        &'a self,
        service_id: &'a str,
        batch_ids: &'a [String],
    ) -> BoxFuture<'a, BatchResult<Vec<T>>>;

    fn available_connections(&self) -> usize;
}

pub trait BatchUpdater<T: BatchStatus> {
    fn update_batch_statuses(&self, service_id: &str, batches: &[T]) -> BatchResult<()>;
}
