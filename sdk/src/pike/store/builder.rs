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
use super::{Agent, AlternateId, Organization, OrganizationMetadata, Role};

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

/// Builder used to create a Pike role
#[derive(Clone, Debug, Default)]
pub struct RoleBuilder {
    name: Option<String>,
    org_id: Option<String>,
    description: Option<String>,
    active: Option<bool>,
    permissions: Option<Vec<String>>,
    allowed_organizations: Option<Vec<String>>,
    inherit_from: Option<Vec<String>>,
    start_commit_num: Option<i64>,
    end_commit_num: Option<i64>,
    service_id: Option<String>,
    last_updated: Option<i64>,
}

impl RoleBuilder {
    /// Creates a new role builder
    pub fn new() -> Self {
        RoleBuilder::default()
    }

    /// Set the name of the role
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the role being built
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Set the organization ID of the role
    ///
    /// # Arguments
    ///
    /// * `org_id` - The ID of the organization the role being built belongs to
    pub fn with_org_id(mut self, org_id: String) -> Self {
        self.org_id = Some(org_id);
        self
    }

    /// Set the description of the role
    ///
    /// # Arguments
    ///
    /// * `description` - The description of the role being built
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set the active flag for the role
    ///
    /// # Arguments
    ///
    /// * `active` - Boolean representing whether the role being built is currently active
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }

    /// Set the permissions of the role
    ///
    /// # Arguments
    ///
    /// * `permissions` - The list of permissions belonging to the role being built
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = Some(permissions);
        self
    }

    /// Set the allowed organizations of the role
    ///
    /// # Arguments
    ///
    /// * `allowed_organizations` - The list of allowed organization IDs of the role being built
    pub fn with_allowed_organizations(mut self, allowed_organizations: Vec<String>) -> Self {
        self.allowed_organizations = Some(allowed_organizations);
        self
    }

    /// Set the list of roles the role being built is able to inherit permissions from
    ///
    /// # Arguments
    ///
    /// * `inherit_from` - List of roles the role being built inherits permissions from
    pub fn with_inherit_from(mut self, inherit_from: Vec<String>) -> Self {
        self.inherit_from = Some(inherit_from);
        self
    }

    /// Set the starting commit num for the role
    ///
    /// # Arguments
    ///
    /// * `start_commit_num` - The start commit num for the role being built
    pub fn with_start_commit_num(mut self, start_commit_num: i64) -> Self {
        self.start_commit_num = Some(start_commit_num);
        self
    }

    /// Set the ending commit num for the role
    ///
    /// # Arguments
    ///
    /// * `end_commit_num` - The end commit num for the role being built
    pub fn with_end_commit_num(mut self, end_commit_num: i64) -> Self {
        self.end_commit_num = Some(end_commit_num);
        self
    }

    /// Set the service ID of the role
    ///
    /// # Arguments
    ///
    /// * `service_id` - The service ID for the role being built
    pub fn with_service_id(mut self, service_id: String) -> Self {
        self.service_id = Some(service_id);
        self
    }

    /// Set the last updated timestamp for the role
    ///
    /// # Arguments
    ///
    /// * `last_updated` - The timestamp the role being built was last updated
    pub fn with_last_updated(mut self, last_updated: i64) -> Self {
        self.last_updated = Some(last_updated);
        self
    }

    pub fn build(self) -> Result<Role, PikeBuilderError> {
        let name = self
            .name
            .ok_or_else(|| PikeBuilderError::MissingRequiredField("name".to_string()))?;
        let org_id = self
            .org_id
            .ok_or_else(|| PikeBuilderError::MissingRequiredField("org_id".to_string()))?;
        let description = self.description.unwrap_or_default();
        let active = self
            .active
            .ok_or_else(|| PikeBuilderError::MissingRequiredField("active".to_string()))?;
        let permissions = self.permissions.unwrap_or_default();
        let allowed_organizations = self.allowed_organizations.unwrap_or_default();
        let inherit_from = self.inherit_from.unwrap_or_default();
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

        Ok(Role {
            name,
            org_id,
            description,
            active,
            permissions,
            allowed_organizations,
            inherit_from,
            start_commit_num,
            end_commit_num,
            service_id,
            last_updated,
        })
    }
}

