// Copyright 2018-2022 Cargill Incorporated
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

use crate::rest_api::resources::submit::v2::error::BuilderError;

use super::{PropertyValue, TransactionPayload};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub enum LocationNamespace {
    #[serde(rename = "GS1")]
    Gs1,
}

impl Default for LocationNamespace {
    fn default() -> Self {
        LocationNamespace::Gs1
    }
}

// Allow the `LocationAction` enum variants to have the same `Location` postfix
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum LocationAction {
    CreateLocation(CreateLocationAction),
    UpdateLocation(UpdateLocationAction),
    DeleteLocation(DeleteLocationAction),
}

impl LocationAction {
    pub fn into_inner(self) -> Box<dyn TransactionPayload> {
        unimplemented!();
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct LocationPayload {
    #[serde(flatten)]
    action: LocationAction,
    timestamp: u64,
}

impl LocationPayload {
    pub fn new(action: LocationAction, timestamp: u64) -> Self {
        LocationPayload { action, timestamp }
    }

    pub fn action(&self) -> &LocationAction {
        &self.action
    }

    pub fn timestamp(&self) -> &u64 {
        &self.timestamp
    }

    pub fn into_transaction_payload(self) -> Box<dyn TransactionPayload> {
        self.action.into_inner()
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct CreateLocationAction {
    namespace: LocationNamespace,
    location_id: String,
    owner: String,
    #[serde(default)]
    properties: Vec<PropertyValue>,
}

impl CreateLocationAction {
    pub fn namespace(&self) -> &LocationNamespace {
        &self.namespace
    }

    pub fn location_id(&self) -> &str {
        &self.location_id
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn properties(&self) -> &[PropertyValue] {
        &self.properties
    }
}

#[derive(Default, Debug)]
pub struct CreateLocationActionBuilder {
    namespace: Option<LocationNamespace>,
    location_id: Option<String>,
    owner: Option<String>,
    properties: Option<Vec<PropertyValue>>,
}

impl CreateLocationActionBuilder {
    pub fn new() -> Self {
        CreateLocationActionBuilder::default()
    }
    pub fn with_namespace(mut self, value: LocationNamespace) -> Self {
        self.namespace = Some(value);
        self
    }
    pub fn with_location_id(mut self, value: String) -> Self {
        self.location_id = Some(value);
        self
    }
    pub fn with_owner(mut self, value: String) -> Self {
        self.owner = Some(value);
        self
    }
    pub fn with_properties(mut self, value: Vec<PropertyValue>) -> Self {
        self.properties = Some(value);
        self
    }
    pub fn build(self) -> Result<CreateLocationAction, BuilderError> {
        let namespace = self.namespace.ok_or_else(|| {
            BuilderError::MissingField("'namespace' field is required".to_string())
        })?;
        let location_id = self
            .location_id
            .ok_or_else(|| BuilderError::MissingField("'location_id' field is required".into()))?;
        let owner = self
            .owner
            .ok_or_else(|| BuilderError::MissingField("'owner' field is required".into()))?;
        let properties = self
            .properties
            .ok_or_else(|| BuilderError::MissingField("'properties' field is required".into()))?;
        Ok(CreateLocationAction {
            namespace,
            location_id,
            owner,
            properties,
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct UpdateLocationAction {
    namespace: LocationNamespace,
    location_id: String,
    properties: Vec<PropertyValue>,
}

impl UpdateLocationAction {
    pub fn namespace(&self) -> &LocationNamespace {
        &self.namespace
    }

    pub fn location_id(&self) -> &str {
        &self.location_id
    }

    pub fn properties(&self) -> &[PropertyValue] {
        &self.properties
    }
}

#[derive(Default, Clone)]
pub struct UpdateLocationActionBuilder {
    namespace: Option<LocationNamespace>,
    location_id: Option<String>,
    properties: Vec<PropertyValue>,
}

impl UpdateLocationActionBuilder {
    pub fn new() -> Self {
        UpdateLocationActionBuilder::default()
    }

    pub fn with_namespace(mut self, namespace: LocationNamespace) -> Self {
        self.namespace = Some(namespace);
        self
    }

    pub fn with_location_id(mut self, location_id: String) -> Self {
        self.location_id = Some(location_id);
        self
    }

    pub fn with_properties(mut self, properties: Vec<PropertyValue>) -> Self {
        self.properties = properties;
        self
    }

    pub fn build(self) -> Result<UpdateLocationAction, BuilderError> {
        let namespace = self.namespace.ok_or_else(|| {
            BuilderError::MissingField("'namespace' field is required".to_string())
        })?;

        let location_id = self.location_id.ok_or_else(|| {
            BuilderError::MissingField("'location_id' field is required".to_string())
        })?;

        let properties = {
            if !self.properties.is_empty() {
                self.properties
            } else {
                return Err(BuilderError::MissingField(
                    "'properties' field is required".to_string(),
                ));
            }
        };

        Ok(UpdateLocationAction {
            namespace,
            location_id,
            properties,
        })
    }
}
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct DeleteLocationAction {
    namespace: LocationNamespace,
    location_id: String,
}

impl DeleteLocationAction {
    pub fn namespace(&self) -> &LocationNamespace {
        &self.namespace
    }

    pub fn location_id(&self) -> &str {
        &self.location_id
    }
}

#[derive(Default, Clone)]
pub struct DeleteLocationActionBuilder {
    namespace: Option<LocationNamespace>,
    location_id: Option<String>,
}

impl DeleteLocationActionBuilder {
    pub fn new() -> Self {
        DeleteLocationActionBuilder::default()
    }

    pub fn with_namespace(mut self, namespace: LocationNamespace) -> Self {
        self.namespace = Some(namespace);
        self
    }

    pub fn with_location_id(mut self, location_id: String) -> Self {
        self.location_id = Some(location_id);
        self
    }

    pub fn build(self) -> Result<DeleteLocationAction, BuilderError> {
        let namespace = self.namespace.ok_or_else(|| {
            BuilderError::MissingField("'namespace' field is required".to_string())
        })?;

        let location_id = self.location_id.ok_or_else(|| {
            BuilderError::MissingField("'location_id' field is required".to_string())
        })?;

        Ok(DeleteLocationAction {
            namespace,
            location_id,
        })
    }
}
