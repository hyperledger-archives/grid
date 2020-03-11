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

use splinter::circuit::template::{
    Builders, CircuitCreateTemplate, CircuitTemplateError, CircuitTemplateManager, RuleArgument,
};

use crate::error::CliError;

const NODES_ARG: &str = "nodes";

pub struct CircuitTemplate {
    template: CircuitCreateTemplate,
    arguments: HashMap<String, String>,
}

impl CircuitTemplate {
    pub fn list_available_templates() -> Result<Vec<String>, CliError> {
        let manager = CircuitTemplateManager::default();
        let templates = manager.list_available_templates()?;
        Ok(templates)
    }

    /// Loads a YAML circuit template file into a YAML string
    pub fn load_raw(name: &str) -> Result<String, CliError> {
        let manager = CircuitTemplateManager::default();
        let template_yaml = manager.load_raw_yaml(name)?;
        Ok(template_yaml)
    }

    /// Loads a YAML circuit template file and returns a CircuitTemplate that can be used to
    /// build CircuitCreate messages.
    pub fn load(name: &str) -> Result<Self, CliError> {
        let manager = CircuitTemplateManager::default();
        let possible_values = manager.list_available_templates()?;
        if !possible_values.iter().any(|val| val == name) {
            return Err(CliError::ActionError(format!(
                "Template with name {} was not found. Available templates: {:?}",
                name, possible_values
            )));
        }
        let template = manager.load(name)?;
        Ok(CircuitTemplate {
            template,
            arguments: HashMap::new(),
        })
    }

    fn check_missing_required_arguments(&self) -> Vec<String> {
        self.template
            .arguments()
            .iter()
            .filter_map(|template_argument| {
                if template_argument.required()
                    && self.arguments.get(template_argument.name()).is_none()
                {
                    Some(template_argument.name().to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn set_nodes(&mut self, nodes: &[String]) {
        if self
            .template
            .arguments()
            .iter()
            .any(|arg| arg.name() == NODES_ARG)
        {
            self.arguments
                .insert(NODES_ARG.to_string(), nodes.join(","));
        }
    }

    pub fn add_arguments(&mut self, user_arguments: &HashMap<String, String>) {
        self.arguments.extend(user_arguments.clone())
    }

    pub fn arguments(&self) -> &[RuleArgument] {
        self.template.arguments()
    }

    pub fn into_builders(mut self) -> Result<Builders, CliError> {
        let missing_args = self.check_missing_required_arguments();
        if !missing_args.is_empty() {
            return Err(CliError::ActionError(format!(
                "Required arguments were not set: {}",
                missing_args.join(", ")
            )));
        }

        for (key, value) in self.arguments.iter() {
            self.template.set_argument_value(key, value)?;
        }

        Ok(self.template.into_builders()?)
    }
}

impl From<CircuitTemplateError> for CliError {
    fn from(err: CircuitTemplateError) -> CliError {
        CliError::ActionError(format!("Failed to process template: {}", err))
    }
}
