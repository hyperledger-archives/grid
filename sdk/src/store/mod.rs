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

#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;

use std::str::FromStr;

#[cfg(feature = "diesel")]
use diesel::r2d2::{ConnectionManager, Pool};

/// An abstract factory for creating Grid stores backed by the same storage
pub trait StoreFactory {
    /// Get a new `CommitStore`
    fn get_grid_commit_store(&self) -> Box<dyn crate::commits::CommitStore>;
    /// Get a new `PikeStore`
    #[cfg(feature = "pike")]
    fn get_grid_pike_store(&self) -> Box<dyn crate::pike::PikeStore>;
    /// Get a new `LocationStore`
    #[cfg(feature = "location")]
    fn get_grid_location_store(&self) -> Box<dyn crate::locations::LocationStore>;
    /// Get a new `ProductStore`
    #[cfg(feature = "product")]
    fn get_grid_product_store(&self) -> Box<dyn crate::products::ProductStore>;
    /// Get a new `SchemaStore`
    #[cfg(feature = "schema")]
    fn get_grid_schema_store(&self) -> Box<dyn crate::schemas::SchemaStore>;
    /// Get a new `TrackAndTraceStore`
    #[cfg(feature = "track-and-trace")]
    fn get_grid_track_and_trace_store(&self)
        -> Box<dyn crate::track_and_trace::TrackAndTraceStore>;
    #[cfg(feature = "batch-store")]
    fn get_batch_store(&self) -> Box<dyn crate::batches::BatchStore>;
}

/// Creates a `StoreFactory` backed by the given connection
///
/// # Arguments
///
/// * `connection_uri` - The identifier of the storage connection that will be used by all stores
///   created by the resulting factory
pub fn create_store_factory(
    connection_uri: &ConnectionUri,
) -> Result<Box<dyn StoreFactory>, StoreFactoryCreationError> {
    // disable clippy warning caused for some combinations of features
    // this warning is intended to reduce "needless complexity" but
    // adding blocks for every combination has the opposite effect
    #[allow(clippy::match_single_binding)]
    match connection_uri {
        #[cfg(feature = "postgres")]
        ConnectionUri::Postgres(url) => {
            let connection_manager = ConnectionManager::<diesel::pg::PgConnection>::new(url);
            let pool = Pool::builder().build(connection_manager).map_err(|err| {
                StoreFactoryCreationError(format!("Failed to build connection pool: {}", err))
            })?;
            Ok(Box::new(postgres::PgStoreFactory::new(pool)))
        }
        #[cfg(feature = "sqlite")]
        ConnectionUri::Sqlite(conn_str) => {
            let connection_manager =
                ConnectionManager::<diesel::sqlite::SqliteConnection>::new(conn_str);
            let mut pool_builder = Pool::builder();
            // A new database is created for each connection to the in-memory SQLite
            // implementation; to ensure that the resulting stores will operate on the same
            // database, only one connection is allowed.
            if conn_str == ":memory:" {
                pool_builder = pool_builder.max_size(1);
            }
            let pool = pool_builder.build(connection_manager).map_err(|err| {
                StoreFactoryCreationError(format!("Failed to build connection pool: {}", err))
            })?;
            Ok(Box::new(sqlite::SqliteStoreFactory::new(pool)))
        }
        #[cfg(all(not(feature = "sqlite"), not(feature = "postgres")))]
        _ => Err(StoreFactoryCreationError(
            "No valid database connection URI".to_string(),
        )),
    }
}

/// Errors raised by trying to create a `StoreFactory`
#[derive(Debug)]
pub struct StoreFactoryCreationError(pub String);

impl std::error::Error for StoreFactoryCreationError {}

impl std::fmt::Display for StoreFactoryCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Unable to create store factory: {}", self.0)
    }
}

/// The possible connection types and identifiers for a `StoreFactory`
#[derive(Clone)]
pub enum ConnectionUri {
    #[cfg(feature = "postgres")]
    Postgres(String),
    #[cfg(feature = "sqlite")]
    Sqlite(String),
}

impl FromStr for ConnectionUri {
    type Err = ParseConnectionUriError;

    // disable clippy warning caused for some combinations of features
    // this warning is intended to reduce "needless complexity" but
    // adding blocks for every combination has the opposite effect
    #[allow(clippy::match_single_binding)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            #[cfg(feature = "postgres")]
            _ if s.starts_with("postgres://") => Ok(ConnectionUri::Postgres(s.into())),
            #[cfg(feature = "sqlite")]
            _ => Ok(ConnectionUri::Sqlite(s.into())),
            #[cfg(not(feature = "sqlite"))]
            _ => Err(ParseConnectionUriError(format!(
                "No compatible connection type: {}",
                s
            ))),
        }
    }
}

/// Errors raised by trying to parse a `ConnectionUri`
#[derive(Debug)]
pub struct ParseConnectionUriError(pub String);

impl std::error::Error for ParseConnectionUriError {}

impl std::fmt::Display for ParseConnectionUriError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Unable to parse connection URI from string: {}", self.0)
    }
}
