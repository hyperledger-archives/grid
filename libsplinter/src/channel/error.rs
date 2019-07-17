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

#[derive(Debug, PartialEq)]
pub struct RecvError {
    pub error: String,
}

impl Error for RecvError {}

impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Receive Error: {}", self.error)
    }
}

#[derive(Debug, PartialEq)]
pub enum TryRecvError {
    Empty,
    Disconnected,
}

impl Error for TryRecvError {}

impl fmt::Display for TryRecvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TryRecvError::Empty => f.write_str("Unable to receive: channel is empty"),
            TryRecvError::Disconnected => {
                f.write_str("Unable to receive: channel has disconnected")
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum RecvTimeoutError {
    Timeout,
    Disconnected,
}

impl Error for RecvTimeoutError {}

impl fmt::Display for RecvTimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RecvTimeoutError::Timeout => f.write_str("Unable to receive: Timeout"),
            RecvTimeoutError::Disconnected => {
                f.write_str("Unable to receive: channel has disconnected")
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SendError {
    pub error: String,
}

impl Error for SendError {}

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Send Error: {}", self.error)
    }
}
