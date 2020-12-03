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

use crate::{
    agents::store::diesel::schema::{agent::dsl::*, role::dsl::role},
    commits::store::diesel::schema::{chain_record::dsl::*, commit::dsl::*},
    locations::store::diesel::schema::{location::dsl::*, location_attribute::dsl::*},
    organizations::store::diesel::schema::organization::dsl::*,
    products::store::diesel::schema::{product::dsl::*, product_property_value::dsl::*},
    schemas::store::diesel::schema::{
        grid_property_definition::dsl::grid_property_definition, grid_schema::dsl::*,
    },
    track_and_trace::store::diesel::schema::{
        associated_agent::dsl::*, property::dsl::*, proposal::dsl::*, record::dsl::*,
        reported_value::dsl::*, reporter::dsl::*,
    },
};

use diesel::RunQueryDsl;
#[cfg(feature = "sqlite")]
use diesel::{sqlite::SqliteConnection, Connection};

use crate::error::ResourceTemporarilyUnavailableError;
use crate::migrations::error::MigrationsError;

embed_migrations!("./src/migrations/diesel/sqlite/migrations");

/// Run database migrations to create Grid tables
///
/// # Arguments
///
/// * `conn` - Connection to database
///
#[cfg(all(feature = "sqlite", feature = "diesel"))]
pub fn run_migrations(conn: &SqliteConnection) -> Result<(), MigrationsError> {
    embedded_migrations::run(conn).map_err(|err| {
        MigrationsError::ResourceTemporarilyUnavailableError(
            ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
        )
    })?;

    info!("Successfully applied Grid migrations");

    Ok(())
}

#[cfg(all(feature = "sqlite", feature = "diesel"))]
pub fn clear_database(conn: &SqliteConnection) -> Result<(), MigrationsError> {
    conn.transaction::<_, MigrationsError, _>(|| {
        diesel::delete(agent).execute(conn)?;
        diesel::delete(role).execute(conn)?;
        diesel::delete(chain_record).execute(conn)?;
        diesel::delete(commit).execute(conn)?;
        diesel::delete(location).execute(conn)?;
        diesel::delete(location_attribute).execute(conn)?;
        diesel::delete(organization).execute(conn)?;
        diesel::delete(product).execute(conn)?;
        diesel::delete(product_property_value).execute(conn)?;
        diesel::delete(grid_schema).execute(conn)?;
        diesel::delete(grid_property_definition).execute(conn)?;
        diesel::delete(associated_agent).execute(conn)?;
        diesel::delete(property).execute(conn)?;
        diesel::delete(proposal).execute(conn)?;
        diesel::delete(record).execute(conn)?;
        diesel::delete(reported_value).execute(conn)?;
        diesel::delete(reporter).execute(conn)?;

        Ok(())
    })?;

    Ok(())
}
