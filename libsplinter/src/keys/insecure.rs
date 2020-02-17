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

//! The insecure keys module provides an allow-all permissions implemenation.
use super::{to_hex, KeyPermissionError, KeyPermissionManager};

/// A KeyPermissioManager that allows all keys access to the requested role.
pub struct AllowAllKeyPermissionManager;

impl KeyPermissionManager for AllowAllKeyPermissionManager {
    fn is_permitted(&self, public_key: &[u8], role: &str) -> Result<bool, KeyPermissionError> {
        debug!("Allowing {} access to {}", to_hex(public_key), role);
        Ok(true)
    }
}
