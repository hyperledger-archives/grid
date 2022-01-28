// Copyright 2021 Cargill Incorporated
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

mod xml_ffi;

use std::cmp::Ordering;
use std::fs;
use std::path::PathBuf;

use crate::data_validation::error::DataValidationError;
use crate::error::{InternalError, InvalidArgumentError};
use xml_ffi::{XmlDoc, XmlSchema, XmlSchemaParserCtxt};

pub enum Schema {
    OrderXmlV3_4,
    GdsnXmlV3_1,
}

/// Checks whether an XML file validates against a specified schema. This
/// function uses calls to the libxml2 C library. An error will be returned if
/// the file fails to validate for any reason.
///
///
/// For more information about libxml2, see the documentation here:
///     http://www.xmlsoft.org/
///
/// # Arguments
///
/// * `data` - A path to an XML file or an XML string to be validated. This
///     allows more flexibility in how a client handles validation.
/// * `is_path` - Whether the data provided is a path or a string.
/// * `schema_type` - Which type of XML Schema Definition to validate against.
///
pub(super) fn validate_xml(
    data: &str,
    is_path: bool,
    schema_type: Schema,
    schema_dir: &str,
) -> Result<(), DataValidationError> {
    let mut schema = load_schema(schema_type, schema_dir)?;
    if is_path {
        validate_xml_by_path(data, &mut schema)
    } else {
        validate_xml_string(data, &mut schema)
    }
}

fn validate_xml_string(data: &str, schema: &mut XmlSchema) -> Result<(), DataValidationError> {
    let doc = XmlDoc::from_str(data).map_err(DataValidationError::Internal)?;
    let validation_result =
        xml_ffi::validate_doc(schema, doc).map_err(DataValidationError::Internal)?;

    match validation_result.cmp(&0) {
        Ordering::Equal => Ok(()),
        Ordering::Greater => Err(DataValidationError::InvalidArgument(
            InvalidArgumentError::new(data.to_string(), "file fails to validate".to_string()),
        )),
        Ordering::Less => Err(DataValidationError::Internal(InternalError::with_message(
            format!("validation generated an internal error: {}", data),
        ))),
    }
}

fn validate_xml_by_path(data: &str, schema: &mut XmlSchema) -> Result<(), DataValidationError> {
    let validation_result =
        xml_ffi::validate_file(schema, data).map_err(DataValidationError::Internal)?;

    match validation_result.cmp(&0) {
        Ordering::Equal => Ok(()),
        Ordering::Greater => Err(DataValidationError::InvalidArgument(
            InvalidArgumentError::new(data.to_string(), "file fails to validate".to_string()),
        )),
        Ordering::Less => Err(DataValidationError::Internal(InternalError::with_message(
            format!("validation generated an internal error: {}", data),
        ))),
    }
}

fn load_schema(schema_type: Schema, schema_dir: &str) -> Result<XmlSchema, DataValidationError> {
    let mut schema_dir_path = PathBuf::from(schema_dir);
    let schema: Vec<u8> = match schema_type {
        Schema::OrderXmlV3_4 => {
            schema_dir_path.push("Order.xsd");
            if !schema_dir_path.exists() {
                return Err(DataValidationError::InvalidArgument(
                    InvalidArgumentError::new(
                        "xml validation".to_string(),
                        format!(
                            "Cannot validate XML file against XSD file: XSD file {} is \
                                missing. Hint: You may need to use the Grid XSD download tool to \
                                get this file, or the file may have an incorrect name.",
                            schema_dir_path.to_string_lossy()
                        ),
                    ),
                ));
            }
            fs::read_to_string(schema_dir_path.as_path())
                .map_err(|err| {
                    DataValidationError::Internal(InternalError::from_source(Box::new(err)))
                })?
                .into_bytes()
        }
        Schema::GdsnXmlV3_1 => {
            schema_dir_path.push("GridTradeItems.xsd");
            if !schema_dir_path.exists() {
                return Err(DataValidationError::InvalidArgument(
                    InvalidArgumentError::new(
                        "xml validation".to_string(),
                        format!(
                            "Cannot validate XML file against XSD file: XSD file {} is \
                            missing.",
                            schema_dir_path.to_string_lossy()
                        ),
                    ),
                ));
            }
            fs::read_to_string(schema_dir_path.as_path())
                .map_err(|err| {
                    DataValidationError::Internal(InternalError::from_source(Box::new(err)))
                })?
                .into_bytes()
        }
    };

    let schema_parser_ctxt =
        XmlSchemaParserCtxt::from_buffer(&schema).map_err(DataValidationError::Internal)?;

    XmlSchema::from_parser(schema_parser_ctxt, &schema_type, schema_dir)
        .map_err(DataValidationError::Internal)
}
