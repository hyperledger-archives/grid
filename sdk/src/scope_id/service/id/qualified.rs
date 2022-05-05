// Copyright 2018-2022 Cargill Incorporated
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

//! Implementation of fully qualified service ID, FullyQualifiedServiceId.

use std::fmt;

use crate::error::InvalidArgumentError;

use super::{CircuitId, ServiceId};

const CIRCUIT_ID_LEN: usize = 11;

const FQSI_SEPARATOR_LEN: usize = 2;

const FQSI_MINIMUM_LEN: usize = CIRCUIT_ID_LEN + FQSI_SEPARATOR_LEN + 1;

/// A fully-qualified service identifier.
///
/// A `FullyQualifiedServiceId` consists of a [`CircuitId`] and [`ServiceId`]. The combination is
/// considered to be fully-qualified because it contains enough context to identify a service.
///
/// The string representation of a fully-qualified service identifier consists of a circuit ID
/// followed by a double-colon separator "::" followed by a service ID. For example, the string
/// "fuKi4-fhek3::93kd" is a valid fully-qualified service identifier.
///
/// The acronym FQSI may be used to refer to a fully-qualified service identifier.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FullyQualifiedServiceId {
    circuit_id: CircuitId,
    service_id: ServiceId,
}

impl FullyQualifiedServiceId {
    /// Create a `FullyQualifiedServiceId` from a [`CircuitId`] and [`ServiceId`].
    pub fn new(circuit_id: CircuitId, service_id: ServiceId) -> Self {
        FullyQualifiedServiceId {
            circuit_id,
            service_id,
        }
    }

    /// Create a `FullyQualifiedServiceId` by parsing a string.
    ///
    /// The string must be a valid circuit id followed by a "::" separator followed by a service
    /// id.
    pub fn new_from_string<T: AsRef<str>>(
        fully_qualified_service_id: T,
    ) -> Result<Self, InvalidArgumentError> {
        let fqsi_str = fully_qualified_service_id.as_ref();

        // Make sure the string is the minimum length.
        if fqsi_str.len() < FQSI_MINIMUM_LEN {
            return Err(InvalidArgumentError::new(
                "fully_qualified_service_id".to_string(),
                format!(
                    "incorrect length of {}, expected at least {}",
                    fqsi_str.len(),
                    FQSI_MINIMUM_LEN,
                ),
            ));
        }

        // Make sure separator "::" immediately follows the circuit id.
        if !(fqsi_str.chars().nth(CIRCUIT_ID_LEN) == Some(':')
            && fqsi_str.chars().nth(CIRCUIT_ID_LEN + 1) == Some(':'))
        {
            return Err(InvalidArgumentError::new(
                "fully_qualified_service_id".to_string(),
                format!(
                    "separator '::' not found at position {} of string",
                    CIRCUIT_ID_LEN
                ),
            ));
        }

        // Extract the CircuitId part, or return error if it's invalid
        let circuit_id = CircuitId::new(&fqsi_str[..CIRCUIT_ID_LEN]).map_err(|e| {
            InvalidArgumentError::new(
                "fully_qualified_service_id".to_string(),
                format!("invalid circuit id part: {}", e.message()),
            )
        })?;

        // Extract the ServiceId part, or return error if it's invalid
        let service_id =
            ServiceId::new(&fqsi_str[CIRCUIT_ID_LEN + FQSI_SEPARATOR_LEN..]).map_err(|e| {
                InvalidArgumentError::new(
                    "fully_qualified_service_id".to_string(),
                    format!("invalid service id part: {}", e.message()),
                )
            })?;

        Ok(FullyQualifiedServiceId {
            circuit_id,
            service_id,
        })
    }

    /// Returns a reference to the [`CircuitId`].
    pub fn circuit_id(&self) -> &CircuitId {
        &self.circuit_id
    }

    /// Returns a reference to the [`ServiceId`].
    pub fn service_id(&self) -> &ServiceId {
        &self.service_id
    }

    /// Returns a tuple of (CircuitId, ServiceId) and consumes this struct.
    pub fn deconstruct(self) -> (CircuitId, ServiceId) {
        (self.circuit_id, self.service_id)
    }
}

impl fmt::Display for FullyQualifiedServiceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}::{}", self.circuit_id, self.service_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{CircuitId, FullyQualifiedServiceId, ServiceId};

    /// Test creating a FullyQualifiedServiceId with new().
    #[test]
    fn test_fqsi_new() {
        let circuit_id =
            CircuitId::new("abcde-abcde").expect("creating CircuitId from \"abcde-abcde\"");
        let service_id = ServiceId::new("fghi").expect("creating CircuitId from \"fghi\"");

        let fqsi = FullyQualifiedServiceId::new(circuit_id.clone(), service_id.clone());
        assert_eq!(*fqsi.circuit_id(), circuit_id);
        assert_eq!(*fqsi.service_id(), service_id);
    }

    /// Tests parsing a string with new_from_string() works for well-formed strings.
    #[test]
    fn test_fqsi_new_from_string_well_formed() {
        let fqsi = FullyQualifiedServiceId::new_from_string("abcde-fghij::0011")
            .expect("creating FullyQualifiedServiceId from string 'abcde-fghij::0011'");
        assert_eq!(fqsi.circuit_id().as_str(), "abcde-fghij");
        assert_eq!(fqsi.service_id().as_str(), "0011");
    }

    /// Tests that parsing a string which is too short gives an error.
    #[test]
    fn test_fqsi_new_from_string_too_short() {
        let result = FullyQualifiedServiceId::new_from_string("abcde-fghi::a");
        assert_eq!(
            &result.unwrap_err().to_string(),
            "incorrect length of 13, expected at least 14 (fully_qualified_service_id)"
        );
    }

    /// Tests that parsing a string which lacks a "::" separator gives an error.
    #[test]
    fn test_fqsi_new_from_string_no_separator() {
        let result = FullyQualifiedServiceId::new_from_string("abcde-fghij--0011");
        assert_eq!(
            &result.unwrap_err().to_string(),
            "separator '::' not found at position 11 of string (fully_qualified_service_id)"
        );
    }

    /// Tests that parsing a string containing an invalid circuit id gives an error.
    #[test]
    fn test_fqsi_new_from_string_invalid_circuit_id() {
        let result = FullyQualifiedServiceId::new_from_string("abc?e-fghij::0011");
        assert_eq!(
            &result.unwrap_err().to_string(),
            "invalid circuit id part: invalid characters, expected ASCII alphanumeric characters \
            separated with a dash ('-') (fully_qualified_service_id)"
        );
    }

    /// Tests that parsing a string containing an invalid service id gives an error.
    #[test]
    fn test_fqsi_new_from_string_invalid_service_id() {
        let result = FullyQualifiedServiceId::new_from_string("abcde-fghij::00?0");
        assert_eq!(
            &result.unwrap_err().to_string(),
            "invalid service id part: invalid characters, expected ASCII alphanumeric \
            (fully_qualified_service_id)"
        );
    }

    /// Tests that Display is implemented to give the expected string.
    #[test]
    fn test_fqsi_new_from_string_display() {
        let fqsi = FullyQualifiedServiceId::new_from_string("abcde-fghij::0012")
            .expect("creating FullyQualifiedServiceId from string 'abcde-fghij::0012'");
        assert_eq!(&format!("{}", fqsi), "abcde-fghij::0012");
    }
}
