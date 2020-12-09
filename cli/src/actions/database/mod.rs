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

#[cfg(feature = "sqlite")]
mod sqlite;

use diesel::{connection::Connection as _, pg::PgConnection};
use std::str::FromStr;

use grid_sdk::migrations::run_postgres_migrations;

use crate::error::CliError;

#[cfg(feature = "sqlite")]
use self::sqlite::sqlite_migrations;

pub fn run_migrations(database_url: &str) -> Result<(), CliError> {
    match ConnectionUri::from_str(&database_url).map_err(CliError::ActionError)? {
        ConnectionUri::Postgres(database_url) => {
            let connection = PgConnection::establish(&database_url).map_err(|err| {
                CliError::ActionError(format!(
                    "Failed to establish database connection to '{}': {}",
                    database_url, err
                ))
            })?;

            run_postgres_migrations(&connection).map_err(|err| {
                CliError::ActionError(format!("Unable to run Postgres migrations: {}", err))
            })?;
        }
        #[cfg(feature = "sqlite")]
        ConnectionUri::Sqlite(connection_string) => sqlite_migrations(connection_string)?,
    }

    Ok(())
}

/// The possible connection types and identifiers passed to the migrate command
pub enum ConnectionUri {
    #[cfg(feature = "postgres")]
    Postgres(String),
    #[cfg(feature = "sqlite")]
    Sqlite(String),
}

impl FromStr for ConnectionUri {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            #[cfg(feature = "postgres")]
            _ if s.starts_with("postgres://") => Ok(ConnectionUri::Postgres(s.into())),
            #[cfg(feature = "sqlite")]
            _ => Ok(ConnectionUri::Sqlite(s.into())),
            #[cfg(not(feature = "sqlite"))]
            _ => Err(format!("No compatible connection type: {}", s)),
        }
    }
}
