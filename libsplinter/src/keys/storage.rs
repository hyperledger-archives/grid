// Copyright 2019 Cargill Incorporated
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

//! Provides a storage-backed KeyRegistry

use std::collections::BTreeMap;
use std::convert::TryInto;

use serde_derive::{Deserialize, Serialize};

use crate::hex::{parse_hex, to_hex};
use crate::storage::get_storage;

use super::{KeyInfo, KeyRegistry, KeyRegistryError};

/// A read-only KeyRegistry backed by the `storage` module.
///
/// This KeyRegistry is backed by the storage module, and therefore supports the same formats
/// available.
pub struct StorageKeyRegistry {
    storage_location: String,
    persisted_key_registry: PersistedKeyRegistry,
}

impl KeyRegistry for StorageKeyRegistry {
    fn save_key(&mut self, key_info: KeyInfo) -> Result<(), KeyRegistryError> {
        self.persisted_key_registry.add_key(key_info)?;
        self.write_key_registry()
    }

    fn save_keys(&mut self, key_infos: Vec<KeyInfo>) -> Result<(), KeyRegistryError> {
        for key_info in key_infos.into_iter() {
            self.persisted_key_registry.add_key(key_info)?;
        }
        self.write_key_registry()
    }

    fn delete_key(&mut self, _public_key: &[u8]) -> Result<Option<KeyInfo>, KeyRegistryError> {
        Err(KeyRegistryError {
            context: "Operation not supported".into(),
            source: None,
        })
    }

    fn get_key(&self, public_key: &[u8]) -> Result<Option<KeyInfo>, KeyRegistryError> {
        self.persisted_key_registry
            .keys
            .get(&to_hex(public_key))
            .cloned()
            .map(PersistedKeyInfo::try_into)
            .transpose()
    }

    fn keys<'a>(&'a self) -> Result<Box<dyn Iterator<Item = KeyInfo> + 'a>, KeyRegistryError> {
        Ok(Box::new(
            self.persisted_key_registry
                .keys
                .iter()
                .map(|(_, key_info)| key_info.clone().try_into())
                .filter(Result::is_ok)
                .map(Result::unwrap),
        ))
    }
}

impl StorageKeyRegistry {
    /// Constructs a new StorageKeyRegistry using the given location.
    ///
    /// # Errors
    ///
    /// Returns a `KeyRegistryError` if the persisted registry fails to load.
    pub fn new(storage_location: String) -> Result<Self, KeyRegistryError> {
        let persisted_key_registry = get_storage(&storage_location, PersistedKeyRegistry::default)
            .map_err(|err: String| KeyRegistryError {
                context: format!("unable to load storage: {}", err),
                source: None,
            })?
            .read()
            .clone();

        Ok(Self {
            storage_location,
            persisted_key_registry,
        })
    }

    pub fn storage_location(&self) -> &str {
        &self.storage_location
    }

    fn write_key_registry(&self) -> Result<(), KeyRegistryError> {
        // Replace stored key_registry with the current key registry
        let mut storage = get_storage(self.storage_location(), || {
            self.persisted_key_registry.clone()
        })
        .map_err(|err: String| KeyRegistryError {
            context: format!("unable to load key registry: {}", err),
            source: None,
        })?;

        // when this is dropped the new state will be written to storage
        **storage.write() = self.persisted_key_registry.clone();
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct PersistedKeyRegistry {
    #[serde(flatten)]
    keys: BTreeMap<String, PersistedKeyInfo>,
}

impl PersistedKeyRegistry {
    pub fn add_key(&mut self, key_info: KeyInfo) -> Result<(), KeyRegistryError> {
        let hex_key = to_hex(key_info.public_key());

        let persisted_key_info = PersistedKeyInfo {
            public_key: hex_key.clone(),
            associated_node_id: key_info.associated_node_id().into(),
            metadata: key_info
                .metadata()
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<BTreeMap<String, String>>(),
        };
        self.keys.insert(hex_key, persisted_key_info);
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PersistedKeyInfo {
    public_key: String,
    associated_node_id: String,

    #[serde(default = "BTreeMap::new")]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    metadata: BTreeMap<String, String>,
}

impl TryInto<KeyInfo> for PersistedKeyInfo {
    type Error = KeyRegistryError;

    fn try_into(self) -> Result<KeyInfo, Self::Error> {
        let mut builder = KeyInfo::builder(
            parse_hex(&self.public_key).map_err(|err| KeyRegistryError {
                context: format!("Unable to parse public key: {}", self.public_key),
                source: Some(Box::new(err)),
            })?,
            self.associated_node_id,
        );

        for (key, value) in self.metadata.into_iter() {
            builder = builder.with_metadata(key, value);
        }

        Ok(builder.build())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use tempdir::TempDir;

    use super::*;

    /// Test the reading of persisted key registry information via the KeyRegistry trait.
    ///
    /// 1. store two keys, one with metadata, one without
    /// 2. test that they are retrieved successfully
    #[test]
    fn test_read_only_storage() {
        let temp_dir = TempDir::new("test_write_circuit").unwrap();
        let mut temp_dir_path = temp_dir.path().to_path_buf();

        let mut keys = BTreeMap::new();
        keys.insert(
            "abcdef".to_string(),
            make_key_info("abcdef", "my-node", vec![]),
        );
        keys.insert(
            "012345".to_string(),
            make_key_info(
                "012345",
                "other-node",
                vec![("meta1".into(), "value1".into())],
            ),
        );
        let persistable = PersistedKeyRegistry { keys };

        temp_dir_path.push("key_reg.yaml");

        let mut file = File::create(&temp_dir_path).expect("unable to create file");
        file.write_all(
            serde_yaml::to_string(&persistable)
                .expect("Could not write yaml")
                .as_bytes(),
        )
        .expect("could not write file");

        let registry = StorageKeyRegistry::new(
            temp_dir_path
                .to_str()
                .expect("could not create path str")
                .to_string(),
        )
        .expect("could not load file");

        let public_key1 = parse_hex("abcdef").expect("unable to parse abcdef");
        let public_key2 = parse_hex("012345").expect("unable to parse 012345");

        let key_info = registry
            .get_key(&public_key1)
            .expect("unable to get key info")
            .expect("Key info for abcdef was none");

        assert_eq!(&public_key1[..], key_info.public_key());
        assert_eq!("my-node", key_info.associated_node_id());

        let key_info = registry
            .get_key(&public_key2)
            .expect("unable to get key info")
            .expect("Key info for 012345 was none");

        assert_eq!(&public_key2[..], key_info.public_key());
        assert_eq!("other-node", key_info.associated_node_id());
        assert_eq!(Some(&"value1".into()), key_info.get_metadata("meta1"));
    }

    fn make_key_info(
        public_key: &str,
        node_id: &str,
        metadata: Vec<(String, String)>,
    ) -> PersistedKeyInfo {
        PersistedKeyInfo {
            public_key: public_key.into(),
            associated_node_id: node_id.into(),
            metadata: metadata.into_iter().collect(),
        }
    }
}
