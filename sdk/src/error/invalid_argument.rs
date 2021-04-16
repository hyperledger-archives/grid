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

//! Module containing InvalidArgumentError implementation.

use std::error;
use std::fmt;

/// An error returned when an argument passed to a function does not conform to the expected format.
///
/// This always indicates a programming error on behalf of the caller, since the caller should have
/// verified the argument prior to passing it into the function.
#[derive(Debug)]
pub struct InvalidArgumentError {
    argument: String,
    message: String,
}

impl InvalidArgumentError {
    /// Constructs a new `InvalidArgumentError` with a specified argument and message string.
    ///
    /// The argument passed in should be the name of the argument in the function's signature. The
    /// message should be the reason it is invalid, and should not contain the name of the argument
    /// (since Display will combine both argument and message).
    ///
    /// The implementation of `std::fmt::Display` for this error will be a combination of the
    /// argument and the message string provided.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid_sdk::error::InvalidArgumentError;
    ///
    /// let invalid_arg_error = InvalidArgumentError::new("arg1".to_string(), "argument too long".to_string());
    /// assert_eq!(format!("{}", invalid_arg_error), "argument too long (arg1)");
    /// ```
    pub fn new(argument: String, message: String) -> Self {
        Self { argument, message }
    }

    /// Returns the name of the invalid argument.
    pub fn argument(&self) -> String {
        self.argument.clone()
    }

    /// Returns the message, which is an explanation of why the argument is invalid.
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

impl error::Error for InvalidArgumentError {}

impl fmt::Display for InvalidArgumentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", &self.message, &self.argument)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Tests that error constructed with `InvalidArgumentError::new` return message as the
    /// display string.
    #[test]
    fn test_display() {
        let arg = "arg1";
        let msg = "test message";
        let err = InvalidArgumentError::new(arg.to_string(), msg.to_string());
        assert_eq!(format!("{}", err), format!("{} ({})", msg, arg));
    }
}
