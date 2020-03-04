// Copyright 2018-2020 Cargill Incorporated
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

use std::sync::{Arc, Mutex};

use super::messages::CircuitProposal;
use super::open_proposals::Proposals;
use super::shared::AdminServiceShared;

pub trait ProposalStore: Send + Sync + Clone {
    fn proposals(&self) -> Result<Proposals, ProposalStoreError>;

    fn proposal(&self, circuit_id: &str) -> Result<Option<CircuitProposal>, ProposalStoreError>;
}

#[derive(Debug)]
pub struct ProposalStoreError {
    context: String,
    source: Option<Box<dyn std::error::Error + Send + 'static>>,
}

impl std::error::Error for ProposalStoreError {}

impl ProposalStoreError {
    pub fn new(context: &str) -> Self {
        Self {
            context: context.into(),
            source: None,
        }
    }

    pub fn from_source<T: std::error::Error + Send + 'static>(context: &str, source: T) -> Self {
        Self {
            context: context.into(),
            source: Some(Box::new(source)),
        }
    }

    pub fn context(&self) -> &str {
        &self.context
    }
}

impl std::fmt::Display for ProposalStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(ref source) = self.source {
            write!(
                f,
                "ProposalStoreError: Source: {} Context: {}",
                source, self.context
            )
        } else {
            write!(f, "ProposalStoreError: Context {}", self.context)
        }
    }
}

#[derive(Clone)]
pub(super) struct AdminServiceProposals {
    shared: Arc<Mutex<AdminServiceShared>>,
}

impl AdminServiceProposals {
    pub fn new(shared: &Arc<Mutex<AdminServiceShared>>) -> Self {
        Self {
            shared: Arc::clone(shared),
        }
    }
}

impl ProposalStore for AdminServiceProposals {
    fn proposals(&self) -> Result<Proposals, ProposalStoreError> {
        Ok(self
            .shared
            .lock()
            .map_err(|_| ProposalStoreError::new("Admin shared lock was lock poisoned"))?
            .get_proposals())
    }

    fn proposal(&self, circuit_id: &str) -> Result<Option<CircuitProposal>, ProposalStoreError> {
        self.shared
            .lock()
            .map_err(|_| ProposalStoreError::new("Admin shared lock was lock poisoned"))?
            .get_proposal(circuit_id)
            .map_err(|err| {
                ProposalStoreError::from_source("Unable to get proposal", Box::new(err))
            })?
            .map(|proto| {
                CircuitProposal::from_proto(proto).map_err(|err| {
                    ProposalStoreError::from_source(
                        "Unable to convert proposal protobuf to native",
                        Box::new(err),
                    )
                })
            })
            .transpose()
    }
}
