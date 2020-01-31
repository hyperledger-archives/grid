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

mod error;
mod scar;
mod submit;

use sawtooth_sdk::messages::batch::BatchList;

pub use error::Error;
pub use scar::{SabreSmartContractDefinition, SabreSmartContractMetadata};
use submit::{submit_batches, wait_for_batches};

/// A client that can be used to submit transactions to scabbard services on a Splinter node.
pub struct ScabbardClient {
    url: String,
}

impl ScabbardClient {
    /// Create a new `ScabbardClient` with the given base `url`. The `url` should be the endpoint
    /// of the Splinter node; it should not include the endpoint of the scabbard service itself.
    pub fn new(url: &str) -> Self {
        Self { url: url.into() }
    }

    /// Submit the given batches to the scabbard service specified by the circuit and service IDs.
    /// Optionally wait the given number of seconds for batches to commit.
    pub fn submit(
        &self,
        circuit_id: &str,
        service_id: &str,
        batches: BatchList,
        wait: Option<u64>,
    ) -> Result<(), Error> {
        let batch_link = submit_batches(&self.url, circuit_id, service_id, batches)?;
        if let Some(wait_secs) = wait {
            wait_for_batches(&self.url, &batch_link, wait_secs)
        } else {
            Ok(())
        }
    }
}
