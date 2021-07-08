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

//! Module containing InvalidStateError implementation.

use std::error;
use std::fmt;

/// An error returned when an operation cannot be completed because the state of the underlying
/// struct is inconsistent.
///
/// This can be caused by a caller when a sequence of functions is called in a way that results in
/// a state which is inconsistent.
///
/// This usually indicates a programming error on behalf of the caller.
#[derive(Debug)]
pub struct InvalidStateError {
    message: String,
}

impl InvalidStateError {
    /// Constructs a new `InvalidStateError` with a specified message string.
    ///
    /// The implementation of `std::fmt::Display` for this error will be the message string
    /// provided.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid_sdk::error::InvalidStateError;
    ///
    /// let invalid_state_error = InvalidStateError::with_message("oops".to_string());
    /// assert_eq!(format!("{}", invalid_state_error), "oops");
    /// ```
    pub fn with_message(message: String) -> Self {
        Self { message }
    }
}

impl error::Error for InvalidStateError {}

impl fmt::Display for InvalidStateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.message)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Tests that error constructed with `InvalidStateError::with_message` return message as the
    /// display string.
    #[test]
    fn test_display_with_message() {
        let msg = "test message";
        let err = InvalidStateError::with_message(msg.to_string());
        assert_eq!(format!("{}", err), msg);
    }
}
