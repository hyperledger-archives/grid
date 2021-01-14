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

#[cfg(feature = "diesel")]
pub mod diesel;
mod error;

pub use error::AgentStoreError;

/// Represents a Grid Agent
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Agent {
    pub public_key: String,
    pub org_id: String,
    pub active: bool,
    pub metadata: Vec<u8>,
    pub roles: Vec<String>,
    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,

    pub service_id: Option<String>,
}

/// Represents a Grid Agent Role
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Role {
    pub public_key: String,
    pub role_name: String,
    // The indicators of the start and stop for the slowly-changing dimensions.
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

pub trait AgentStore: Send + Sync {
    /// Adds an agent to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `agent` - The agent to be added
    fn add_agent(&self, agent: Agent) -> Result<(), AgentStoreError>;

    ///  Lists agents from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `service_id` - The service id to list agents for
    fn list_agents(&self, service_id: Option<&str>) -> Result<Vec<Agent>, AgentStoreError>;

    /// Fetches an agent from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `pub_key` - This public key of the agent to fetch
    ///  * `service_id` - The service id of the agent to fetch
    fn fetch_agent(
        &self,
        pub_key: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Agent>, AgentStoreError>;

    ///  Updates an agent in the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `agent` - The updated agent to add
    fn update_agent(&self, agent: Agent) -> Result<(), AgentStoreError>;
}

impl<AS> AgentStore for Box<AS>
where
    AS: AgentStore + ?Sized,
{
    fn add_agent(&self, agent: Agent) -> Result<(), AgentStoreError> {
        (**self).add_agent(agent)
    }

    fn list_agents(&self, service_id: Option<&str>) -> Result<Vec<Agent>, AgentStoreError> {
        (**self).list_agents(service_id)
    }

    fn fetch_agent(
        &self,
        pub_key: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Agent>, AgentStoreError> {
        (**self).fetch_agent(pub_key, service_id)
    }

    fn update_agent(&self, agent: Agent) -> Result<(), AgentStoreError> {
        (**self).update_agent(agent)
    }
}
