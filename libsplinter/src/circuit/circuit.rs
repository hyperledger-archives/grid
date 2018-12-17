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
pub struct Circuit {
    #[serde(skip)]
    id: String,
    auth: String,
    members: Vec<String>,
    roster: Vec<String>,
    persistence: String,
    durability: String,
    routes: String,
}

impl Circuit {
    pub fn new(
        id: String,
        auth: String,
        members: Vec<String>,
        roster: Vec<String>,
        persistence: String,
        durability: String,
        routes: String,
    ) -> Self {
        Circuit {
            id,
            auth,
            members,
            roster,
            persistence,
            durability,
            routes,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn auth(&self) -> &str {
        &self.auth
    }

    pub fn members(&self) -> &[String] {
        &self.members
    }

    pub fn roster(&self) -> &[String] {
        &self.roster
    }

    pub fn persistence(&self) -> &str {
        &self.persistence
    }

    pub fn durability(&self) -> &str {
        &self.durability
    }

    pub fn routes(&self) -> &str {
        &self.routes
    }
}
