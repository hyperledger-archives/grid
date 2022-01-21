/*
 * Copyright 2019 - 2021 Cargill Incorporated
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

use std::ffi::CString;
use std::path::Path;

use super::error::CliError;

#[cfg(feature = "pike")]
pub mod agent;
#[cfg(feature = "database")]
pub mod database;
pub mod keygen;
#[cfg(feature = "location")]
pub mod location;
#[cfg(feature = "pike")]
pub mod organization;
#[cfg(feature = "product")]
pub mod product;
#[cfg(any(feature = "purchase-order"))]
pub mod purchase_order;
#[cfg(feature = "pike")]
pub mod role;
#[cfg(feature = "schema")]
pub mod schema;

#[cfg(any(feature = "purchase-order", feature = "product"))]
const DEFAULT_SCHEMA_DIR: &str = "/var/lib/grid/xsd";

fn chown(path: &Path, uid: u32, gid: u32) -> Result<(), CliError> {
    let pathstr = path
        .to_str()
        .ok_or_else(|| CliError::UserError(format!("Invalid path: {:?}", path)))?;
    let cpath = CString::new(pathstr).map_err(|err| CliError::UserError(format!("{}", err)))?;
    let result = unsafe { libc::chown(cpath.as_ptr(), uid, gid) };
    match result {
        0 => Ok(()),
        code => Err(CliError::UserError(format!(
            "Error chowning file {}: {}",
            pathstr, code
        ))),
    }
}
