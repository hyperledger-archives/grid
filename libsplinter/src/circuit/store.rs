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

use std::collections::BTreeMap;

use crate::circuit::Circuit;

pub trait CircuitStore: Send + Sync + Clone {
    fn circuits(&self) -> Result<BTreeMap<String, Circuit>, CircuitStoreError>;

    fn circuit(&self, circuit_name: &str) -> Result<Option<Circuit>, CircuitStoreError>;
}

#[derive(Debug)]
pub struct CircuitStoreError {
    context: String,
    source: Option<Box<dyn std::error::Error + Send + 'static>>,
}

impl std::error::Error for CircuitStoreError {}

impl CircuitStoreError {
    pub fn new(context: String) -> Self {
        Self {
            context,
            source: None,
        }
    }

    pub fn from_source<T: std::error::Error + Send + 'static>(context: String, source: T) -> Self {
        Self {
            context,
            source: Some(Box::new(source)),
        }
    }

    pub fn context(&self) -> String {
        self.context.clone()
    }
}

impl std::fmt::Display for CircuitStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(ref source) = self.source {
            write!(
                f,
                "CircuitStoreError: Source: {} Context: {}",
                source, self.context
            )
        } else {
            write!(f, "CircuitStoreError: Context {}", self.context)
        }
    }
}
