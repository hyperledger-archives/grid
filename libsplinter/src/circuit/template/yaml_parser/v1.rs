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
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Rules {
    set_management_type: Option<CircuitManagement>,
}

impl Rules {
    pub fn set_management_type(&self) -> Option<&CircuitManagement> {
        self.set_management_type.as_ref()
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
