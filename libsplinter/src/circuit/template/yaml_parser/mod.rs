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

pub mod v1;

use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::CircuitTemplateError;

#[derive(Deserialize, Debug)]
struct TemplateVersionGuard {
    version: String,
}

#[derive(Deserialize, Debug)]
pub enum CircuitTemplate {
    V1(v1::CircuitCreateTemplate),
}

impl CircuitTemplate {
    pub fn load_from_file(file_path: &str) -> Result<Self, CircuitTemplateError> {
        let path = Path::new(file_path);
        if !path.is_file() {
            return Err(CircuitTemplateError::new(&format!(
                "File does not exist or is inaccessible: {}",
                file_path
            )));
        }
        let file = File::open(path).map_err(|err| {
            CircuitTemplateError::new_with_source("Error opening template file", err.into())
        })?;

        let template = Self::deserialize(file)?;
        Ok(template)
    }

    fn deserialize(mut reader: impl Read) -> Result<Self, CircuitTemplateError> {
        let mut data = Vec::new();
        reader.read_to_end(&mut data).map_err(|err| {
            CircuitTemplateError::new_with_source(
                "Error reading data from template file",
                err.into(),
            )
        })?;

        let version_guard: TemplateVersionGuard = serde_yaml::from_slice(&data)?;
        match version_guard.version.as_ref() {
            "v1" => {
                let template: v1::CircuitCreateTemplate = serde_yaml::from_slice(&data)?;
                Ok(Self::V1(template))
            }
            _ => Err(CircuitTemplateError::new(&format!(
                "Invalid template version: {}. The supported versions are: v1",
                version_guard.version
            ))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::v1::{Metadata, Value};
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    use tempdir::TempDir;

    const EXAMPLE_TEMPLATE_YAML: &[u8] = br##"version: v1
args:
    - name: admin-keys
      required: false
      default: $(a:SIGNER_PUB_KEY)
rules:
    set-management-type:
        management-type: "gameroom"
    create-services:
        service-type: 'scabbard'
        service-args:
        - key: 'admin-keys'
          value: [$(admin-keys)]
        - key: 'peer-services'
          value: '$(r:ALL_OTHER_SERVICES)'
        first-service: 'a000'
    set-metadata:
        encoding: json
        metadata:
            - key: "scabbard_admin_keys"
              value: [$(cs:ADMIN)]
            - key: "alias"
              value: "$(sm:gameroom_name)" "##;

    /*
     * Verifies load_template correctly loads a template version 1
     */
    #[test]
    fn test_parse_template_v1() {
        // create temp directoy
        let temp_dir = TempDir::new("test_parse_template_v1").unwrap();
        let temp_dir = temp_dir.path().to_path_buf();
        let file_path = get_file_path(temp_dir);

        write_yaml_file(&file_path, EXAMPLE_TEMPLATE_YAML);

        let template_version =
            CircuitTemplate::load_from_file(&file_path).expect("failed to load template");
        match template_version {
            CircuitTemplate::V1(template) => {
                assert_eq!(template.version(), "v1");
                let args = template.args();
                for arg in args {
                    assert_eq!(arg.name(), "admin-keys");
                    assert_eq!(arg.required(), false);
                    assert_eq!(
                        arg.default_value(),
                        Some(&"$(a:SIGNER_PUB_KEY)".to_string())
                    );

                    let create_services = template
                        .rules()
                        .create_services()
                        .expect("Did not parse create_services rule");
                    assert_eq!(create_services.service_type(), "scabbard");

                    assert_eq!(create_services.first_service(), "a000");

                    let service_args = create_services.service_args();
                    assert!(service_args.iter().any(|arg| arg.key() == "admin-keys"
                        && arg.value() == &Value::List(vec!["$(admin-keys)".to_string()])));
                    assert!(service_args.iter().any(|arg| arg.key() == "peer-services"
                        && arg.value() == &Value::Single("$(r:ALL_OTHER_SERVICES)".to_string())));
                }

                let management_type = template
                    .rules()
                    .set_management_type()
                    .expect("Management type was not deserialize correctly");
                assert_eq!(management_type.management_type(), "gameroom");

                let metadata = template
                    .rules()
                    .set_metadata()
                    .expect("Metadata was not deserialize correctly")
                    .metadata();

                match metadata {
                    Metadata::Json { metadata } => {
                        assert!(metadata.iter().any(|metadata| metadata.key()
                            == "scabbard_admin_keys"
                            && metadata.value() == &Value::List(vec!["$(cs:ADMIN)".to_string()])));
                        assert!(metadata.iter().any(|metadata| metadata.key() == "alias"
                            && metadata.value()
                                == &Value::Single("$(sm:gameroom_name)".to_string())));
                    }
                }
            }
        }
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
