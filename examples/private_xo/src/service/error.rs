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

use std::error::Error;
use std::fmt;

use libsplinter::consensus::error::ProposalManagerError;

#[derive(Debug)]
pub struct ServiceError(pub String);

impl Error for ServiceError {
    fn cause(&self) -> Option<&dyn Error> {
        None
    }
    fn description(&self) -> &str {
        "A Service Error"
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<T> From<std::sync::mpsc::SendError<T>> for ServiceError {
    fn from(err: std::sync::mpsc::SendError<T>) -> Self {
        ServiceError(format!("Unable to send: {}", err))
    }
}

impl From<ServiceError> for ProposalManagerError {
    fn from(err: ServiceError) -> Self {
        ProposalManagerError::Internal(Box::new(err))
    }
}