/// Builder used to create a Pike organization
#[derive(Clone, Debug, Default)]
pub struct OrganizationBuilder {
    org_id: Option<String>,
    name: Option<String>,
    locations: Option<Vec<String>>,
    alternate_ids: Option<Vec<AlternateId>>,
    metadata: Option<Vec<OrganizationMetadata>>,
    start_commit_num: Option<i64>,
    end_commit_num: Option<i64>,
    service_id: Option<String>,
    last_updated: Option<i64>,
}

impl OrganizationBuilder {
    /// Creates a new organization builder
    pub fn new() -> Self {
        OrganizationBuilder::default()
    }

    /// Set the organization's ID
    ///
    /// # Arguments
    ///
    /// * `org_id` - The unique identifier of this organization
    pub fn with_org_id(mut self, org_id: String) -> Self {
        self.org_id = Some(org_id);
        self
    }

    /// Set the name of the organization
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the organization being built
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Set the locations of the organization
    ///
    /// # Arguments
    ///
    /// * `locations` - The locations of the organization being built
    pub fn with_locations(mut self, locations: Vec<String>) -> Self {
        self.locations = Some(locations);
        self
    }

    /// Set the alternate IDs for the organization
    ///
    /// # Arguments
    ///
    /// * `alternate_ids` - List of alternate IDs belonging to the organization being built
    pub fn with_alternate_ids(mut self, alternate_ids: Vec<AlternateId>) -> Self {
        self.alternate_ids = Some(alternate_ids);
        self
    }

    /// Set the metadata of the organization
    ///
    /// # Arguments
    ///
    /// * `metadata` - The metadata belonging to the organization being built
    pub fn with_metadata(mut self, metadata: Vec<OrganizationMetadata>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set the starting commit num for the organization
    ///
    /// # Arguments
    ///
    /// * `start_commit_num` - The start commit num for the organization being built
    pub fn with_start_commit_num(mut self, start_commit_num: i64) -> Self {
        self.start_commit_num = Some(start_commit_num);
        self
    }

    /// Set the ending commit num for the organization
    ///
    /// # Arguments
    ///
    /// * `end_commit_num` - The end commit num for the organization being built
    pub fn with_end_commit_num(mut self, end_commit_num: i64) -> Self {
        self.end_commit_num = Some(end_commit_num);
        self
    }

    /// Set the service ID of the organization
    ///
    /// # Arguments
    ///
    /// * `service_id` - The service ID for the organization being built
    pub fn with_service_id(mut self, service_id: String) -> Self {
        self.service_id = Some(service_id);
        self
    }

    /// Set the last updated timestamp for the organization
    ///
    /// # Arguments
    ///
    /// * `last_updated` - The timestamp the organization being built was last updated
    pub fn with_last_updated(mut self, last_updated: i64) -> Self {
        self.last_updated = Some(last_updated);
        self
    }

    pub fn build(self) -> Result<Organization, PikeBuilderError> {
        let org_id = self
            .org_id
            .ok_or_else(|| PikeBuilderError::MissingRequiredField("org_id".to_string()))?;
        let name = self
            .name
            .ok_or_else(|| PikeBuilderError::MissingRequiredField("name".to_string()))?;
        let locations = self.locations.unwrap_or_default();
        let alternate_ids = self.alternate_ids.unwrap_or_default();
        let metadata = self.metadata.unwrap_or_default();
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

        Ok(Organization {
            org_id,
            name,
            locations,
            alternate_ids,
            metadata,
            start_commit_num,
            end_commit_num,
            service_id,
            last_updated,
        })
    }
}

/// Builder used to create an Alternate ID
#[derive(Clone, Debug, Default)]
pub struct AlternateIdBuilder {
    org_id: Option<String>,
    alternate_id_type: Option<String>,
    alternate_id: Option<String>,
    start_commit_num: Option<i64>,
    end_commit_num: Option<i64>,
    service_id: Option<String>,
}

impl AlternateIdBuilder {
    /// Creates a new Alternate ID builder
    pub fn new() -> Self {
        AlternateIdBuilder::default()
    }

    /// Set the unique identifier for the organization associated with the Alternate ID
    ///
    /// # Arguments
    ///
    /// * `org_id` - The unique identifier of the organization the Alternate ID belongs to
    pub fn with_org_id(mut self, org_id: String) -> Self {
        self.org_id = Some(org_id);
        self
    }

    /// Set the type of the Alternate ID
    ///
    /// # Arguments
    ///
    /// * `alternate_id_type` - Type of the Alternate ID being built
    pub fn with_alternate_id_type(mut self, alternate_id_type: String) -> Self {
        self.alternate_id_type = Some(alternate_id_type);
        self
    }

