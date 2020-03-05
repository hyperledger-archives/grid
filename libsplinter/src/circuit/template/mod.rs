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

mod error;
mod rules;
mod yaml_parser;

pub use error::CircuitTemplateError;

use rules::{RuleArgument, Rules};
use yaml_parser::{v1, CircuitTemplate};

pub(self) use crate::admin::messages::{CreateCircuitBuilder, SplinterServiceBuilder};

pub struct CircuitCreateTemplate {
    version: String,
    arguments: Vec<RuleArgument>,
    rules: Rules,
}

impl CircuitCreateTemplate {
    pub fn from_yaml_file(path: &str) -> Result<Self, CircuitTemplateError> {
        let circuit_template = CircuitTemplate::load_from_file(path)?;
        match circuit_template {
            CircuitTemplate::V1(template) => Ok(Self::from(template)),
        }
    }

    pub fn into_builders(self) -> Result<Builders, CircuitTemplateError> {
        let mut builders = Builders {
            create_circuit_builder: CreateCircuitBuilder::new(),
            service_builders: vec![],
        };

        self.rules.apply_rules(&mut builders, &self.arguments)?;

        Ok(builders)
    }

}

impl From<v1::CircuitCreateTemplate> for CircuitCreateTemplate {
    fn from(create_circuit_template: v1::CircuitCreateTemplate) -> Self {
        CircuitCreateTemplate {
            version: create_circuit_template.version().to_string(),
            arguments: create_circuit_template
                .args()
                .to_owned()
                .into_iter()
                .map(RuleArgument::from)
                .collect(),
            rules: Rules::from(create_circuit_template.rules().clone()),
        }
    }
}

pub struct Builders {
    create_circuit_builder: CreateCircuitBuilder,
    service_builders: Vec<SplinterServiceBuilder>,
}

impl Builders {
    pub fn set_create_circuit_builder(&mut self, builder: CreateCircuitBuilder) {
        self.create_circuit_builder = builder;
    }

    pub fn set_service_builders(&mut self, builders: Vec<SplinterServiceBuilder>) {
        self.service_builders = builders;
    }

    pub fn create_circuit_builder(&self) -> CreateCircuitBuilder {
        self.create_circuit_builder.clone()
    }

    pub fn service_builders(&self) -> Vec<SplinterServiceBuilder> {
        self.service_builders.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    use tempdir::TempDir;

    const EXAMPLE_TEMPLATE_YAML: &[u8] = br##"version: v1
args:
    - name: admin-keys
      required: true
      default: $(a:SIGNER_PUB_KEY)
rules:
    set-management-type:
        management-type: "gameroom" "##;

    /*
     * Verifies that Builders can be parsed from template v1 and correctly
     * applies the set-management-type rule
     */
    #[test]
    fn test_builds_template_v1() {
        let temp_dir = TempDir::new("test_builds_template_v1").unwrap();
        let temp_dir = temp_dir.path().to_path_buf();
        let file_path = get_file_path(temp_dir);

        write_yaml_file(&file_path, EXAMPLE_TEMPLATE_YAML);
        let mut template =
            CircuitCreateTemplate::from_yaml_file(&file_path).expect("failed to parse template");

        let builders = template
            .into_builders()
            .expect("Error getting builders from templates");

        let circuit_create_builder = builders.create_circuit_builder();
        assert_eq!(
            circuit_create_builder.circuit_management_type(),
            Some("gameroom".to_string())
        );
    }

    fn get_file_path(mut temp_dir: PathBuf) -> String {
        temp_dir.push("example_template.yaml");
        let path = temp_dir.to_str().unwrap().to_string();
        path
    }

    fn write_yaml_file(file_path: &str, data: &[u8]) {
        let mut file = File::create(file_path).expect("Error creating test template yaml file.");

        file.write_all(data)
            .expect("Error writing example template yaml.");
    }
}
