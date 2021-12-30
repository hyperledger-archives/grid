/*
 * Copyright 2021 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
*/

use regex::Regex;

use crate::data_validation::error::DataValidationError;
use crate::error::InvalidArgumentError;

pub const ALT_ID_FORMAT: &str =
    "^[\\w\\-\\+=/~!@#\\$%\\^&\\*{}|\\[\\]<>\\?]+:[\\w\\-\\+=/~!@#\\$%\\^&\\*{}|\\[\\]<>\\?]+$";

pub fn validate_alt_id_format(id: &str) -> Result<(), DataValidationError> {
    let alt_id_format = Regex::new(ALT_ID_FORMAT).unwrap();
    if !alt_id_format.is_match(id) {
        return Err(DataValidationError::InvalidArgument(
            InvalidArgumentError::new(
                "alternate_id".to_string(),
                format!(
            "Invalid alternate ID format: '{}'; must match <alternate_id_type>:<alternate_id>",
            id
        ),
            ),
        ));
    }
    Ok(())
}