    /// Set the unique identifier of the Alternate ID
    ///
    /// # Arguments
    ///
    /// * `alternate_id` - Unique identifier of the Alternate ID being built
    pub fn with_alternate_id(mut self, alternate_id: String) -> Self {
        self.alternate_id = Some(alternate_id);
        self
    }

    /// Set the starting commit num for the Alternate ID
    ///
    /// # Arguments
    ///
    /// * `start_commit_num` - The start commit num for the Alternate ID being built
    pub fn with_start_commit_num(mut self, start_commit_num: i64) -> Self {
        self.start_commit_num = Some(start_commit_num);
        self
    }

    /// Set the ending commit num for the Alternate ID
    ///
    /// # Arguments
    ///
    /// * `end_commit_num` - The end commit num for the Alternate ID being built
    pub fn with_end_commit_num(mut self, end_commit_num: i64) -> Self {
        self.end_commit_num = Some(end_commit_num);
        self
    }

    /// Set the service ID of the Alternate ID
    ///
    /// # Arguments
    ///
    /// * `service_id` - The service ID for the Alternate ID being built
    pub fn with_service_id(mut self, service_id: String) -> Self {
        self.service_id = Some(service_id);
        self
    }

    pub fn build(self) -> Result<AlternateId, PikeBuilderError> {
        let org_id = self
            .org_id
            .ok_or_else(|| PikeBuilderError::MissingRequiredField("org_id".to_string()))?;
        let alternate_id_type = self.alternate_id_type.ok_or_else(|| {
            PikeBuilderError::MissingRequiredField("alternate_id_type".to_string())
        })?;
        let alternate_id = self
            .alternate_id
            .ok_or_else(|| PikeBuilderError::MissingRequiredField("alternate_id".to_string()))?;

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

        Ok(AlternateId {
            org_id,
            alternate_id_type,
            alternate_id,
            start_commit_num,
            end_commit_num,
            service_id,
        })
    }
}

/// Builder used to create organization metadata, represented as a key-value pair
#[derive(Clone, Debug, Default)]
pub struct OrganizationMetadataBuilder {
    key: Option<String>,
    value: Option<String>,
    start_commit_num: Option<i64>,
    end_commit_num: Option<i64>,
    service_id: Option<String>,
}

impl OrganizationMetadataBuilder {
    /// Creates a new organization metadata builder
    pub fn new() -> Self {
        OrganizationMetadataBuilder::default()
    }

    /// Set the key for the organization metadata
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the organization's metadata's internal key-value pair
    pub fn with_key(mut self, key: String) -> Self {
        self.key = Some(key);
        self
    }

    /// Set the value for the organization metadata
    ///
    /// # Arguments
    ///
    /// * `value` - The value of the organization's metadata's internal key-value pair
    pub fn with_value(mut self, value: String) -> Self {
        self.value = Some(value);
        self
    }

    /// Set the starting commit num for the organization metadata
    ///
    /// # Arguments
    ///
    /// * `start_commit_num` - The start commit num for the organization metadata being built
    pub fn with_start_commit_num(mut self, start_commit_num: i64) -> Self {
        self.start_commit_num = Some(start_commit_num);
        self
    }

    /// Set the ending commit num for the organization metadata
    ///
    /// # Arguments
    ///
    /// * `end_commit_num` - The end commit num for the organization metadata being built
    pub fn with_end_commit_num(mut self, end_commit_num: i64) -> Self {
        self.end_commit_num = Some(end_commit_num);
        self
    }

    /// Set the service ID of the organization metadata
    ///
    /// # Arguments
    ///
    /// * `service_id` - The service ID for the organization metadata being built
    pub fn with_service_id(mut self, service_id: String) -> Self {
        self.service_id = Some(service_id);
        self
    }

    pub fn build(self) -> Result<OrganizationMetadata, PikeBuilderError> {
        let key = self
            .key
            .ok_or_else(|| PikeBuilderError::MissingRequiredField("key".to_string()))?;
        let value = self
            .value
            .ok_or_else(|| PikeBuilderError::MissingRequiredField("value".to_string()))?;

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

        Ok(OrganizationMetadata {
            key,
            value,
            start_commit_num,
            end_commit_num,
            service_id,
        })
    }
}
