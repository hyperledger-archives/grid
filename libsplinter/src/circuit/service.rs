// Copyright 2018 Cargill Incorporated
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

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Service {
    id: String,
    node: SplinterNode,
}

impl Service {
    pub fn new(id: String, node: SplinterNode) -> Self {
        Service { id, node }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct SplinterNode {
    #[serde(skip)]
    id: String,
    endpoints: Vec<String>,
}

impl SplinterNode {
    pub fn new(id: String, endpoints: Vec<String>) -> Self {
        SplinterNode { id, endpoints }
    }

    pub fn endpoints(&self) -> &[String] {
        &self.endpoints
    }
}
