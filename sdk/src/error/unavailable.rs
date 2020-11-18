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

//! Module containing ResourceTemporarilyUnavailableError implementation.

use std::error;
use std::fmt;
use std::time::Duration;

/// An error which is returned when an underlying resource is unavailable.
///
/// This error can be handled by retrying, usually in a loop with a small delay.
#[derive(Debug)]
pub struct ResourceTemporarilyUnavailableError {
    source: Box<dyn error::Error>,
    retry_duration_hint: Option<Duration>,
}

impl ResourceTemporarilyUnavailableError {
    /// Constructs a new `ResourceTemporarilyUnavailableError` from a specified source error.
    ///
    /// The implementation of `std::fmt::Display` for this error will simply pass through the
    /// display of the source message unmodified.
    ///
    /// # Examples
    ///
    /// ```
    /// use grid_sdk::error::ResourceTemporarilyUnavailableError;
    ///
    /// let io_err = std::io::Error::new(std::io::ErrorKind::Other, "io error");
    /// let rtu_error = ResourceTemporarilyUnavailableError::from_source(Box::new(io_err));
    /// assert_eq!(format!("{}", rtu_error), "io error");
    /// ```
    pub fn from_source(source: Box<dyn error::Error>) -> Self {
        Self {
            source,
            retry_duration_hint: None,
        }
    }

    /// Constructs a new `ResourceTemporarilyUnavailableError` from a specified source error with
    /// a retry duration hint.
    ///
    /// The hint specified here can be used by the caller as the duration between retry attempts.
    /// Callers may ignore this hint and provide their own algorithms, or may use this `Duration`
    /// as provided.
    ///
    /// The implementation of `std::fmt::Display` for this error will simply pass through the
    /// display of the source message unmodified.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use grid_sdk::error::ResourceTemporarilyUnavailableError;
    ///
    /// let io_err = std::io::Error::new(std::io::ErrorKind::Other, "io error");
    /// let rtu_error = ResourceTemporarilyUnavailableError::from_source_with_hint(Box::new(io_err), Duration::new(10, 0));
    /// assert_eq!(format!("{}", rtu_error), "io error");
    /// ```
    pub fn from_source_with_hint(
        source: Box<dyn error::Error>,
        retry_duration_hint: Duration,
    ) -> Self {
        Self {
            source,
            retry_duration_hint: Some(retry_duration_hint),
        }
    }

    /// Returns the duration which the underlying library provides as a suggestion for an
    /// appropriate amount of time between retry attempts.
    pub fn retry_duration_hint(&self) -> Option<Duration> {
        self.retry_duration_hint
    }
}

impl error::Error for ResourceTemporarilyUnavailableError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

impl fmt::Display for ResourceTemporarilyUnavailableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.source)
    }
}

#[cfg(test)]
pub mod tests {
    use std::time::Duration;

    use crate::error::InternalError;

    use super::*;

    /// Tests that error constructed with `ResourceTemporarilyUnavailableError::from_source` return
    /// a display string which is the same as the source's display string.
    #[test]
    fn test_display_from_source() {
        let msg = "test message";
        let err = ResourceTemporarilyUnavailableError::from_source(Box::new(
            InternalError::with_message(msg.to_string()),
        ));
        assert_eq!(format!("{}", err), msg);
    }

    /// Tests that error constructed with
    /// `ResourceTemporarilyUnavailableError::from_source_with_hint` return a display string which
    /// is the same as the source's display string.
    #[test]
    fn test_display_from_source_with_hint() {
        let msg = "test message";
        let err = ResourceTemporarilyUnavailableError::from_source_with_hint(
            Box::new(InternalError::with_message(msg.to_string())),
            Duration::new(10, 0),
        );
        assert_eq!(format!("{}", err), msg);
    }
}
