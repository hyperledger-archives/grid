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

use super::{BatchError, BatchId, BatchStatus};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event<Id: BatchId, Status: BatchStatus> {
    FetchPending,
    FetchPendingComplete {
        ids: Vec<Id>,
    },
    FetchStatuses {
        service_id: String,
        batches: Vec<String>,
    },
    FetchStatusesComplete {
        service_id: String,
        batches: Vec<String>,
        statuses: Vec<Status>,
    },
    Update {
        service_id: String,
        statuses: Vec<Status>,
    },
    UpdateComplete {
        service_id: String,
        statuses: Vec<Status>,
    },
    Waiting(Duration),
    Error(BatchError),
}

impl<Id: BatchId, Status: BatchStatus> Display for Event<Id, Status> {
    fn fmt(&self, f: &mut Formatter<'_>) -> DisplayResult {
        match self {
            Event::FetchPending => write!(f, "fetching pending batches"),
            Event::FetchPendingComplete { ids } => {
                write!(f, "found {batches} pending batches", batches = ids.len())
            }
            Event::FetchStatuses {
                service_id,
                batches,
            } => {
                write!(
                    f,
                    "fetching {service_id} batch statuses for [{batches}]",
                    batches = batches.join(", ")
                )
            }
            Event::FetchStatusesComplete {
                service_id,
                batches,
                statuses,
            } => {
                write!(
                    f,
                    "service {service_id} fetched {total_statuses} batch statuses \
                        for {total_batches} batches",
                    total_statuses = statuses.len(),
                    total_batches = batches.len()
                )
            }
            Event::Update {
                service_id,
                statuses,
            } => {
                write!(
                    f,
                    "service {service_id} updating {total} batch statuses",
                    total = statuses.len()
                )
            }
            Event::UpdateComplete {
                service_id,
                statuses,
            } => {
                write!(
                    f,
                    "service {service_id} updated {total} batch statuses",
                    total = statuses.len()
                )
            }
            Event::Waiting(frequency) => {
                write!(f, "waiting at interval {frequency:?}")
            }
            Event::Error(err) => write!(f, "Error: {err:?}"),
        }
    }
}
