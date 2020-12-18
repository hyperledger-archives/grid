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

use std::error;
use std::fmt;

/// Designed with the expectation that it may be converted into an http
/// response.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    status_code: u16,
    message: String,
    #[serde(skip_serializing)]
    source: Option<Box<dyn error::Error>>,
}

impl ErrorResponse {
    pub fn new(status_code: u16, message: &str) -> Self {
        Self {
            status_code,
            message: message.to_string(),
            source: None,
        }
    }

    pub fn internal_error(source: Box<dyn error::Error>) -> Self {
        Self {
            status_code: 500,
            message: "An internal error occurred".to_string(),
            source: Some(source),
        }
    }

    pub fn status_code(&self) -> u16 {
        self.status_code
    }
}

impl error::Error for ErrorResponse {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &self.source {
            Some(s) => Some(s.as_ref()),
            None => None,
        }
    }
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref src) = self.source {
            write!(f, "{}", src)
        } else {
            write!(
                f,
                "Status Code {}: Message {}",
                self.status_code, self.message
            )
        }
    }
}
