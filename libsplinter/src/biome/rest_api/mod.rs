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

//! Provides an API for managing Biome REST API endpoints
//!
//! Below is an example of building an instance of BiomeRestResourceManager and passing its resources
//! to a running instance of `RestApi`.
//!
//! ```no_run
//! use splinter::rest_api::{Resource, Method, RestApiBuilder, RestResourceProvider};
//! use splinter::biome::rest_api::{BiomeRestResourceManager, BiomeRestResourceManagerBuilder};
//! use splinter::database::{self, ConnectionPool};
//!
//! let connection_pool: ConnectionPool = database::create_connection_pool(
//!            "postgres://db_admin:db_password@0.0.0.0:5432/db",
//!        )
//!        .unwrap();
//!
//! let biome_rest_provider_builder: BiomeRestResourceManagerBuilder = Default::default();
//! let biome_rest_provider = biome_rest_provider_builder
//!             .with_user_store(connection_pool.clone())
//!             .build()
//!             .unwrap();
//!
//! RestApiBuilder::new()
//!     .add_resources(biome_rest_provider.resources())
//!     .with_bind("localhost:8080")
//!     .build()
//!     .unwrap()
//!     .run();
//! ```

#[cfg(feature = "rest-api-actix")]
mod actix;
mod config;
mod error;
mod resources;

use std::sync::Arc;

use crate::database::ConnectionPool;
use crate::rest_api::{Resource, RestResourceProvider};

#[cfg(all(feature = "biome-key-management", feature = "rest-api-actix"))]
use self::actix::key_management::make_key_management_route;

#[cfg(feature = "biome-key-management")]
use super::key_management::store::postgres::PostgresKeyStore;
use super::secrets::{AutoSecretManager, SecretManager};
use super::user::store::diesel::SplinterUserStore;

pub use config::{BiomeRestConfig, BiomeRestConfigBuilder};
pub use error::BiomeRestResourceManagerBuilderError;

#[cfg(all(feature = "biome-credentials", feature = "rest-api-actix"))]
use self::actix::{
    login::make_login_route,
    register::make_register_route,
    user::{make_list_route, make_user_routes},
};
#[cfg(feature = "biome-credentials")]
use super::credentials::store::diesel::SplinterCredentialsStore;

#[allow(unused_imports)]
use super::sessions::AccessTokenIssuer;

/// Manages Biome REST API endpoints
pub struct BiomeRestResourceManager {
    // Disable lint warning, for now this is only used if the biome-credentials feature is enabled
    #[allow(dead_code)]
    user_store: Arc<SplinterUserStore>,
    #[cfg(feature = "biome-key-management")]
    key_store: Arc<PostgresKeyStore>,
    // Disable lint warning, for now this is only used if the biome-credentials feature is enabled
    #[allow(dead_code)]
    rest_config: Arc<BiomeRestConfig>,
    #[allow(dead_code)]
    token_secret_manager: Arc<dyn SecretManager>,
    #[cfg(feature = "biome-credentials")]
    credentials_store: Option<Arc<SplinterCredentialsStore>>,
}

impl RestResourceProvider for BiomeRestResourceManager {
    fn resources(&self) -> Vec<Resource> {
        // This needs to be mutable if biome-credentials feature is enable
        #[allow(unused_mut)]
        let mut resources = Vec::new();

        #[cfg(feature = "biome-credentials")]
        match &self.credentials_store {
            Some(credentials_store) => {
                resources.push(make_register_route(
                    credentials_store.clone(),
                    self.user_store.clone(),
                    self.rest_config.clone(),
                ));
                resources.push(make_login_route(
                    credentials_store.clone(),
                    self.rest_config.clone(),
                    Arc::new(AccessTokenIssuer::new(self.token_secret_manager.clone())),
                ));
                resources.push(make_user_routes(
                    credentials_store.clone(),
                    self.user_store.clone(),
                ));
                resources.push(make_list_route(credentials_store.clone()));
            }
            None => {
                debug!(
                    "Credentials store not provided. Credentials REST API resources will not be'
                ' included in the biome endpoints."
                );
            }
        };
        #[cfg(all(feature = "biome-key-management", feature = "rest-api-actix"))]
        resources.push(make_key_management_route(
            self.rest_config.clone(),
            self.key_store.clone(),
            self.token_secret_manager.clone(),
        ));
        resources
    }
}

