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

use std::fs::{self, File, OpenOptions};
use std::io::{Error as IoError, ErrorKind};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{DefaultStoreError, DefaultValue, DefaultValueStore};

const DEFAULT_FILE_NAME: &str = "circuit_default_values.yaml";
const DEFAULTS_PATH: &str = ".splinter";

#[derive(Serialize, Deserialize)]
pub struct SerdeDefaultValue {
    key: String,
    value: String,
}

impl Into<DefaultValue> for SerdeDefaultValue {
    fn into(self) -> DefaultValue {
        DefaultValue {
            key: self.key,
            value: self.value,
        }
    }
}

impl From<&DefaultValue> for SerdeDefaultValue {
    fn from(default_value: &DefaultValue) -> SerdeDefaultValue {
        SerdeDefaultValue {
            key: default_value.key.clone(),
            value: default_value.value.clone(),
        }
    }
}

pub struct FileBackedDefaultStore {
    file_name: String,
}

impl Default for FileBackedDefaultStore {
    fn default() -> Self {
        FileBackedDefaultStore {
            file_name: DEFAULT_FILE_NAME.to_owned(),
        }
    }
}

impl DefaultValueStore for FileBackedDefaultStore {
    fn set_default_value(&self, new_default_value: &DefaultValue) -> Result<(), DefaultStoreError> {
        let mut defaults = self.load()?;
        let existing_default_index = defaults.iter().enumerate().find_map(|(index, default)| {
            if default.key == new_default_value.key() {
                Some(index)
            } else {
                None
            }
        });
        if let Some(index) = existing_default_index {
            defaults.remove(index);
        }

        defaults.push(SerdeDefaultValue::from(new_default_value));

        self.save(&defaults)?;

        Ok(())
    }

    fn unset_default_value(&self, default_key: &str) -> Result<(), DefaultStoreError> {
        let mut all_defaults = self.load()?;

        let key_index = all_defaults
            .iter()
            .enumerate()
            .find_map(|(index, default)| {
                if default.key == default_key {
                    Some(index)
                } else {
                    None
                }
            });

        if let Some(index) = key_index {
            all_defaults.remove(index);
            self.save(&all_defaults)?;
        } else {
            return Err(DefaultStoreError::NotSet(format!(
                "Default value for {} not found",
                default_key
            )));
        }

        Ok(())
    }

    fn list_default_values(&self) -> Result<Vec<DefaultValue>, DefaultStoreError> {
        let serde_defaults = self.load()?;
        let defaults = serde_defaults
            .into_iter()
            .map(|default_value| default_value.into())
            .collect::<Vec<DefaultValue>>();
        Ok(defaults)
    }

    fn get_default_value(&self, key: &str) -> Result<Option<DefaultValue>, DefaultStoreError> {
        let defaults = self.load()?;

        let default_value = defaults.into_iter().find_map(|default_value| {
            if default_value.key == key {
                return Some(default_value.into());
            }
            None
        });

        Ok(default_value)
    }
}

impl FileBackedDefaultStore {
    fn load(&self) -> Result<Vec<SerdeDefaultValue>, DefaultStoreError> {
        let file = open_file(false, &self.file_name)?;

        if file.metadata()?.len() == 0 {
            return Ok(vec![]);
        }

        let default_values = serde_yaml::from_reader(file)?;
        Ok(default_values)
    }

    fn save(&self, default_values: &[SerdeDefaultValue]) -> Result<(), DefaultStoreError> {
        let temp_file_name = format!("{}.tmp", self.file_name);
        let file = open_file(true, &temp_file_name)?;

        serde_yaml::to_writer(file, &default_values)?;

        let temp_file_path = build_file_path(&temp_file_name)?;
        let perm_file_path = build_file_path(&self.file_name)?;
        fs::rename(temp_file_path, perm_file_path)?;
        Ok(())
    }
}

fn build_file_path(file_name: &str) -> Result<PathBuf, DefaultStoreError> {
    let mut path = get_file_path()?;
    path.push(file_name);
    Ok(path)
}

fn open_file(truncate: bool, file_name: &str) -> Result<File, DefaultStoreError> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(truncate)
        .open(&build_file_path(file_name)?)?;
    Ok(file)
}

fn get_file_path() -> Result<PathBuf, DefaultStoreError> {
    let mut path = dirs::home_dir().ok_or_else(|| {
        let err = IoError::new(ErrorKind::NotFound, "Home directory not found");
        DefaultStoreError::IoError(err)
    })?;
    path.push(DEFAULTS_PATH);
    fs::create_dir_all(&path)?;
    Ok(path)
}
