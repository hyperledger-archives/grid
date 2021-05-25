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

use std::sync::Arc;

use crate::backend::BackendClient;
#[cfg(feature = "backend-sawtooth")]
use crate::backend::SawtoothBackendClient;
#[cfg(feature = "backend-splinter")]
use crate::backend::SplinterBackendClient;

#[derive(Clone)]
pub struct BackendState {
    pub client: Arc<dyn BackendClient + 'static>,
}

impl BackendState {
    pub fn new(client: Arc<dyn BackendClient + 'static>) -> Self {
        Self { client }
    }

    #[cfg(feature = "backend-sawtooth")]
    pub fn with_sawtooth(client: SawtoothBackendClient) -> Self {
        Self {
            client: Arc::new(client),
        }
    }

    #[cfg(feature = "backend-splinter")]
    pub fn with_splinter(client: SplinterBackendClient) -> Self {
        Self {
            client: Arc::new(client),
        }
    }
}
