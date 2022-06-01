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

pub mod submission;

use std::convert::From;

use crate::batch_tracking::store::{GlobalTrackingBatch, ServiceTrackingBatch};
use crate::scope_id::{GlobalScopeId, ScopeId, ServiceScopeId};

#[derive(Debug)]
pub struct Submission<S: ScopeId> {
    batch_header: String,
    scope_id: S,
    serialized_batch: Vec<u8>,
}

impl<S: ScopeId> Submission<S> {
    pub fn batch_header(&self) -> &String {
        &self.batch_header
    }

    pub fn scope_id(&self) -> &S {
        &self.scope_id
    }

    pub fn serialized_batch(&self) -> &Vec<u8> {
        &self.serialized_batch
    }
}

impl From<GlobalTrackingBatch> for Submission<GlobalScopeId> {
    fn from(batch: GlobalTrackingBatch) -> Self {
        Self {
            batch_header: batch.batch_header().to_string(),
            scope_id: batch.scope_id().clone(),
            serialized_batch: batch.serialized_batch().to_vec(),
        }
    }
}

impl From<ServiceTrackingBatch> for Submission<ServiceScopeId> {
    fn from(batch: ServiceTrackingBatch) -> Self {
        Self {
            batch_header: batch.batch_header().to_string(),
            scope_id: batch.scope_id().clone(),
            serialized_batch: batch.serialized_batch().to_vec(),
        }
    }
}
