/*
 * Copyright 2019 Cargill Incorporated
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

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AppAuthHandlerError {
    RequestError(String),
    IOError(std::io::Error),
    DeserializationError(Box<dyn Error + Send>),
}

impl Error for AppAuthHandlerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppAuthHandlerError::RequestError(_) => None,
            AppAuthHandlerError::IOError(err) => Some(err),
            AppAuthHandlerError::DeserializationError(err) => Some(&**err),
        }
    }
}

impl fmt::Display for AppAuthHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppAuthHandlerError::RequestError(msg) => write!(f, "Failed to build request, {}", msg),
            AppAuthHandlerError::IOError(msg) => write!(f, "An I/O error occurred: {}", msg),
            AppAuthHandlerError::DeserializationError(msg) => {
                write!(f, "Failed to deserialize message: {}", msg)
            }
        }
    }
}

impl From<std::io::Error> for AppAuthHandlerError {
    fn from(err: std::io::Error) -> AppAuthHandlerError {
        AppAuthHandlerError::IOError(err)
    }
}

impl From<serde_json::error::Error> for AppAuthHandlerError {
    fn from(err: serde_json::error::Error) -> AppAuthHandlerError {
        AppAuthHandlerError::DeserializationError(Box::new(err))
    }
}

impl From<std::string::FromUtf8Error> for AppAuthHandlerError {
    fn from(err: std::string::FromUtf8Error) -> AppAuthHandlerError {
        AppAuthHandlerError::DeserializationError(Box::new(err))
    }
}
