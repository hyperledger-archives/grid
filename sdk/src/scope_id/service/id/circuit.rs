// Copyright 2018-2022 Cargill Incorporated
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

//! Implementation of circuit identifier, CircuitId.

use std::convert::TryFrom;
use std::fmt;

use crate::error::InvalidArgumentError;

// Circuit ID strings are always 11 characters in length.
const CIRCUIT_ID_STR_LEN: usize = 11;

/// A circuit identifier.
///
/// A valid circuit identifier is a string of length 11 with the following structure:
///
/// - characters 0-4 are alphanumeric
/// - character 5 is a `-`
/// - characters 6-10 are alphanumeric
///
/// The circuit identifier `00000-00000` is reserved for use as the management (admin) circuit.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct CircuitId {
    inner: Box<str>,
}

impl CircuitId {
    /// Create a `CircuitId` from a string.
    ///
    /// # Arguments
    ///
    /// * `circuit_id` - A string representation of a circuit identifier.
    ///
    /// # Errors
    ///
    /// [`InvalidArgumentError`] can occur when the string length is incorrect (a length of 11 is
    /// expected) or the string contains invalid characters.
    pub fn new<T: Into<String>>(circuit_id: T) -> Result<Self, InvalidArgumentError> {
        CircuitId::new_box_str(circuit_id.into().into_boxed_str())
    }

    // Private as external API for this is to use try_from.
    fn new_box_str(circuit_id: Box<str>) -> Result<Self, InvalidArgumentError> {
        if circuit_id.len() != CIRCUIT_ID_STR_LEN {
            return Err(InvalidArgumentError::new(
                "circuit_id".to_string(),
                format!(
                    "incorrect length of {}, expected {}",
                    circuit_id.len(),
                    CIRCUIT_ID_STR_LEN
                ),
            ));
        }

        if !circuit_id.char_indices().all(|(i, c)| match i {
            5 => c == '-',
            _ => c.is_ascii_alphanumeric(),
        }) {
            return Err(InvalidArgumentError::new(
                "circuit_id".to_string(),
                "invalid characters, expected ASCII alphanumeric characters separated with a \
                dash ('-')"
                    .to_string(),
            ));
        }

        Ok(CircuitId { inner: circuit_id })
    }

    /// Returns a `&str` representing the value of `CircuitId`.
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Returns a `Box<str>`, consuming the `CircuitId`.
    pub fn deconstruct(self) -> Box<str> {
        self.inner
    }
}

impl TryFrom<String> for CircuitId {
    type Error = InvalidArgumentError;

    fn try_from(circuit_id: String) -> Result<Self, Self::Error> {
        CircuitId::new(circuit_id)
    }
}

impl TryFrom<Box<str>> for CircuitId {
    type Error = InvalidArgumentError;

    fn try_from(circuit_id: Box<str>) -> Result<Self, Self::Error> {
        CircuitId::new_box_str(circuit_id)
    }
}

