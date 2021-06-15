// Copyright 2021 Cargill Incorporated
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

use crate::protos::ProtoConversionError;
use std::error::Error;
use std::fmt;

cfg_if! {
  if #[cfg(target_arch = "wasm32")] {
      use sabre_sdk::WasmSdkError as ContextError;
  } else {
      use sawtooth_sdk::processor::handler::ContextError;
  }
}

#[derive(Debug)]
pub enum PermissionCheckerError {
    /// Returned for an error originating at the TransactionContext.
    Context(ContextError),
    /// Returned for an invalid agent public key.
    InvalidPublicKey(String),
    /// Returned for an invalid role.
    InvalidRole(String),
    /// Returned for an error in the protobuf data.
    ProtoConversion(ProtoConversionError),
}

impl fmt::Display for PermissionCheckerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PermissionCheckerError::Context(ref e) => e.fmt(f),
            PermissionCheckerError::InvalidPublicKey(ref msg) => {
                write!(f, "InvalidPublicKey: {}", msg)
            }
            PermissionCheckerError::InvalidRole(ref msg) => write!(f, "InvalidRole: {}", msg),
            PermissionCheckerError::ProtoConversion(ref e) => e.fmt(f),
        }
    }
}

impl Error for PermissionCheckerError {
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            PermissionCheckerError::Context(_) => None,
            PermissionCheckerError::InvalidPublicKey(_) => None,
            PermissionCheckerError::InvalidRole(_) => None,
            PermissionCheckerError::ProtoConversion(ref e) => Some(e),
        }
    }
}

impl From<ContextError> for PermissionCheckerError {
    fn from(err: ContextError) -> PermissionCheckerError {
        PermissionCheckerError::Context(err)
    }
}

impl From<ProtoConversionError> for PermissionCheckerError {
    fn from(err: ProtoConversionError) -> PermissionCheckerError {
        PermissionCheckerError::ProtoConversion(err)
    }
}
