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

mod create_services;
mod set_management_type;
mod set_metadata;

use std::convert::TryFrom;

use super::{yaml_parser::v1, Builders, CircuitTemplateError};

use create_services::CreateServices;
use set_management_type::CircuitManagement;
use set_metadata::SetMetadata;

pub struct Rules {
    set_management_type: Option<CircuitManagement>,
    create_services: Option<CreateServices>,
    set_metadata: Option<SetMetadata>,
}

impl Rules {
    pub fn apply_rules(
        &self,
        builders: &mut Builders,
        template_arguments: &[RuleArgument],
    ) -> Result<(), CircuitTemplateError> {
        let mut service_builders = builders.service_builders();

        let mut circuit_builder = builders.create_circuit_builder();

        if let Some(circuit_management) = &self.set_management_type {
            circuit_builder = circuit_management.apply_rule(circuit_builder)?;
        }

        if let Some(create_services) = &self.create_services {
            service_builders.extend(create_services.apply_rule(template_arguments)?);
        }

        if let Some(set_metadata) = &self.set_metadata {
            circuit_builder = set_metadata.apply_rule(circuit_builder, template_arguments)?;
        }

        builders.set_create_circuit_builder(circuit_builder);
        builders.set_service_builders(service_builders);
        Ok(())
    }
}

impl From<v1::Rules> for Rules {
    fn from(rules: v1::Rules) -> Self {
        Rules {
            set_management_type: rules
                .set_management_type()
                .map(|val| CircuitManagement::from(val.clone())),
            create_services: rules
                .create_services()
                .map(|val| CreateServices::from(val.clone())),
            set_metadata: rules
                .set_metadata()
                .map(|val| SetMetadata::from(val.clone())),
        }
    }
}

#[derive(Clone)]
pub struct RuleArgument {
    name: String,
    required: bool,
    default_value: Option<String>,
    description: Option<String>,
    user_value: Option<String>,
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

    pub fn user_value(&self) -> Option<&String> {
        self.user_value.as_ref()
    }

    pub fn set_user_value(&mut self, value: &str) {
        self.user_value = Some(value.to_string())
    }
}

impl TryFrom<v1::RuleArgument> for RuleArgument {
    type Error = CircuitTemplateError;
    fn try_from(arguments: v1::RuleArgument) -> Result<Self, Self::Error> {
        Ok(RuleArgument {
            name: strip_arg_marker(arguments.name())?,
            required: arguments.required(),
            default_value: arguments.default_value().map(String::from),
            description: arguments.description().map(String::from),
            user_value: None,
        })
    }
}

fn is_arg(key: &str) -> bool {
    key.starts_with("$(a:")
}

fn strip_arg_marker(key: &str) -> Result<String, CircuitTemplateError> {
    if key.starts_with("$(a:") && key.ends_with(')') {
        let mut key = key.to_string();
        key.pop();
        Ok(key
            .get(4..)
            .ok_or_else(|| {
                CircuitTemplateError::new(&format!("{} is not a valid argument name", key))
            })?
            .to_string()
            .to_lowercase())
    } else {
        Err(CircuitTemplateError::new(&format!(
            "{} is not a valid argument name",
            key
        )))
    }
}

#[derive(Debug)]
enum Value {
    Single(String),
    List(Vec<String>),
}

impl From<v1::Value> for Value {
    fn from(value: v1::Value) -> Self {
        match value {
            v1::Value::Single(value) => Self::Single(value),
            v1::Value::List(values) => Self::List(values),
        }
    }
}

fn get_argument_value(
    key: &str,
    template_arguments: &[RuleArgument],
) -> Result<String, CircuitTemplateError> {
    let key = strip_arg_marker(key)?;
    let value = match template_arguments.iter().find(|arg| arg.name == key) {
        Some(arg) => match arg.user_value() {
            Some(val) => val.to_string(),
            None => {
                if arg.required {
                    return Err(CircuitTemplateError::new(&format!(
                        "Argument {} is required but was not provided",
                        key
                    )));
                } else {
                    let default_value = arg.default_value.to_owned().ok_or_else(|| {
                        CircuitTemplateError::new(&format!(
                            "Argument {} was not provided and no default value is set",
                            key
                        ))
                    })?;
                    if is_arg(&default_value) {
                        get_argument_value(&default_value, template_arguments)?
                    } else {
                        default_value
                    }
                }
            }
        },
        None => {
            return Err(CircuitTemplateError::new(&format!(
                "Invalid template. Argument {} was expected but not provided",
                key
            )));
        }
    };

    Ok(value)
}
