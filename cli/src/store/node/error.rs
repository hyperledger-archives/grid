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

use std::error::Error;
use std::fmt;
use std::io::Error as IoError;

use serde_yaml::Error as SerdeError;

#[derive(Debug)]
pub enum NodeStoreError {
    NotFound(String),
    IoError(IoError),
    SerdeError(SerdeError),
}

impl Error for NodeStoreError {}

impl fmt::Display for NodeStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeStoreError::NotFound(msg) => write!(f, "Node not found: {}", msg),
            NodeStoreError::IoError(err) => {
                write!(f, "Node store encountered an IO error: {}", err)
            }
            NodeStoreError::SerdeError(err) => write!(
                f,
                "Node store encountered and serialization/deserialization error  {}",
                err
            ),
        }
    }
}

impl From<IoError> for NodeStoreError {
    fn from(err: IoError) -> NodeStoreError {
        NodeStoreError::IoError(err)
    }
}

impl From<SerdeError> for NodeStoreError {
    fn from(err: SerdeError) -> NodeStoreError {
        NodeStoreError::SerdeError(err)
    }
}
