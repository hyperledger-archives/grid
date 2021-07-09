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

use crate::error::InvalidArgumentError;

use super::error::PikeBuilderError;
use super::Agent;

/// Builder used to create a Pike Agent
#[derive(Clone, Debug, Default)]
pub struct AgentBuilder {
    public_key: Option<String>,
    org_id: Option<String>,
    active: Option<bool>,
    metadata: Option<Vec<u8>>,
    roles: Option<Vec<String>>,
    // The indicators of the start and stop for the slowly-changing dimensions.
    start_commit_num: Option<i64>,
    end_commit_num: Option<i64>,
    service_id: Option<String>,
    last_updated: Option<i64>,
}

impl AgentBuilder {
    /// Creates a new Agent builder
    pub fn new() -> Self {
        AgentBuilder::default()
    }

    /// Set the public key of the Agent
    ///
    /// # Arguments
    ///
    /// * `public_key` - The public key of the Agent being built
    pub fn with_public_key(mut self, public_key: String) -> Self {
        self.public_key = Some(public_key);
        self
    }

    /// Set the organization ID of the Agent
    ///
    /// # Arguments
    ///
    /// * `org_id` - The ID of the organization the Agent being built belongs to
    pub fn with_org_id(mut self, org_id: String) -> Self {
        self.org_id = Some(org_id);
        self
    }

    /// Set the active flag for the Agent
    ///
    /// # Arguments
    ///
    /// * `active` - Boolean representing whether the Agent being built is currently active
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }

    /// Set the metadata of the Agent
    ///
    /// # Arguments
    ///
    /// * `metadata` - Metadata represented as key-value pairs related to the Agent being built
    pub fn with_metadata(mut self, metadata: Vec<u8>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set the roles for the Agent
    ///
    /// # Arguments
    ///
    /// * `roles` - List of roles assigned to the Agent being built
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = Some(roles);
        self
    }

    /// Set the starting commit num for the Agent
    ///
    /// # Arguments
    ///
    /// * `start_commit_num` - The start commit num for the Agent being built
    pub fn with_start_commit_num(mut self, start_commit_num: i64) -> Self {
        self.start_commit_num = Some(start_commit_num);
        self
    }

    /// Set the ending commit num for the Agent
    ///
    /// # Arguments
    ///
    /// * `end_commit_num` - The end commit num for the Agent being built
    pub fn with_end_commit_num(mut self, end_commit_num: i64) -> Self {
        self.end_commit_num = Some(end_commit_num);
        self
    }

    /// Set the service ID of the Agent
    ///
    /// # Arguments
    ///
    /// * `service_id` - The service ID for the Agent being built
    pub fn with_service_id(mut self, service_id: String) -> Self {
        self.service_id = Some(service_id);
        self
    }

    /// Set the last updated timestamp for the Agent
    ///
    /// # Arguments
    ///
    /// * `last_updated` - The timestamp the Agent being built was last updated
    pub fn with_last_updated(mut self, last_updated: i64) -> Self {
        self.last_updated = Some(last_updated);
        self
    }

    pub fn build(self) -> Result<Agent, PikeBuilderError> {
        let public_key = self
            .public_key
            .ok_or_else(|| PikeBuilderError::MissingRequiredField("public_key".to_string()))?;
        let org_id = self
            .org_id
            .ok_or_else(|| PikeBuilderError::MissingRequiredField("org_id".to_string()))?;
        let active = self
            .active
            .ok_or_else(|| PikeBuilderError::MissingRequiredField("active".to_string()))?;
        let metadata = self.metadata.unwrap_or_default();
        let roles = self.roles.unwrap_or_default();

        let start_commit_num = self.start_commit_num.ok_or_else(|| {
            PikeBuilderError::MissingRequiredField("start_commit_num".to_string())
        })?;
        let end_commit_num = self
            .end_commit_num
            .ok_or_else(|| PikeBuilderError::MissingRequiredField("end_commit_num".to_string()))?;

        if start_commit_num >= end_commit_num {
            return Err(PikeBuilderError::InvalidArgumentError(
                InvalidArgumentError::new(
                    "start_commit_num".to_string(),
                    "argument cannot be greater than or equal to `end_commit_num`".to_string(),
                ),
            ));
        }
        if end_commit_num <= start_commit_num {
            return Err(PikeBuilderError::InvalidArgumentError(
                InvalidArgumentError::new(
                    "end_commit_num".to_string(),
                    "argument cannot be less than or equal to `start_commit_num`".to_string(),
                ),
            ));
        }
        let service_id = self.service_id;
        let last_updated = self.last_updated;

        Ok(Agent {
            public_key,
            org_id,
            active,
            metadata,
            roles,
            start_commit_num,
            end_commit_num,
            service_id,
            last_updated,
        })
    }
}
