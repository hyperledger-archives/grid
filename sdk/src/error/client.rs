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

//! Module containing ClientError implementation.
use std::io;

/// An error which is returned from a Client Implementation.
///
/// This error is produced when a failure occurred within the function but the failure is due to an
/// internal implementation detail of the function. This generally means that there is no specific
/// information which can be returned that would help the caller of the function recover or
/// otherwise take action.
pub enum ClientError {
    IoError(io::Error),
    DaemonError(String),
}

impl From<io::Error> for ClientError {
    fn from(err: io::Error) -> Self {
        ClientError::IoError(err)
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(err: reqwest::Error) -> Self {
        ClientError::DaemonError(format!("Request Failed: {}", err))
    }
}
