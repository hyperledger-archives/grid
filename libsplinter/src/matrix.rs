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

use crate::transport::Connection;

/// MatrixLifeCycle trait abstracts out adding and removing connections to a
/// connection handler without requiring knowledge about sending or receiving messges.
pub trait MatrixLifeCycle: Clone + Send {
    fn add(&self, connection: Box<dyn Connection>) -> Result<usize, MatrixAddError>;
    fn remove(&self, id: usize) -> Result<Box<dyn Connection>, MatrixRemoveError>;
}

pub trait MatrixSender: Clone + Send {
    fn send(&self, id: usize, message: Vec<u8>) -> Result<(), MatrixSendError>;
}

#[derive(Debug)]
pub struct MatrixAddError {
    pub context: String,
    pub source: Option<Box<dyn Error + Send>>,
}

impl MatrixAddError {
    pub fn new(context: String, source: Option<Box<dyn Error + Send>>) -> Self {
        Self { context, source }
    }
}

impl Error for MatrixAddError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Some(ref err) = self.source {
            Some(&**err)
        } else {
            None
        }
    }
}

impl fmt::Display for MatrixAddError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref err) = self.source {
            write!(f, "{}: {}", self.context, err)
        } else {
            f.write_str(&self.context)
        }
    }
}

#[derive(Debug)]
pub struct MatrixRemoveError {
    pub context: String,
    pub source: Option<Box<dyn Error + Send>>,
}

impl MatrixRemoveError {
    pub fn new(context: String, source: Option<Box<dyn Error + Send>>) -> Self {
        Self { context, source }
    }
}

impl Error for MatrixRemoveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Some(ref err) = self.source {
            Some(&**err)
        } else {
            None
        }
    }
}

impl fmt::Display for MatrixRemoveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref err) = self.source {
            write!(f, "{}: {}", self.context, err)
        } else {
            f.write_str(&self.context)
        }
    }
}

#[derive(Debug)]
pub struct MatrixSendError {
    pub context: String,
    pub source: Option<Box<dyn Error + Send>>,
}

impl MatrixSendError {
    pub fn new(context: String, source: Option<Box<dyn Error + Send>>) -> Self {
        Self { context, source }
    }
}

impl Error for MatrixSendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Some(ref err) = self.source {
            Some(&**err)
        } else {
            None
        }
    }
}

impl fmt::Display for MatrixSendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref err) = self.source {
            write!(f, "{}: {}", self.context, err)
        } else {
            f.write_str(&self.context)
        }
    }
}
