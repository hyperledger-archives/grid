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

mod error;

use std::sync::Arc;

use crate::database::ConnectionPool;
use crate::rest_api::{Resource, RestResourceProvider};

use super::users::user_store::SplinterUserStore;

pub use error::BiomeRestResourceManagerBuilderError;

#[cfg(feature = "biome-credentials")]
use super::credentials::{
    credentials_store::SplinterCredentialsStore, rest_resources::make_register_route,
};

/// Manages Biome REST API endpoints
pub struct BiomeRestResourceManager {
    // This is only used if the biome-credentials feature is enabled
    #[allow(dead_code)]
    user_store: Arc<SplinterUserStore>,
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
            Some(credentials_store) => resources.push(make_register_route(
                credentials_store.clone(),
                self.user_store.clone(),
            )),
            None => {
                debug!(
                    "Credentials store not provided. Credentials REST API resources will not be'
                ' included in the biome endpoints."
                );
            }
        };
        resources
    }
}

/// Builder for BiomeRestResourceManager
#[derive(Default)]
pub struct BiomeRestResourceManagerBuilder {
    user_store: Option<SplinterUserStore>,
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

    /// Consumes the builder and returns a BiomeRestResourceManager
    pub fn build(self) -> Result<BiomeRestResourceManager, BiomeRestResourceManagerBuilderError> {
        let user_store = self.user_store.ok_or_else(|| {
            BiomeRestResourceManagerBuilderError::MissingRequiredField(
                "Missing user store".to_string(),
            )
        })?;

        Ok(BiomeRestResourceManager {
            user_store: Arc::new(user_store),
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
