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

use clap::ArgMatches;
use flexi_logger::ReconfigurationHandle;

use super::Action;
use crate::error::CliError;
use diesel::{connection::Connection as _, pg::PgConnection};
#[cfg(feature = "database-migrate-biome-credentials")]
use splinter::biome::credentials::database::run_migrations as run_biome_credentials_migrations;
#[cfg(feature = "database-migrate-biome-notifications")]
use splinter::biome::notifications::database::run_migrations as run_biome_notifications_migrations;
#[cfg(feature = "database-migrate-biome-user")]
use splinter::biome::user::database::run_migrations as run_biome_user_migrations;
use splinter::database::run_migrations as run_setup_migrations;

pub struct MigrateAction;

impl Action for MigrateAction {
    fn run<'a>(
        &mut self,
        arg_matches: Option<&ArgMatches<'a>>,
        _logger_handle: &mut ReconfigurationHandle,
    ) -> Result<(), CliError> {
        let url = if let Some(args) = arg_matches {
            args.value_of("connect")
                .unwrap_or("postgres://admin:admin@localhost:5432/splinterd")
        } else {
            "postgres://admin:admin@localhost:5432/splinterd"
        };

        let connection =
            PgConnection::establish(url).map_err(|err| CliError::DatabaseError(err.to_string()))?;

        run_setup_migrations(&connection).map_err(|err| {
            CliError::DatabaseError(format!("Unable to run Biome setup migrations: {}", err))
        })?;

        #[cfg(feature = "database-migrate-biome-user")]
        run_biome_user_migrations(&connection).map_err(|err| {
            CliError::DatabaseError(format!("Unable to run Biome users migrations: {}", err))
        })?;
        #[cfg(feature = "database-migrate-biome-credentials")]
        run_biome_credentials_migrations(&connection).map_err(|err| {
            CliError::DatabaseError(format!(
                "Unable to run Biome credentials migrations: {}",
                err
            ))
        })?;
        #[cfg(feature = "database-migrate-biome-notifications")]
        run_biome_notifications_migrations(&connection).map_err(|err| {
            CliError::DatabaseError(format!(
                "Unable to run Biome notifications migrations: {}",
                err
            ))
        })?;

        Ok(())
    }
}
