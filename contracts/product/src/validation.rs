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

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::ApplyError;
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
    }
}

/* The purpose of this file is to programmatically express the equation used to validate a GTIN
It validates gtin format to avoid mistype errors similar to a credit card validation
Check digit validation: (https://www.gs1.org/services/how-calculate-check-digit-manually) */

// Leaving this as an extensible function, so other validation rules can be implemented by GTIN format
pub fn validate_gtin(gtin: &str) -> Result<(), ApplyError> {
    // Check that gtin is numeric only
    if is_numeric(gtin) {
        match gtin.chars().count() {
            // GTIN-8 is an 8-digit number used predominately outside of North America on smaller packaging (not supported)
            8 => Err(ApplyError::InvalidTransaction(format!(
                "Invalid GTIN, GTIN-8 is not supported at this time: {}",
                gtin
            ))),
            // GTIN-12 is a 12-digit number used primarily in North America
            12 => check_digit_validation(gtin),
            // GTIN-13 (it could also be a GLN or the first 13 digits of a GRAI, GDTI or GCN.) (ex: 9781981855728)
            13 => check_digit_validation(gtin),
            // GTIN-14 is a 14-digit number used to identify trade items at various packaging levels
            14 => check_digit_validation(gtin),
            // Invalid length
            _ => Err(ApplyError::InvalidTransaction(format!(
                "Invalid length for GTIN identifier: {}",
                gtin
            ))),
        }
    } else {
        Err(ApplyError::InvalidTransaction(format!(
            "Invalid format, GTIN identifiers only contain numbers: {}",
            gtin
        )))
    }
}

fn check_digit_validation(gtin: &str) -> Result<(), ApplyError> {
    let mut gtin_vec: Vec<char> = gtin.chars().collect();
    // Remove the check digit from the gtin_vec and store it for later
    let check_digit_char = gtin_vec
        .pop()
        .expect("No characters found, but string length was > 0");
    let check_digit = convert_char_to_int(check_digit_char);
    let mut sum = 0;
    let mut index = 0;

    if is_even(gtin_vec.len()) {
        // For gtin-13
        for digit in &gtin_vec {
            if is_even(index) {
                sum += convert_char_to_int(*digit);
                index += 1;
            } else {
                sum += 3 * convert_char_to_int(*digit);
                index += 1;
            }
        }
    } else {
        // For gtin 12, 14
        for digit in &gtin_vec {
            if is_even(index) {
                sum += 3 * convert_char_to_int(*digit);
                index += 1;
            } else {
                sum += convert_char_to_int(*digit);
                index += 1;
            }
        }
    }

    let nearest_ten = ceiling_to_nearest_ten(sum as f32);
    let computed_check_digit = nearest_ten - sum;

    if computed_check_digit == check_digit {
        Ok(())
    } else {
        Err(ApplyError::InvalidTransaction(format!(
            "Invalid gtin, check digit validation failed: {}",
            gtin
        )))
    }
}

fn is_even(num: usize) -> bool {
    num % 2 == 0
}

fn is_numeric(s: &str) -> bool {
    s.parse::<f64>().is_ok()
}

fn ceiling_to_nearest_ten(num: f32) -> i32 {
    let c: f32 = num / 10.0;
    let ceil: f32 = c.ceil();
    let c_ceil = ceil as i32;
    10 * c_ceil
}

fn convert_char_to_int(c: char) -> i32 {
    char::to_digit(c, 10).expect("No conversion for char to i32") as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // This tests that the check-digit validation of the valid gtin-12: "688955434684" is true
    fn valid_gtin_12() {
        assert!(validate_gtin("688955434684").is_ok());
    }

    #[test]
    // This tests that the check-digit validation of the valid gtin-12: "688955434584" is false
    fn invalid_gtin_12() {
        assert_eq!(
            validate_gtin("688955434584").err().unwrap().to_string(),
            "InvalidTransaction: Invalid gtin, check digit validation failed: 688955434584"
        );
    }

    #[test]
    // This tests that the check-digit validation of the valid gtin-13: "9781981855728" is true
    fn valid_gtin_13() {
        assert!(validate_gtin("9781981855728").is_ok());
    }

    #[test]
    // This tests that the check-digit validation of the valid gtin-13: "9781981855738" is false
    fn invalid_gtin_13() {
        assert_eq!(
            validate_gtin("9781981855738").err().unwrap().to_string(),
            "InvalidTransaction: Invalid gtin, check digit validation failed: 9781981855738"
        );
    }

    #[test]
    // This tests that the check-digit validation of the valid gtin-13: "10012345678902" is true
    fn valid_gtin_14() {
        assert!(validate_gtin("10012345678902").is_ok());
    }

    #[test]
    // This tests that the check-digit validation of the valid gtin-13: "10012345678912" is false
    fn invalid_gtin_14() {
        assert_eq!(
            validate_gtin("10012345678912").err().unwrap().to_string(),
            "InvalidTransaction: Invalid gtin, check digit validation failed: 10012345678912"
        );
    }

    #[test]
    // This tests gtins entered of an invalid length (too long)
    fn invalid_gtin_length_long() {
        assert_eq!(
            validate_gtin("10012345678923423423423412")
                .err()
                .unwrap()
                .to_string(),
            "InvalidTransaction: Invalid length for GTIN identifier: 10012345678923423423423412"
        );
    }

    #[test]
    // This tests gtins entered of an invalid length (too short)
    fn invalid_gtin_length_short() {
        assert_eq!(
            validate_gtin("123").err().unwrap().to_string(),
            "InvalidTransaction: Invalid length for GTIN identifier: 123"
        );
    }

    #[test]
    // This tests gtins entered of an invalid format
    fn invalid_gtin_format() {
        assert_eq!(
            validate_gtin("1012938473jer").err().unwrap().to_string(),
            "InvalidTransaction: Invalid format, GTIN identifiers only contain numbers: 1012938473jer"
        );
    }

    #[test]
    // This tests that a valid gtin-8 returns an error (not supported)
    fn valid_gtin_8() {
        assert!(validate_gtin("40170725").is_err());
        assert_eq!(
            validate_gtin("40170725").err().unwrap().to_string(),
            "InvalidTransaction: Invalid GTIN, GTIN-8 is not supported at this time: 40170725"
        );
    }
}
