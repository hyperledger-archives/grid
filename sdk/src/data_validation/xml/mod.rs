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
) -> Result<(), DataValidationError> {
    let mut schema = load_schema(schema_type)?;
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

fn load_schema(schema_type: Schema) -> Result<XmlSchema, DataValidationError> {
    static ORDER_XML_V3_4_SCHEMA: &str = include_str!("xsd/po/gs1/ecom/Order.xsd");
    static GRID_TRADE_ITEMS_SCHEMA: &str = include_str!("xsd/product/GridTradeItems.xsd");

    let schema = match schema_type {
        Schema::OrderXmlV3_4 => ORDER_XML_V3_4_SCHEMA,
        Schema::GdsnXmlV3_1 => GRID_TRADE_ITEMS_SCHEMA,
    };

    let schema_parser_ctxt =
        XmlSchemaParserCtxt::from_buffer(schema).map_err(DataValidationError::Internal)?;

    XmlSchema::from_parser(schema_parser_ctxt).map_err(DataValidationError::Internal)
}
