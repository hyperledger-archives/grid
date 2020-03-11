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

use std::convert::TryFrom;

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
            CircuitTemplate::V1(template) => Ok(Self::try_from(template)?),
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

    pub fn set_argument_value(
        &mut self,
        key: &str,
        value: &str,
    ) -> Result<(), CircuitTemplateError> {
        let name = key.to_lowercase();
        let (index, mut arg) = self
            .arguments
            .iter()
            .enumerate()
            .find_map(|(index, arg)| {
                if arg.name() == name {
                    Some((index, arg.clone()))
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                CircuitTemplateError::new(&format!(
                    "Argument {} is not defined in the template",
                    key
                ))
            })?;
        arg.set_user_value(value);
        self.arguments[index] = arg;
        Ok(())
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn arguments(&self) -> &[RuleArgument] {
        &self.arguments
    }

    pub fn rules(&self) -> &Rules {
        &self.rules
    }
}

impl TryFrom<v1::CircuitCreateTemplate> for CircuitCreateTemplate {
    type Error = CircuitTemplateError;
    fn try_from(create_circuit_template: v1::CircuitCreateTemplate) -> Result<Self, Self::Error> {
        Ok(CircuitCreateTemplate {
            version: create_circuit_template.version().to_string(),
            arguments: create_circuit_template
                .args()
                .to_owned()
                .into_iter()
                .map(RuleArgument::try_from)
                .collect::<Result<_, CircuitTemplateError>>()?,
            rules: Rules::from(create_circuit_template.rules().clone()),
        })
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
    - name: $(a:ADMIN_KEYS)
      required: false
      default: $(a:SIGNER_PUB_KEY)
    - name: $(a:NODES)
      required: true
    - name: $(a:SIGNER_PUB_KEY)
      required: false
    - name: $(a:GAMEROOM_NAME)
      required: true
rules:
    set-management-type:
        management-type: "gameroom"
    create-services:
        service-type: 'scabbard'
        service-args:
        - key: 'admin-keys'
          value: [$(a:ADMIN_KEYS)]
        - key: 'peer-services'
          value: '$(r:ALL_OTHER_SERVICES)'
        first-service: 'a000'
    set-metadata:
        encoding: json
        metadata:
            - key: "scabbard_admin_keys"
              value: ["$(a:ADMIN_KEYS)"]
            - key: "alias"
              value: "$(a:GAMEROOM_NAME)" "##;

    /*
     * Verifies that Builders can be parsed from template v1 and correctly
     * applies the set-management-type, create-services and set-metadata rules correctly
     */
    #[test]
    fn test_builds_template_v1() {
        let temp_dir = TempDir::new("test_builds_template_v1").unwrap();
        let temp_dir = temp_dir.path().to_path_buf();
        let file_path = get_file_path(temp_dir);

        write_yaml_file(&file_path, EXAMPLE_TEMPLATE_YAML);
        let mut template =
            CircuitCreateTemplate::from_yaml_file(&file_path).expect("failed to parse template");

        template
            .set_argument_value("nodes", "alpha-node-000,beta-node-000")
            .expect("Error setting argument");
        template
            .set_argument_value("signer_pub_key", "signer_key")
            .expect("Error setting argument");
        template
            .set_argument_value("gameroom_name", "my gameroom")
            .expect("Error setting argument");

        let builders = template
            .into_builders()
            .expect("Error getting builders from templates");

        let circuit_create_builder = builders.create_circuit_builder();
        assert_eq!(
            circuit_create_builder.circuit_management_type(),
            Some("gameroom".to_string())
        );

        let metadata = String::from_utf8(
            circuit_create_builder
                .application_metadata()
                .expect("Application metadata is not set"),
        )
        .expect("Failed to parse metadata to string");
        assert_eq!(
            metadata,
            "{\"scabbard_admin_keys\":[\"signer_key\"],\"alias\":\"my gameroom\"}"
        );

        let service_builders = builders.service_builders();
        let service_alpha_node = service_builders
            .iter()
            .find(|service| service.allowed_nodes() == Some(vec!["alpha-node-000".to_string()]))
            .expect("service builder for alpha-node was not created correctly");

        assert_eq!(service_alpha_node.service_id(), Some("a000".to_string()));
        assert_eq!(
            service_alpha_node.service_type(),
            Some("scabbard".to_string())
        );

        let alpha_service_args = service_alpha_node
            .arguments()
            .expect("service for alpha node has no arguments set");
        assert!(alpha_service_args
            .iter()
            .any(|(key, value)| key == "admin-keys" && value == "[\"signer_key\"]"));
        assert!(alpha_service_args
            .iter()
            .any(|(key, value)| key == "peer-services" && value == "[\"a001\"]"));

        let service_beta_node = service_builders
            .iter()
            .find(|service| service.allowed_nodes() == Some(vec!["beta-node-000".to_string()]))
            .expect("service builder for beta-node was not created correctly");

        assert_eq!(service_beta_node.service_id(), Some("a001".to_string()));
        assert_eq!(
            service_beta_node.service_type(),
            Some("scabbard".to_string())
        );

        let beta_service_args = service_beta_node
            .arguments()
            .expect("service for beta node has no arguments set");
        assert!(beta_service_args
            .iter()
            .any(|(key, value)| key == "admin-keys" && value == "[\"signer_key\"]"));
        assert!(beta_service_args
            .iter()
            .any(|(key, value)| key == "peer-services" && value == "[\"a000\"]"));
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
