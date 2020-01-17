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

#[cfg(feature = "admin-keygen")]
pub mod admin;
pub mod agents;
pub mod database;
pub mod keygen;
pub mod organizations;
pub mod products;
pub mod schemas;

use std::fs::File;
use std::io::Write;

use crate::error::CliError;

/// Write the given hex string to the given file, appending a newline at the end.
fn write_hex_to_file(hex: &str, file: &mut File) -> Result<(), CliError> {
    writeln!(file, "{}", hex)?;
    Ok(())
}
