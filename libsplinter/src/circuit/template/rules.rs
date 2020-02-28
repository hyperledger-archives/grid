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

use std::collections::HashMap;

use super::{yaml_parser::v1, CircuitTemplateError};
use super::{Builders, CreateCircuitBuilder};

pub struct CircuitCreateTemplate {
    _version: String,
    _args: Vec<RuleArgument>,
    rules: Rules,
}

impl CircuitCreateTemplate {
    pub fn apply_rules(
        &self,
        builders: &mut Builders,
        arguments: &HashMap<String, String>,
    ) -> Result<(), CircuitTemplateError> {
        self.rules.apply_rules(builders, arguments)
    }
}

impl From<v1::CircuitCreateTemplate> for CircuitCreateTemplate {
    fn from(create_circuit_template: v1::CircuitCreateTemplate) -> Self {
        CircuitCreateTemplate {
            _version: create_circuit_template.version().to_string(),
            _args: create_circuit_template
                .args()
                .to_owned()
                .into_iter()
                .map(RuleArgument::from)
                .collect(),
            rules: Rules::from(create_circuit_template.rules().clone()),
        }
    }
}

struct RuleArgument {
    _name: String,
    _required: bool,
    _default_value: Option<String>,
}

impl From<v1::RuleArgument> for RuleArgument {
    fn from(arguments: v1::RuleArgument) -> Self {
        RuleArgument {
            _name: arguments.name().to_string(),
            _required: arguments.required(),
            _default_value: arguments.default_value().map(String::from),
        }
    }
}

struct Rules {
    set_management_type: Option<CircuitManagement>,
}

impl Rules {
    fn apply_rules(
        &self,
        builders: &mut Builders,
        _arguments: &HashMap<String, String>,
    ) -> Result<(), CircuitTemplateError> {
        let mut create_service_builder = builders.create_circuit_builder();

        if let Some(circuit_management) = &self.set_management_type {
            create_service_builder = circuit_management.apply_rule(create_service_builder)?;
        }

        builders.set_create_circuit_builder(create_service_builder);
        Ok(())
    }
}

impl From<v1::Rules> for Rules {
    fn from(rules: v1::Rules) -> Self {
        Rules {
            set_management_type: rules
                .set_management_type()
                .map(|val| CircuitManagement::from(val.clone())),
        }
    }
}

struct CircuitManagement {
    management_type: String,
}

impl CircuitManagement {
    fn apply_rule(
        &self,
        builder: CreateCircuitBuilder,
    ) -> Result<CreateCircuitBuilder, CircuitTemplateError> {
        Ok(builder.with_circuit_management_type(&self.management_type))
    }
}

impl From<v1::CircuitManagement> for CircuitManagement {
    fn from(yaml_circuit_management: v1::CircuitManagement) -> Self {
        CircuitManagement {
            management_type: yaml_circuit_management.management_type().to_string(),
        }
    }
}
