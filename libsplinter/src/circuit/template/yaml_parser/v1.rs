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

#[derive(Deserialize, Debug, Clone)]
pub struct CircuitCreateTemplate {
    version: String,
    args: Vec<RuleArgument>,
    rules: Rules,
}

impl CircuitCreateTemplate {
    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn args(&self) -> &[RuleArgument] {
        &self.args
    }

    pub fn rules(&self) -> &Rules {
        &self.rules
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct RuleArgument {
    name: String,
    required: bool,
    #[serde(rename = "default")]
    default_value: Option<String>,
    description: Option<String>,
}

impl RuleArgument {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn required(&self) -> bool {
        self.required
    }

    pub fn default_value(&self) -> Option<&String> {
        self.default_value.as_ref()
    }

    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Rules {
    set_management_type: Option<CircuitManagement>,
    create_services: Option<CreateServices>,
    set_metadata: Option<SetMetadata>,
}

impl Rules {
    pub fn set_management_type(&self) -> Option<&CircuitManagement> {
        self.set_management_type.as_ref()
    }

    pub fn create_services(&self) -> Option<&CreateServices> {
        self.create_services.as_ref()
    }

    pub fn set_metadata(&self) -> Option<&SetMetadata> {
        self.set_metadata.as_ref()
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CircuitManagement {
    management_type: String,
}

impl CircuitManagement {
    pub fn management_type(&self) -> &str {
        &self.management_type
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CreateServices {
    service_type: String,
    service_args: Vec<ServiceArgument>,
    first_service: String,
}

impl CreateServices {
    pub fn service_type(&self) -> &str {
        &self.service_type
    }

    pub fn service_args(&self) -> &[ServiceArgument] {
        &self.service_args
    }

    pub fn first_service(&self) -> &str {
        &self.first_service
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServiceArgument {
    key: String,
    value: Value,
}

impl ServiceArgument {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct SetMetadata {
    #[serde(flatten)]
    metadata: Metadata,
}

impl SetMetadata {
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "encoding")]
#[serde(rename_all(deserialize = "camelCase"))]
pub enum Metadata {
    Json { metadata: Vec<JsonMetadata> },
}

#[derive(Deserialize, Debug, Clone)]
pub struct JsonMetadata {
    key: String,
    value: Value,
}

impl JsonMetadata {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Value {
    Single(String),
    List(Vec<String>),
}
