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

//! Identifies the scope context of a batch
//!
//! The scope ID describes and identifies the partition with which batches can be applied. For
//! example, Sawtooth uses a `GlobalScopeId`, meaning that all batches fall into the same global
//! partition. Scabbard, on the other hand, uses a `ServiceScopeId`, meaning that batches are
//! partitioned by `service_id` - they must be submitted only to the `service_id` with which they
//! are associated.

use crate::error::InvalidArgumentError;

mod service;
pub use service::{CircuitId, FullyQualifiedServiceId, ServiceId};

pub trait ValidId {
    // Designate that the ID is valid
}

/// The scope ID trait.
pub trait ScopeId: 'static + ValidId + Clone + std::fmt::Debug + PartialEq + Sync + Send {
    // Require a collection of trait bounds for a valid scope id
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GlobalScopeId {}
impl ScopeId for GlobalScopeId {}
impl ValidId for GlobalScopeId {}

impl GlobalScopeId {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServiceScopeId {
    service_id: FullyQualifiedServiceId,
}
impl ScopeId for ServiceScopeId {}
impl ValidId for ServiceScopeId {}

impl ServiceScopeId {
    pub fn new_from_string(full_service_id: String) -> Result<Self, InvalidArgumentError> {
        let service_id = FullyQualifiedServiceId::new_from_string(full_service_id)?;
        Ok(Self { service_id })
    }

    pub fn service_id(&self) -> &FullyQualifiedServiceId {
        &self.service_id
    }
}
