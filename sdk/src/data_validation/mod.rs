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

mod error;
mod xml;

pub use error::DataValidationError;
use xml::{validate_xml, Schema};

/// Checks whether an XML file is valid against the GS1 BMS Order XML 3.4 schema.
/// An error will be returned if the file fails to validate for any reason.
///
/// For more information about this specification, see the documentation here:
///     https://www.gs1.org/standards/edi-xml/xml-order/3-4-1
///
/// This implementation uses the libxml2 C library. For more information about
/// libxml2, see the documentation here:
///     http://www.xmlsoft.org/
///
/// # Arguments
///
/// * `data` - A path to an XML file or an XML string to be validated.
/// * `is_path` - Whether the data provided is a path or a string.
///
pub fn validate_order_xml_3_4(data: &str, is_path: bool) -> Result<(), DataValidationError> {
    validate_xml(data, is_path, Schema::OrderXmlV3_4)?;
    Ok(())
}
