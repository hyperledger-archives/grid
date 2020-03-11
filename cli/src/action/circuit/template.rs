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

use clap::ArgMatches;

use crate::error::CliError;
use crate::template::CircuitTemplate;

use super::Action;

pub struct ListCircuitTemplates;

impl Action for ListCircuitTemplates {
    fn run<'a>(&mut self, _: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let templates = CircuitTemplate::list_available_templates()?;

        println!("Available templates:");
        for name in templates {
            println!("{}", name);
        }
        Ok(())
    }
}

pub struct ShowCircuitTemplate;

impl Action for ShowCircuitTemplate {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;
        let template_name = match args.value_of("name") {
            Some(name) => name,
            None => return Err(CliError::ActionError("Name is required".into())),
        };

        let template = CircuitTemplate::load_raw(template_name)?;

        println!("{}", template);

        Ok(())
    }
}

pub struct ListCircuitTemplateArguments;

impl Action for ListCircuitTemplateArguments {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;
        let template_name = match args.value_of("name") {
            Some(name) => name,
            None => return Err(CliError::ActionError("Name is required".into())),
        };

        let template = CircuitTemplate::load(template_name)?;

        let arguments = template.arguments();
        for argument in arguments {
            println!("\nname: {}", argument.name());
            println!("required: {}", argument.required());
            println!(
                "default_value: {}",
                argument.default_value().unwrap_or(&"Not set".to_string())
            );
            println!(
                "description: {}",
                argument.description().unwrap_or(&"Not set".to_string())
            );
        }

        Ok(())
    }
}