/// Builder for BiomeRestResourceManager
#[derive(Default)]
pub struct BiomeRestResourceManagerBuilder {
    user_store: Option<SplinterUserStore>,
    #[cfg(feature = "biome-key-management")]
    key_store: Option<PostgresKeyStore>,
    rest_config: Option<BiomeRestConfig>,
    token_secret_manager: Option<Arc<dyn SecretManager>>,
    #[cfg(feature = "biome-credentials")]
    credentials_store: Option<SplinterCredentialsStore>,
}

impl BiomeRestResourceManagerBuilder {
    /// Sets a UserStore for the BiomeRestResourceManager
    ///
    /// # Arguments
    ///
    /// * `pool`: ConnectionPool to database that will serve as backend for UserStore
    pub fn with_user_store(mut self, pool: ConnectionPool) -> BiomeRestResourceManagerBuilder {
        self.user_store = Some(SplinterUserStore::new(pool));
        self
    }

    /// Sets a KeyStore for the BiomeRestResourceManager
    ///
    /// # Arguments
    ///
    /// * `pool`: ConnectionPool to database that will serve as backend for KeyStore
    #[cfg(feature = "biome-key-management")]
    pub fn with_key_store(mut self, pool: ConnectionPool) -> BiomeRestResourceManagerBuilder {
        self.key_store = Some(PostgresKeyStore::new(pool));
        self
    }

    /// Sets a BiomeRestConfig for the BiomeRestResourceManager
    ///
    /// # Arguments
    ///
    /// * `config`: the BiomeRestConfig that will be used to configure the Biome resources
    pub fn with_rest_config(mut self, config: BiomeRestConfig) -> BiomeRestResourceManagerBuilder {
        self.rest_config = Some(config);
        self
    }

    #[cfg(feature = "biome-credentials")]
    /// Sets a CredentialsStore for the BiomeRestResourceManager
    ///
    /// # Arguments
    ///
    /// * `pool`: ConnectionPool to database that will serve as backend for CredentialsStore
    pub fn with_credentials_store(
        mut self,
        pool: ConnectionPool,
    ) -> BiomeRestResourceManagerBuilder {
        self.credentials_store = Some(SplinterCredentialsStore::new(pool));
        self
    }

    /// Sets a SecretManager for JWT tokens for the BiomeRestResourceManager
    ///
    /// # Arguments
    ///
    /// * `secret_manager`: the SecretManager to be used for fetching and generating secrets to
    ///   sign and verify JWT tokens
    pub fn set_token_secret_manager(
        mut self,
        secret_manager: impl SecretManager + 'static,
    ) -> BiomeRestResourceManagerBuilder {
        self.token_secret_manager = Some(Arc::new(secret_manager));
        self
    }

    /// Consumes the builder and returns a BiomeRestResourceManager
    pub fn build(self) -> Result<BiomeRestResourceManager, BiomeRestResourceManagerBuilderError> {
        let user_store = self.user_store.ok_or_else(|| {
            BiomeRestResourceManagerBuilderError::MissingRequiredField(
                "Missing user store".to_string(),
            )
        })?;
        #[cfg(feature = "biome-key-management")]
        let key_store = self.key_store.ok_or_else(|| {
            BiomeRestResourceManagerBuilderError::MissingRequiredField(
                "MissingKeyStore".to_string(),
            )
        })?;
        let rest_config = match self.rest_config {
            Some(config) => config,
            None => {
                debug!("Building BiomeRestResourceManager with default config.");
                BiomeRestConfigBuilder::default().build()?
            }
        };

        let token_secret_manager = self.token_secret_manager.unwrap_or_else(|| {
            debug!("Building BiomeRestResourceManager with default SecretManager.");
            Arc::new(AutoSecretManager::default())
        });

        Ok(BiomeRestResourceManager {
            user_store: Arc::new(user_store),
            #[cfg(feature = "biome-key-management")]
            key_store: Arc::new(key_store),
            rest_config: Arc::new(rest_config),
            token_secret_manager,
            #[cfg(feature = "biome-credentials")]
            credentials_store: match self.credentials_store {
                Some(credentials_store) => Some(Arc::new(credentials_store)),
                None => {
                    debug!("Building BiomeRestResourceManager without credentials store");
                    None
                }
            },
        })
    }
}
