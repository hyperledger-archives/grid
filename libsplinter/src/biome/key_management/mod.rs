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

//! Provides an API for storing key pairs and associating them with users.

pub mod database;

#[cfg(feature = "rest-api")]
pub(in crate::biome) mod rest_resources;

pub(in crate::biome) mod store;

use database::postgres::models::KeyModel;

// Represents a public and private key pair
pub struct Key {
    public_key: String,
    encrypted_private_key: String,
    user_id: String,
    display_name: String,
}

impl Key {
    /// Creates a new Key
    ///
    /// # Arguments
    ///
    /// * `public_key`: The public key of the key pair.
    /// * `encrypted_private_key`: The private key of the key pair. This key
    ///     should be encrypted by the client before being sent to the key
    ///     management module
    /// * `user_id`: The identity of the Biome user who owns the key.
    /// * `display_name`: A human readable name for the key.
    ///
    pub fn new(
        public_key: &str,
        encrypted_private_key: &str,
        user_id: &str,
        display_name: &str,
    ) -> Self {
        Key {
            public_key: public_key.to_string(),
            encrypted_private_key: encrypted_private_key.to_string(),
            user_id: user_id.to_string(),
            display_name: display_name.to_string(),
        }
    }
}

impl From<KeyModel> for Key {
    fn from(key: KeyModel) -> Self {
        Key {
            public_key: key.public_key,
            encrypted_private_key: key.encrypted_private_key,
            user_id: key.user_id,
            display_name: key.display_name,
        }
    }
}
impl Into<KeyModel> for Key {
    fn into(self) -> KeyModel {
        KeyModel {
            public_key: self.public_key,
            encrypted_private_key: self.encrypted_private_key,
            user_id: self.user_id,
            display_name: self.display_name,
        }
    }
}
