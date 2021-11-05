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

use std::error;
use std::fmt;

/// Generic error designed with the expectation that it may be converted into an HTTP response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// A corresponding HTTP status code for the error
    status_code: u16,

    /// The message that would be returned in an HTTP response
    message: String,

    /// Wrapped error that is not exposed in the HTTP resposne
    #[serde(skip_serializing)]
    source: Option<Box<dyn error::Error>>,
}

impl ErrorResponse {
    /// Create a new ErrorResponse
    ///
    /// # Arguments
    ///
    /// * `status_code` - Corresponding HTTP status code
    /// * `message` - External message to display to the user
    ///
    /// # Examples
    /// ```
    /// use crate::grid_sdk::rest_api::resources::error::ErrorResponse;
    ///
    /// let response = ErrorResponse::new(404, "The requested purchase order was not found");
    ///
    /// assert_eq!(404, response.status_code());
    /// assert_eq!("The requested purchase order was not found", response.message());
    /// ```
    pub fn new(status_code: u16, message: &str) -> Self {
        Self {
            status_code,
            message: message.to_string(),
            source: None,
        }
    }

    /// Create a new ErrorResponse that does not expose the underlaying error
    ///
    /// # Arguments
    ///
    /// * `source` - Underlaying internal error
    ///
    /// # Examples
    /// ```
    /// use crate::grid_sdk::rest_api::resources::error::ErrorResponse;
    ///
    /// // Mock an internal error
    /// let error = "NaN".parse::<u32>().unwrap_err();
    ///
    /// let response = ErrorResponse::internal_error(Box::new(error));
    ///
    /// assert_eq!(500, response.status_code());
    /// assert_eq!("An internal error occurred", response.message());
    /// ```
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

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl error::Error for ErrorResponse {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.source.as_ref().map(|s| s.as_ref())
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

#[cfg(test)]
mod tests {
    use super::*;

    use serde::Deserialize;
    use serde_json::Result;

    // Deny any unknown fields so we can test for data leaks
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Response {
        status_code: u16,
        message: String,
    }

    #[test]
    fn test_error_response_internal_error_display() {
        let error = "NaN".parse::<u32>().unwrap_err();
        let response = ErrorResponse::internal_error(Box::new(error));
        assert_eq!(response.to_string(), "invalid digit found in string");
    }

    #[test]
    fn test_error_response_new_display() {
        let response = ErrorResponse::new(501, "The endpoint is not implemented");
        assert_eq!(
            response.to_string(),
            "Status Code 501: Message The endpoint is not implemented"
        );
    }

    #[test]
    fn test_error_response_new_json_serialization() -> Result<()> {
        let response = ErrorResponse::new(501, "The endpoint is not implemented");
        let json = serde_json::to_string(&response)?;
        let deserialized: Response = serde_json::from_str(&json)?;

        assert_eq!(deserialized.status_code, 501);
        assert_eq!(deserialized.message, "The endpoint is not implemented");

        Ok(())
    }

    #[test]
    fn test_error_response_internal_error_json_serialization() -> Result<()> {
        let err = "NaN".parse::<u32>().unwrap_err();
        let response = ErrorResponse::internal_error(Box::new(err));
        let json = serde_json::to_string(&response)?;
        let deserialized: Response = serde_json::from_str(&json)?;

        assert_eq!(deserialized.status_code, 500);
        assert_eq!(deserialized.message, "An internal error occurred");

        Ok(())
    }
}
