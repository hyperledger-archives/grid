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
use grid_sdk::protocol::product::state::ProductType;
use grid_sdk::protocol::schema::state::{DataType, PropertyValue, PropertyValueBuilder};
use grid_sdk::protocol::schema::state::{LatLong, LatLongBuilder};
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
 * Given a yaml object, parse it as a vector of bytes
 *
 * property - Yaml object we wish to parse in as a vector of bytes
 */
pub fn parse_value_as_bytes(property: &Mapping, key: &str) -> Result<Option<Vec<u8>>, CliError> {
    match property.get(&Value::String(key.to_string())) {
        Some(value) => match value.as_i64() {
            Some(value) => Ok(Some(value.to_string().into_bytes())),
            None => Err(CliError::InvalidYamlError(format!(
                "Value of {} has an invalid format. Expected is a yaml bytes value.",
                key
            ))),
        },
        None => Ok(None),
    }
}

/**
 * Given a yaml object, parse it as an i64
 *
 * property - Yaml object we wish to parse in as an i64
 */
pub fn parse_value_as_i64(property: &Mapping, key: &str) -> Result<Option<i64>, CliError> {
    match property.get(&Value::String(key.to_string())) {
        Some(value) => match value.as_i64() {
            Some(value) => Ok(Some(value.to_string().parse::<i64>().map_err(|_| {
                CliError::InvalidYamlError(format!(
                    "Failed to parse value of {} to 64 bit integer",
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
 * Given a yaml object, parse it as an u32
 *
 * property - Yaml object we wish to parse in as an u32
 */
pub fn parse_value_as_u32(property: &Mapping, key: &str) -> Result<Option<u32>, CliError> {
    match property.get(&Value::String(key.to_string())) {
        // Serde only has methods to match 64 bit nums
        Some(value) => match value.as_u64() {
            Some(value) => Ok(Some(value.to_string().parse::<u32>().map_err(|_| {
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
 * Given a yaml object, parse it as a LatLong object
 *
 * property - Yaml object we wish to parse in as a LatLong object
 */
pub fn parse_value_as_lat_long(property: &Mapping, key: &str) -> Result<Option<LatLong>, CliError> {
    match property.get(&Value::String(key.to_string())) {
        Some(value) => match value.as_str() {
            Some(value) => {
                let lat_long: Vec<&str> = value.split(',').collect();

                let lat: i64 = lat_long[0].parse().map_err(|err| {
                    CliError::InvalidYamlError(format!(
                        "Failed to parse the Latitude value for LatLong: {}",
                        err
                    ))
                })?;

                let long: i64 = lat_long[1].parse().map_err(|err| {
                    CliError::InvalidYamlError(format!(
                        "Failed to parse the Longitude value for LatLong: {}",
                        err
                    ))
                })?;

                Ok(Some(
                    LatLongBuilder::new()
                        .with_lat_long(lat, long)
                        .build()
                        .map_err(|err| {
                            CliError::InvalidYamlError(format!("Failed to build LatLong: {}", err))
                        })?,
                ))
            }
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

/**
 * Given a yaml object, parse it as a Product ProductType
 *
 * product_type - Yaml object we wish to parse in as a Product ProductType
 */
pub fn parse_value_as_product_type(product_type: &str) -> Result<ProductType, CliError> {
    match product_type.to_uppercase().as_ref() {
        "GS1" => Ok(ProductType::GS1),
        _ => Err(CliError::InvalidYamlError(format!(
            "Invalid product_type for value: {}",
            product_type
        ))),
    }
}

/**
 * Given a yaml key/val, parse the val as a list of Property Value objects
 *
 * properties - One or more yaml objects to be parsed as a Property Value
 */
pub fn parse_value_as_repeated_property_values(
    properties: &[Value],
) -> Result<Vec<PropertyValue>, CliError> {
    properties
        .iter()
        .map(|value| {
            let property = value.as_mapping().ok_or_else(|| {
                CliError::InvalidYamlError(
                    "Failed to parse schema property definition.".to_string(),
                )
            })?;
            parse_value_as_property_value(property)
        })
        .collect()
}

/**
 * Given a yaml object, parse it as a Property Value
 *
 * property - Yaml object we have determined to be a Property Value
 */
pub fn parse_value_as_property_value(property: &Mapping) -> Result<PropertyValue, CliError> {
    let data_type = parse_value_as_data_type(
        &parse_value_as_string(property, "data_type")?.ok_or_else(|| {
            CliError::InvalidYamlError(
                "Missing `data_type` field for property definition.".to_string(),
            )
        })?,
    )?;

    let mut property_value = PropertyValueBuilder::new()
        .with_name(parse_value_as_string(property, "name")?.ok_or_else(|| {
            CliError::InvalidYamlError(
                "Missing `name` field for product property value.".to_string(),
            )
        })?)
        .with_data_type(data_type.clone());

    property_value = match data_type {
        DataType::Bytes => property_value.with_bytes_value(
            parse_value_as_bytes(property, "bytes_value")?.ok_or_else(|| {
                CliError::InvalidYamlError(
                    "Missing `bytes_value` field for property value with type BYTES.".to_string(),
                )
            })?,
        ),
        DataType::Boolean => property_value.with_boolean_value(
            parse_value_as_boolean(property, "boolean_value")?.ok_or_else(|| {
                CliError::InvalidYamlError(
                    "Missing `boolean_value` field for property value with type BOOLEAN."
                        .to_string(),
                )
            })?,
        ),
        DataType::Number => property_value.with_number_value(
            parse_value_as_i64(property, "number_value")?.ok_or_else(|| {
                CliError::InvalidYamlError(
                    "Missing `number_value` field for property value with type NUMBER.".to_string(),
                )
            })?,
        ),
        DataType::String => property_value.with_string_value(
            parse_value_as_string(property, "string_value")?.ok_or_else(|| {
                CliError::InvalidYamlError(
                    "Missing `string_value` field for property value with type STRING.".to_string(),
                )
            })?,
        ),
        DataType::Enum => property_value.with_enum_value(
            parse_value_as_u32(property, "enum_value")?.ok_or_else(|| {
                CliError::InvalidYamlError(
                    "Missing `enum_value` field for property value with type ENUM.".to_string(),
                )
            })?,
        ),
        DataType::Struct => {
            // Properties is a repeated field, so we recursively call `parse_value_as_properties`
            let properties = parse_value_as_repeated_property_values(
                property
                    .get(&Value::String("struct_values".to_string()))
                    .ok_or_else(|| {
                        CliError::InvalidYamlError(
                            "Missing `struct_values` field for property value with type STRUCT."
                                .to_string(),
                        )
                    })?
                    .as_sequence()
                    .ok_or_else(|| {
                        CliError::InvalidYamlError(
                            "Failed to parse `struct_values` as a sequence".to_string(),
                        )
                    })?,
            )?;
            property_value.with_struct_values(properties)
        }
        DataType::LatLong => property_value.with_lat_long_value(
            parse_value_as_lat_long(property, "lat_long_value")?.ok_or_else(|| {
                CliError::InvalidYamlError(
                    "Missing `lat_long_value` field for property value with type LATLONG."
                        .to_string(),
                )
            })?,
        ),
    };

    property_value.build().map_err(|err| {
        CliError::PayloadError(format!("Failed to build property definition: {}", err))
    })
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

    /*
     * Verifies parse_value_as_bytes can parse Values as bytes for valid inputs
     * and returns an error for invalid inputs
     */
    #[test]
    fn test_parse_value_as_bytes() {
        let key = "bytes".to_string();
        let bytes_val: i64 = 123456;

        // Check method can properly parse a bytes value
        let mut property_valid = Mapping::new();
        property_valid.insert(Value::String(key.clone()), Value::Number(bytes_val.into()));
        assert_eq!(
            parse_value_as_bytes(&property_valid, &key).unwrap(),
            Some(bytes_val.to_string().into_bytes())
        );

        // Check method returns an error when key is found but value cannot be parsed into a
        // bytes
        let mut property_invalid = Mapping::new();
        property_invalid.insert(
            Value::String(key.clone()),
            Value::String("invalid".to_string()),
        );
        assert!(parse_value_as_bytes(&property_invalid, &key).is_err());

        // Check method returns Ok(None) when key is not found
        assert!(parse_value_as_bytes(&Mapping::new(), &key)
            .unwrap()
            .is_none());
    }

    /*
     * Verifies parse_value_as_i64 can parse Values as i64 for valid inputs
     * and returns an error for invalid inputs
     */
    #[test]
    fn test_parse_value_as_i64() {
        let key = "i64".to_string();
        let number_val: i64 = 123456;

        // Check method can properly parse a i64 value
        let mut property_valid = Mapping::new();
        property_valid.insert(Value::String(key.clone()), Value::Number(number_val.into()));
        assert_eq!(
            parse_value_as_i64(&property_valid, &key).unwrap(),
            Some(number_val)
        );

        // Check method returns an error when key is found but value cannot be parsed into a
        // i64
        let mut property_invalid = Mapping::new();
        property_invalid.insert(
            Value::String(key.clone()),
            Value::String("invalid".to_string()),
        );
        assert!(parse_value_as_i64(&property_invalid, &key).is_err());

        // Check method returns Ok(None) when key is not found
        assert!(parse_value_as_i64(&Mapping::new(), &key).unwrap().is_none());
    }

    /*
     * Verifies parse_value_as_u32 can parse Values as u32 for valid inputs
     * and returns an error for invalid inputs
     */
    #[test]
    fn test_parse_value_as_u32() {
        let key = "u32".to_string();
        let number_val: u32 = 123456;

        // Check method can properly parse a u32 value
        let mut property_valid = Mapping::new();
        property_valid.insert(Value::String(key.clone()), Value::Number(number_val.into()));
        assert_eq!(
            parse_value_as_u32(&property_valid, &key).unwrap(),
            Some(number_val)
        );

        // Check method returns an error when key is found but value cannot be parsed into a
        // u32
        let mut property_invalid = Mapping::new();
        property_invalid.insert(
            Value::String(key.clone()),
            Value::String("invalid".to_string()),
        );
        assert!(parse_value_as_u32(&property_invalid, &key).is_err());

        // Check method returns Ok(None) when key is not found
        assert!(parse_value_as_u32(&Mapping::new(), &key).unwrap().is_none());
    }

    /*
     * Verifies parse_value_as_lat_long can parse Values as LatLong for valid inputs
     * and returns an error for invalid inputs
     */
    #[test]
    fn test_parse_value_as_lat_long() {
        let key = "lat_long".to_string();
        let lat_long_str = "44919724,-93277953".to_string();
        let lat_long: LatLong = LatLongBuilder::new()
            .with_lat_long(44919724 as i64, -93277953 as i64)
            .build()
            .unwrap();

        // Check method can properly parse a lat_long value
        let mut property_valid = Mapping::new();
        property_valid.insert(
            Value::String(key.clone()),
            Value::String(lat_long_str.into()),
        );
        assert_eq!(
            parse_value_as_lat_long(&property_valid, &key).unwrap(),
            Some(lat_long)
        );

        // Check method returns an error when key is found but value cannot be parsed into a
        // lat_long
        let mut property_invalid = Mapping::new();
        property_invalid.insert(
            Value::String(key.clone()),
            Value::String("invalid".to_string()),
        );
        assert!(parse_value_as_lat_long(&property_invalid, &key).is_err());

        // Check method returns Ok(None) when key is not found
        assert!(parse_value_as_lat_long(&Mapping::new(), &key)
            .unwrap()
            .is_none());
    }

    /*
     * Verifies parse_value_as_product_type can parse Values as PropertyType for valid inputs
     */
    #[test]
    fn test_parse_value_as_product_type() {
        assert_eq!(
            parse_value_as_product_type("GS1").unwrap(),
            ProductType::GS1
        );
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
