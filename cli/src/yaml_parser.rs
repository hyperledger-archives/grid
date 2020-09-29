// Copyright (c) 2019 Target Brands, Inc.
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

use crate::error::CliError;
use grid_sdk::protocol::schema::state::DataType;
use serde_yaml::{Mapping, Sequence, Value};

/**
 * Given a yaml object, parse it as a sequence
 *
 * property - Yaml object we wish to parse in as a sequence
 */
pub fn parse_value_as_sequence(
    property: &Mapping,
    key: &str,
) -> Result<Option<Sequence>, CliError> {
    match property.get(&Value::String(key.to_string())) {
        Some(value) => match value.as_sequence() {
            Some(value) => Ok(Some(value.to_vec())),
            None => Err(CliError::InvalidYamlError(format!(
                "Value of {} has an invalid format. Expected is a yaml list.",
                key
            ))),
        },
        None => Ok(None),
    }
}

/**
 * Given a yaml object, parse it as a string
 *
 * property - Yaml object we wish to parse in as a string
 */
pub fn parse_value_as_string(property: &Mapping, key: &str) -> Result<Option<String>, CliError> {
    match property.get(&Value::String(key.to_string())) {
        Some(value) => match value.as_str() {
            Some(value) => Ok(Some(value.to_string())),
            None => Err(CliError::InvalidYamlError(format!(
                "Value of {} has an invalid format. Expected is a yaml string.",
                key
            ))),
        },
        None => Ok(None),
    }
}

/**
 * Given a yaml object, parse it as a bool
 *
 * property - Yaml object we wish to parse in as a bool
 */
pub fn parse_value_as_boolean(property: &Mapping, key: &str) -> Result<Option<bool>, CliError> {
    match property.get(&Value::String(key.to_string())) {
        Some(value) => match value.as_bool() {
            Some(value) => Ok(Some(value)),
            None => Err(CliError::InvalidYamlError(format!(
                "Value of {} has an invalid format. Expected is a yaml boolean (true/false).",
                key
            ))),
        },
        None => Ok(None),
    }
}

/**
 * Given a yaml object, parse it as an i32
 *
 * property - Yaml object we wish to parse in as an i32
 */
pub fn parse_value_as_i32(property: &Mapping, key: &str) -> Result<Option<i32>, CliError> {
    match property.get(&Value::String(key.to_string())) {
        Some(value) => match value.as_i64() {
            Some(value) => Ok(Some(value.to_string().parse::<i32>().map_err(|_| {
                CliError::InvalidYamlError(format!(
                    "Failed to parse value of {} to 32 bit integer",
                    key
                ))
            })?)),
            None => Err(CliError::InvalidYamlError(format!(
                "Value of {} has an invalid format. Expected is a yaml integer.",
                key
            ))),
        },
        None => Ok(None),
    }
}

/**
 * Given a yaml object, parse it as a PropertyDefinition DataType
 *
 * data_type - Yaml object we wish to parse in as a PropertyDefinition DataType
 */
pub fn parse_value_as_data_type(data_type: &str) -> Result<DataType, CliError> {
    match data_type.to_lowercase().as_ref() {
        "string" => Ok(DataType::String),
        "boolean" => Ok(DataType::Boolean),
        "bytes" => Ok(DataType::Bytes),
        "number" => Ok(DataType::Number),
        "enum" => Ok(DataType::Enum),
        "struct" => Ok(DataType::Struct),
        "lat_long" => Ok(DataType::LatLong),
        _ => Err(CliError::InvalidYamlError(format!(
            "Invalid data type for PropertyDefinition: {}",
            data_type
        ))),
    }
}

/**
 * Given a yaml object, parse it as a vec of strings
 *
 * property - Yaml object we wish to parse in as a vec of strings
 */
