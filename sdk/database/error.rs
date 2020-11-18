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

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ConnectionError {
    pub context: String,
    pub source: Box<dyn Error>,
}

impl Error for ConnectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.source)
    }
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to connect to database: {}", self.context)
    }
}

#[derive(Debug)]
pub enum DatabaseError {
    ConnectionError {
        context: String,
        source: Box<dyn Error>,
    },
    MigrationError(Box<dyn Error>),
    QueryError(Box<dyn Error>),
}

impl Error for DatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DatabaseError::ConnectionError { source, .. } => Some(&**source),
            DatabaseError::MigrationError(e) => Some(&**e),
            DatabaseError::QueryError(e) => Some(&**e),
        }
    }
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DatabaseError::ConnectionError { context, source } => {
                write!(f, "Unable to connect to database: {}: {}", context, source)
            }
            DatabaseError::MigrationError(e) => write!(f, "Unable to migrate database: {}", e),
            DatabaseError::QueryError(e) => write!(f, "Database query failed: {}", e),
        }
    }
}

impl From<diesel::ConnectionError> for DatabaseError {
    fn from(err: diesel::ConnectionError) -> Self {
        DatabaseError::ConnectionError {
            context: "{}".to_string(),
            source: Box::new(err),
        }
    }
}

impl From<diesel_migrations::RunMigrationsError> for DatabaseError {
    fn from(err: diesel_migrations::RunMigrationsError) -> Self {
        DatabaseError::MigrationError(Box::new(err))
    }
}

impl From<diesel::result::Error> for DatabaseError {
    fn from(err: diesel::result::Error) -> Self {
        DatabaseError::QueryError(Box::new(err))
    }
}
