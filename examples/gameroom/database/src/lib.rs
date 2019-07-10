/*
 * Copyright 2019 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

#[macro_use]
extern crate diesel;

pub mod error;
pub mod models;
pub mod schema;

use std::ops::Deref;

use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};

pub use crate::error::DatabaseError;

pub fn create_connection_pool(database_url: &str) -> Result<ConnectionPool, DatabaseError> {
    let connection_manager = ConnectionManager::<PgConnection>::new(database_url);
    Ok(ConnectionPool {
        pool: Pool::builder()
            .build(connection_manager)
            .map_err(|err| DatabaseError::ConnectionError(Box::new(err)))?,
    })
}

pub struct Connection(PooledConnection<ConnectionManager<PgConnection>>);

impl Deref for Connection {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
pub struct ConnectionPool {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl ConnectionPool {
    pub fn get(&self) -> Result<Connection, DatabaseError> {
        self.pool
            .get()
            .map(Connection)
            .map_err(|err| DatabaseError::ConnectionError(Box::new(err)))
    }
}