impl fmt::Display for CircuitId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::convert::{TryFrom, TryInto};

    use crate::error::InvalidArgumentError;

    use super::CircuitId;

    /// Tests successfully creating a CircuitId from a well-formed `&str` using CircuitId::new().
    #[test]
    fn test_circuit_id_well_formed_new_str() {
        let circuit_id =
            CircuitId::new("abcde-abcde").expect("creating CircuitId from \"abcde-abcde\"");
        assert_eq!(circuit_id.as_str(), "abcde-abcde")
    }

    /// Tests successfully creating a CircuitId from a well-formed `Box<str>` using
    /// CircuitId::new().
    #[test]
    fn test_circuit_id_well_formed_new_box_str() {
        let s: Box<str> = "abcde-fghij".into();
        let circuit_id = CircuitId::new(s).expect("creating CircuitId from \"abcde-fghij\"");
        assert_eq!(circuit_id.as_str(), "abcde-fghij");
    }

    /// Tests successfully creating a CircuitId from a well-formed `String` using CircuitId::new().
    #[test]
    fn test_circuit_id_well_formed_new_string() {
        let circuit_id = CircuitId::new(String::from("abcde-00000"))
            .expect("creating CircuitId from \"abcde-00000\"");
        assert_eq!(circuit_id.as_str(), "abcde-00000")
    }

    /// Tests successfully creating a CircuitId from a well-formed `String` using
    /// CircuitId::try_from().
    #[test]
    fn test_circuit_id_well_formed_try_from_string() {
        let circuit_id = CircuitId::try_from(String::from("11111-abcde"))
            .expect("creating CircuitId from \"11111-abcde\"");
        assert_eq!(circuit_id.as_str(), "11111-abcde")
    }

    /// Tests successfully creating a CircuitId from a well-formed `String` using
    /// String::try_into().
    #[test]
    fn test_circuit_id_well_formed_try_into_string() {
        let circuit_id: CircuitId = String::from("abcde-1abcd")
            .try_into()
            .expect("creating CircuitId from \"abcde-1abcd\"");
        assert_eq!(circuit_id.as_str(), "abcde-1abcd")
    }

    /// Tests successfully creating a CircuitId from a well-formed `Box<str>` using
    /// Box<str>::try_into().
    #[test]
    fn test_circuit_id_well_formed_from_box_str() {
        let s: Box<str> = "abcd4-ABCDE".into();
        let circuit_id: CircuitId = s
            .try_into()
            .expect("creating CircuitId from \"abcd4-ABCDE\"");
        assert_eq!(circuit_id.as_str(), "abcd4-ABCDE")
    }

    /// Tests successfully creating a CircuitId, deconstructing, and turning it back into
    /// a CircuitId while avoiding the need to allocate a String.
    #[test]
    fn test_circuit_id_round_trip_without_string() {
        let circuit_id: CircuitId = CircuitId::new("abcde-abcde")
            .expect("creating CircuitId from \"abcde-abcde\"")
            .deconstruct()
            .try_into()
            .expect("creating CircuitId from \"abcde-abcde\"");
        assert_eq!(circuit_id.as_str(), "abcde-abcde");
    }

    /// Tests for error creating a CircuitId with two few characters using CircuitId::new().
    #[test]
    fn test_circuit_id_too_short_new() {
        assert_eq!(
            &CircuitId::new("abcd").unwrap_err().to_string(),
            "incorrect length of 4, expected 11 (circuit_id)",
        );
    }

    /// Tests for error creating a CircuitId with two few characters using String::try_into().
    #[test]
    fn test_circuit_id_too_short_from_string() {
        let result: Result<CircuitId, InvalidArgumentError> = String::from("abcd").try_into();
        assert_eq!(
            &result.unwrap_err().to_string(),
            "incorrect length of 4, expected 11 (circuit_id)",
        );
    }

    /// Tests for error creating a CircuitId with two few characters using Box<str>::try_into().
    #[test]
    fn test_circuit_id_too_short_from_box_str() {
        let s: Box<str> = "abcd".into();
        let result: Result<CircuitId, InvalidArgumentError> = s.try_into();
        assert_eq!(
            &result.unwrap_err().to_string(),
            "incorrect length of 4, expected 11 (circuit_id)",
        );
    }

    /// Tests for error creating a CircuitId with two many characters using CircuitId::new().
    #[test]
    fn test_circuit_id_too_long_new() {
        assert_eq!(
            &CircuitId::new("abcde-123456").unwrap_err().to_string(),
            "incorrect length of 12, expected 11 (circuit_id)",
        );
    }

    /// Tests for error creating a CircuitId with two many characters using String::try_into().
    #[test]
    fn test_circuit_id_too_long_from_string() {
        let result: Result<CircuitId, InvalidArgumentError> =
            String::from("abcdef-12345").try_into();
        assert_eq!(
            &result.unwrap_err().to_string(),
            "incorrect length of 12, expected 11 (circuit_id)",
        );
    }

    /// Tests for error creating a CircuitId with two many characters using Box<str>::try_into().
    #[test]
    fn test_circuit_id_too_long_from_box_str() {
        let s: Box<str> = "abcdef-abcde".into();
        let result: Result<CircuitId, InvalidArgumentError> = s.try_into();
        assert_eq!(
            &result.unwrap_err().to_string(),
            "incorrect length of 12, expected 11 (circuit_id)",
        );
    }

    /// Tests for error creating a CircuitId with invalid characters using CircuitId::new().
    #[test]
    fn test_circuit_id_invalid_characters_new() {
        assert_eq!(
            &CircuitId::new("abcd!-12345").unwrap_err().to_string(),
            "invalid characters, expected ASCII alphanumeric characters separated with a dash \
            ('-') (circuit_id)",
        );
    }

    /// Tests for error creating a CircuitId without dash using CircuitId::new().
    #[test]
    fn test_circuit_id_invalid_no_dash_new() {
        assert_eq!(
            &CircuitId::new("abcdef12345").unwrap_err().to_string(),
            "invalid characters, expected ASCII alphanumeric characters separated with a dash \
            ('-') (circuit_id)",
        );
    }

    /// Tests for error creating a CircuitId with invalid characters using String::try_into().
    #[test]
    fn test_circuit_id_invalid_characters_from_string() {
        let result: Result<CircuitId, InvalidArgumentError> =
            String::from("abcd--abcde").try_into();
        assert_eq!(
            &result.unwrap_err().to_string(),
            "invalid characters, expected ASCII alphanumeric characters separated with a dash \
            ('-') (circuit_id)",
        );
    }

    /// Tests for error creating a CircuitId with invalid characters using Box<str>::try_into().
    #[test]
    fn test_circuit_id_invalid_characters_from_box_str() {
        let s: Box<str> = "ab(de-00000".into();
        let result: Result<CircuitId, InvalidArgumentError> = s.try_into();
        assert_eq!(
            &result.unwrap_err().to_string(),
            "invalid characters, expected ASCII alphanumeric characters separated with a dash \
            ('-') (circuit_id)",
        );
    }

    /// Test that CircuitId is displayed as its alphanumeric string of characters.
    #[test]
    fn test_circuit_id_display_value() {
        let circuit_id: CircuitId = String::from("abcde-01234")
            .try_into()
            .expect("creating a CircuitId from \"abcde-01234\"");
        assert_eq!(&format!("{}", circuit_id), "abcde-01234");
    }
}
