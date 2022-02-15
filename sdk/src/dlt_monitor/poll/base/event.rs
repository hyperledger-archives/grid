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

use std::fmt::{Display, Formatter, Result as DisplayResult};
use std::time::Duration;

use super::{BatchError, BatchId};

#[derive(Debug)]
pub enum Event<T: BatchId> {
    FetchPending,
    FetchPendingComplete(Duration, Vec<T>),
    FetchStatusesComplete(String, usize, Duration),
    UpdateComplete(String, usize, Duration),
    Waiting(Duration),
    Error(BatchError),
}

impl<T: BatchId> Display for Event<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> DisplayResult {
        match self {
            Event::FetchPending => write!(f, "fetching pending batches"),
            Event::FetchPendingComplete(duration, statuses) => write!(
                f,
                "found {batches} pending batches ({duration:?})",
                batches = statuses.len()
            ),
            Event::FetchStatusesComplete(service_id, total, duration) => {
                write!(
                    f,
                    "service {service_id} fetched {total} batch statuses ({duration:?})"
                )
            }
            Event::UpdateComplete(service_id, total, duration) => {
                write!(
                    f,
                    "service {service_id} updated {total} batch statuses ({duration:?})"
                )
            }
            Event::Waiting(frequency) => {
                write!(f, "waiting at interval {frequency:?}")
            }
            Event::Error(err) => write!(f, "Error: {err:?}"),
        }
    }
}
