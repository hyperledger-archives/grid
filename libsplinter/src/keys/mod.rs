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

//! Public key information and role-based permissions.
//!
//! The `keys` module provides interfaces for managing key information and role-based permissions.
//!
//! Key information, accessed via the `KeyRegistry` interface, is the link between a public key and
//! its associated splinter node.  This can provide context about a key used to sign messages.
//! Additional, optional meta data can be associated with the public key as well.
//!
//! Key permissions, accessed via the `KeyPermissionManager` interface, are queried through a simple
//! role-based access system.  The underlying implementation determines how those values are set
//! and modified.

mod error;

use std::collections::HashMap;
use std::fmt::{self, Write};

pub use error::{KeyPermissionError, KeyRegistryError};

/// Information associated with a public key.
///
/// This struct contains information related to a public key, with the most specific information
/// pertaining to the associated splinter node.  
///
/// It also provides metadata about the key, that maybe provided to the registry for
/// application-specific details.  For example, the name of the person or organization of the key.
pub struct KeyInfo {
    public_key: Vec<u8>,
    associated_node_id: String,
    metadata: HashMap<String, String>,
}

impl KeyInfo {
    /// Build a key info
    ///
    /// ```
    /// # use libsplinter::keys::KeyInfo;
    ///
    /// let key = KeyInfo::builder(b"some pub key".to_vec(), "my node".into())
    ///     .with_metadata("username", "Alice")
    ///     .with_metadata("organization", "ACME, Corp")
    ///     .build();
    ///
    /// assert_eq!(b"some pub key", key.public_key());
    /// assert_eq!("my node", key.associated_node_id());
    /// assert_eq!(Some(&"Alice".into()), key.get_metadata("username"));
    /// assert_eq!(Some(&"ACME, Corp".into()), key.get_metadata("organization"));
    /// ```
    pub fn builder(public_key: Vec<u8>, associated_node_id: String) -> KeyInfoBuilder {
        KeyInfoBuilder {
            public_key,
            associated_node_id,
            metadata: HashMap::default(),
        }
    }

    /// The public key.
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    /// The associated splinter node.
    ///
    /// This could be thought of as the "home node" of this public key.
    pub fn associated_node_id(&self) -> &str {
        &self.associated_node_id
    }

    /// Get a piece of metadata for the given key.
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

impl fmt::Debug for KeyInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            r#"KeyInfo {{ public_key: "{}", associated_node_id: {:?}, metadata: {:?} }}"#,
            to_hex(&self.public_key),
            &self.associated_node_id,
            &self.metadata
        )
    }
}

/// Builder for creating KeyInfo instances.
pub struct KeyInfoBuilder {
    public_key: Vec<u8>,
    associated_node_id: String,
    metadata: HashMap<String, String>,
}

impl KeyInfoBuilder {
    /// Add a meta data entry.
    pub fn with_metadata<S: Into<String>>(mut self, key: S, value: S) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Build the key info
    pub fn build(self) -> KeyInfo {
        KeyInfo {
            public_key: self.public_key,
            associated_node_id: self.associated_node_id,
            metadata: self.metadata,
        }
    }
}

type KeyRegistryResult<T> = Result<T, KeyRegistryError>;

/// A registry of public key information.
///
/// The key registry provides an interface for storing and retrieving key information. Key
/// information helps to tie a public key to a particular splinter node, as well as associating
/// application metadata with the public key.
pub trait KeyRegistry {
    /// Save a public key and its information.
    ///
    /// # Errors
    ///
    /// Returns a `KeyRegistryError` if the underling implementation could not save the key
    /// information.
    fn save_key(&mut self, key_info: KeyInfo) -> KeyRegistryResult<()>;

    /// Delete a public key and its information.
    ///
    /// Returns the existing key information, if it exists.
    ///
    /// # Errors
    ///
    /// Returns a `KeyRegistryError` if the underling implementation could not delete the key
    /// information.
    fn delete_key(&mut self, public_key: &[u8]) -> KeyRegistryResult<Option<KeyInfo>>;

    /// Return a public key and its information.
    ///
    /// Returns the key information, if it exists.
    ///
    /// # Errors
    ///
    /// Returns a `KeyRegistryError` if the underling implementation could not retrieve the key
    /// information.
    fn get_key(&self, public_key: &[u8]) -> KeyRegistryResult<Option<KeyInfo>>;

    /// Return an iterator over all keys in the registry.
    ///
    /// This returns an iterator over the key registry.  The iterator allows the underlying
    /// implementation to stream the results in a lazy fashion, if needed.
    ///
    /// # Errors
    ///
    /// Returns a `KeyRegistryError` if the underling implementation could not provide the
    /// iterator.
    fn keys<'a>(&'a self) -> KeyRegistryResult<Box<dyn Iterator<Item = KeyInfo> + 'a>>;
}

type KeyPermissionResult<T> = Result<T, KeyPermissionError>;

/// Manages role-based permissions associated with public keys.
///
/// The KeyPermissionManager provides an interface for providing details on whether or not a public
/// key has permissions to act in specific roles.
///
/// Note: the underlying implementation determines how those values are set and modified - these
/// operations are not exposed via this interface.
pub trait KeyPermissionManager {
    /// Checks to see if a public key is permitted for the given role.
    ///
    /// # Errors
    ///
    /// Returns a `KeyPermissionError` if the underling implementation encountered an error while
    /// checking the permissions.
    fn is_permitted(&self, public_key: &[u8], role: &str) -> KeyPermissionResult<bool>;
}

fn to_hex(bytes: &[u8]) -> String {
    let mut buf = String::new();
    for b in bytes {
        write!(&mut buf, "{:02x}", b).expect("Unable to write to string");
    }

    buf
}