pub fn parse_value_as_vec_string(
    property: &Mapping,
    key: &str,
) -> Result<Option<Vec<String>>, CliError> {
    match property.get(&Value::String(key.to_string())) {
        Some(value) => match value.as_sequence() {
            Some(sequence) => Ok(Some(
                sequence
                    .iter()
                    .map(|value| match value.as_str() {
                        Some(value) => Ok(value.to_string()),
                        None => Err(CliError::InvalidYamlError(format!(
                            "Values in {} cannot be parsed to string.",
                            key
                        ))),
                    })
                    .collect::<Result<Vec<String>, CliError>>()?,
            )),
            None => Err(CliError::InvalidYamlError(format!(
                "Value of {} has an invalid format. Expected is a yaml list.",
                key
            ))),
        },
        None => Ok(None),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_yaml::Mapping;

    /*
     * Verifies parse_value_as_data_type can parse Values as DataType for valid inputs
     * and returns an error for invalid inputs
     */
    #[test]
    fn test_parse_data_type() {
        // Check the method returns the expected data types for each valid input
        assert_eq!(
            parse_value_as_data_type("string").unwrap(),
            DataType::String
        );
        assert_eq!(parse_value_as_data_type("bytes").unwrap(), DataType::Bytes);
        assert_eq!(
            parse_value_as_data_type("NUMBER").unwrap(),
            DataType::Number
        );
        assert_eq!(parse_value_as_data_type("enum").unwrap(), DataType::Enum);
        assert_eq!(
            parse_value_as_data_type("Struct").unwrap(),
            DataType::Struct
        );
        assert_eq!(
            parse_value_as_data_type("lat_long").unwrap(),
            DataType::LatLong
        );

        // Check the method returns an error for an invalid input
        assert!(parse_value_as_data_type("not_a_valid_type").is_err());
    }

    /*
     * Verifies parse_value_as_sequence can parse Values as sequence for valid inputs
     * and returns an error for invalid inputs
     */
    #[test]
    fn test_parse_value_as_sequence() {
        let key = "sequence".to_string();
        let mut property_valid = Mapping::new();

        // Check method can properly parse a sequence value
        let sequence = Vec::<Value>::new();
        property_valid.insert(
            Value::String(key.clone()),
            Value::Sequence(sequence.clone()),
        );
        assert_eq!(
            parse_value_as_sequence(&property_valid, &key).unwrap(),
            Some(sequence)
        );

        // Check method returns an error when key is found but value cannot be parsed into a
        // sequence
        let mut property_invalid = Mapping::new();
        property_invalid.insert(
            Value::String(key.clone()),
            Value::String("not a sequence".to_string()),
        );
        assert!(parse_value_as_sequence(&property_invalid, &key).is_err());

        // Check method returns Ok(None) when key is not found
        assert!(parse_value_as_sequence(&Mapping::new(), &key)
            .unwrap()
            .is_none());
    }

    /*
     * Verifies parse_value_as_string can parse Values as string for valid inputs
     * and returns an error for invalid inputs
     */
    #[test]
    fn test_parse_value_as_string() {
        let key = "string".to_string();
        let string_value = "my string".to_string();

        // Check method can properly parse a string value
        let mut property_valid = Mapping::new();
        property_valid.insert(
            Value::String(key.clone()),
            Value::String(string_value.clone()),
        );
        assert_eq!(
            parse_value_as_string(&property_valid, &key).unwrap(),
            Some(string_value.clone())
        );

        // Check method returns an error when key is found but value cannot be parsed into a
        // string
        let mut property_invalid = Mapping::new();
        property_invalid.insert(Value::String(key.clone()), Value::Number(0.into()));
        assert!(parse_value_as_string(&property_invalid, &key).is_err());

        // Check method returns Ok(None) when key is not found
        assert!(parse_value_as_string(&Mapping::new(), &key)
            .unwrap()
            .is_none());
    }

    /*
     * Verifies parse_value_as_boolean can parse Values as boolean for valid inputs
     * and returns an error for invalid inputs
     */
    #[test]
    fn test_parse_value_as_boolean() {
        let key = "bool".to_string();
        let bool_value = true;

        // Check method can properly parse a boolean value
        let mut property_valid = Mapping::new();
        property_valid.insert(Value::String(key.clone()), Value::Bool(bool_value.clone()));
        assert_eq!(
            parse_value_as_boolean(&property_valid, &key).unwrap(),
            Some(bool_value.clone())
        );

        // Check method returns an error when key is found but value cannot be parsed into a
        // boolean
        let mut property_invalid = Mapping::new();
        property_invalid.insert(Value::String(key.clone()), Value::Number(0.into()));
        assert!(parse_value_as_boolean(&property_invalid, &key).is_err());

        // Check method returns Ok(None) when key is not found
        assert!(parse_value_as_boolean(&Mapping::new(), &key)
            .unwrap()
            .is_none());
    }

    ///
    /// Verifies parse_parse_value_as_i32 can parse Values as i32 for valid inputs
    ///  and returns an error for invalid inputs
    ///
    #[test]
    fn test_parse_value_as_i32() {
        let key = "number_i32".to_string();
        let number_value = 200;

        // Check method can properly parse a number value
        let mut property_valid = Mapping::new();
        property_valid.insert(
            Value::String(key.clone()),
            Value::Number(number_value.clone().into()),
        );
        assert_eq!(
            parse_value_as_i32(&property_valid, &key).unwrap(),
            Some(number_value.clone())
        );

        // Check method returns an error when key is found but value overflows an i32 capacity.
        let mut property_invalid = Mapping::new();
        property_invalid.insert(
            Value::String(key.clone()),
            Value::Number(3000000000_i64.into()),
        );
        assert!(parse_value_as_i32(&property_invalid, &key).is_err());

        // Check method returns an error when key is found but value cannot be parsed into a
        // number
        let mut property_invalid = Mapping::new();
        property_invalid.insert(
            Value::String(key.clone()),
            Value::String("Not a number".to_string()),
        );
        assert!(parse_value_as_i32(&property_invalid, &key).is_err());

        // Check method returns Ok(None) when key is not found
        assert!(parse_value_as_i32(&Mapping::new(), &key).unwrap().is_none());
    }

    ///
    /// Verifies pparse_value_as_vec_string can parse Values as Vec<String> for valid inputs
    ///  and returns an error for invalid inputs
    ///
    #[test]
    fn test_parse_value_as_vec_string() {
        let key = "vec".to_string();
        let string_value = "value".to_string();
        let sequence_value = vec![Value::String(string_value.clone())];

        // Check method can properly parse a Value into Vec<String>
        let mut property_valid = Mapping::new();
        property_valid.insert(Value::String(key.clone()), Value::Sequence(sequence_value));
        assert_eq!(
            parse_value_as_vec_string(&property_valid, &key).unwrap(),
            Some(vec![string_value.clone()])
        );

        // Check method returns an error when key is found but values inside the sequence cannot
        // be parsed into a string.
        let mut property_invalid = Mapping::new();
        let sequence_value_invalid = vec![Value::Number(0.into())];
        property_invalid.insert(
            Value::String(key.clone()),
            Value::Sequence(sequence_value_invalid),
        );
        assert!(parse_value_as_vec_string(&property_invalid, &key).is_err());

        // Check method returns an error when key is found but value cannot be parsed into a vec
        let mut property_invalid = Mapping::new();
        property_invalid.insert(
            Value::String(key.clone()),
            Value::String("Not a sequence".to_string()),
        );
        assert!(parse_value_as_vec_string(&property_invalid, &key).is_err());

        // Check method returns Ok(None) when key is not found
        assert!(parse_value_as_i32(&Mapping::new(), &key).unwrap().is_none());
    }
}
