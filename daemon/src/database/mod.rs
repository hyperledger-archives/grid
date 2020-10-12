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

pub mod error;

use std::ops::Deref;

use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};

pub use super::database::error::{ConnectionError, DatabaseError};

pub struct Connection(PooledConnection<ConnectionManager<PgConnection>>);

impl Deref for Connection {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct ConnectionPool<C: diesel::Connection + 'static> {
    pub pool: Pool<ConnectionManager<C>>,
}

impl<C: diesel::Connection> ConnectionPool<C> {
    pub fn new(database_url: &str) -> Result<Self, DatabaseError> {
        Ok(ConnectionPool {
            pool: Pool::new(ConnectionManager::<C>::new(database_url)).map_err(|err| {
                DatabaseError::ConnectionError {
                    context: "Failed to build connection pool".to_string(),
                    source: Box::new(err),
                }
            })?,
        })
    }

    pub fn get(&self) -> Result<PooledConnection<ConnectionManager<C>>, DatabaseError> {
        self.pool
            .get()
            .map_err(|err| DatabaseError::ConnectionError {
                context: "Failed to get Connection from connection pool".to_string(),
                source: Box::new(err),
            })
    }
}

impl<C: diesel::Connection> Clone for ConnectionPool<C> {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}
