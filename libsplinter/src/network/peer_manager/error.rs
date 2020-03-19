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

use std::{error, fmt};

#[derive(Debug, PartialEq)]
pub enum PeerManagerError {
    StartUpError(String),
    SendMessageError(String),
    RetryEndpoints(String),
}

impl error::Error for PeerManagerError {}

impl fmt::Display for PeerManagerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PeerManagerError::StartUpError(msg) => write!(f, "{}", msg),
            PeerManagerError::SendMessageError(msg) => write!(f, "{}", msg),
            PeerManagerError::RetryEndpoints(msg) => write!(
                f,
                "Error occured while trying to find new active endpoint: {}",
                msg
            ),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PeerRefAddError {
    InternalError(String),
    ReceiveError(String),
    AddError(String),
}

impl error::Error for PeerRefAddError {}

impl fmt::Display for PeerRefAddError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PeerRefAddError::InternalError(msg) => write!(f, "Received internal error: {}", msg),
            PeerRefAddError::ReceiveError(msg) => {
                write!(f, "Unable to receive response from PeerManager: {}", msg)
            }
            PeerRefAddError::AddError(msg) => write!(f, "Unable to add peer: {}", msg),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PeerRefRemoveError {
    InternalError(String),
    ReceiveError(String),
    RemoveError(String),
}

impl error::Error for PeerRefRemoveError {}

impl fmt::Display for PeerRefRemoveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PeerRefRemoveError::InternalError(msg) => write!(f, "Received internal error: {}", msg),
            PeerRefRemoveError::ReceiveError(msg) => {
                write!(f, "Unable to receive response from PeerManager: {}", msg)
            }
            PeerRefRemoveError::RemoveError(msg) => write!(f, "Unable to remove peer: {}", msg),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PeerRefUpdateError {
    InternalError(String),
    ReceiveError(String),
    UpdateError(String),
}

impl error::Error for PeerRefUpdateError {}

impl fmt::Display for PeerRefUpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PeerRefUpdateError::InternalError(msg) => write!(f, "Received internal error: {}", msg),
            PeerRefUpdateError::ReceiveError(msg) => {
                write!(f, "Unable to receive response from PeerManager: {}", msg)
            }
            PeerRefUpdateError::UpdateError(msg) => write!(f, "Unable to update peer: {}", msg),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PeerListError {
    InternalError(String),
    ReceiveError(String),
    ListError(String),
}

impl error::Error for PeerListError {}

impl fmt::Display for PeerListError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PeerListError::InternalError(msg) => write!(f, "Received internal error: {}", msg),
            PeerListError::ReceiveError(msg) => {
                write!(f, "Unable to receive response from PeerManager: {}", msg)
            }
            PeerListError::ListError(msg) => write!(f, "Unable to list peers: {}", msg),
        }
    }
}

#[derive(Debug)]
pub struct PeerUpdateError(pub String);

impl error::Error for PeerUpdateError {}

impl fmt::Display for PeerUpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to update peer, {}", self.0)
    }
}
