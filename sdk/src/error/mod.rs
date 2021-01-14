// Copyright 2018-2021 Cargill Incorporated
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

//! Common set of basic errors used throughout the library.
//!
//! The errors in this module are intended to be used by themselves or as part of a more complex
//! error `enum`.
//!
//! # Examples
//!
//! ## Returning an Error from a Function
//!
//! A function may return an error such as `InternalError` by itself.
//!
//! ```
//! use std::fs;
//!
//! use grid_sdk::error::InternalError;
//!
//! fn check_path(path: &str) -> Result<bool, InternalError> {
//!     let metadata = fs::metadata(path).map_err(|e| InternalError::from_source(Box::new(e)))?;
//!     Ok(metadata.is_file())
//! }
//! ```
//!
//! ## Constructing Complex Errors
//!
//! Errors such as `InternalError` may be used to construct more complicated errors by defining
//! an `enum`.
//!
//! ```
//! use std::error;
//! use std::fmt;
//! use std::fs;
//!
//! use grid_sdk::error::InternalError;
//!
//! #[derive(Debug)]
//! enum MyError {
//!     InternalError(InternalError),
//!     MissingFilenameExtension,
//! }
//!
//! impl error::Error for MyError {}
//!
//! impl fmt::Display for MyError {
//!     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//!         match self {
//!             MyError::InternalError(e) => write!(f, "{}", e),
//!             MyError::MissingFilenameExtension => write!(f, "Missing filename extension"),
//!         }
//!     }
//! }
//!
//! fn check_path(path: &str) -> Result<bool, MyError> {
//!     match !path.ends_with(".md") {
//!         true => Err(MyError::MissingFilenameExtension),
//!         false => {
//!             let metadata = fs::metadata(path).map_err(|e| MyError::InternalError(InternalError::from_source(Box::new(e))))?;
//!             Ok(metadata.is_file())
//!         }
//!     }
//! }
//! ```

mod constraint_violation;
mod internal;
mod unavailable;

pub use constraint_violation::{ConstraintViolationError, ConstraintViolationType};
pub use internal::InternalError;
pub use unavailable::ResourceTemporarilyUnavailableError;
