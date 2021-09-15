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

use std::cmp::Ordering;
use std::env::{current_dir, set_current_dir};
use std::ffi::CString;
use std::path::PathBuf;

use crate::data_validation::error::DataValidationError;
use crate::error::{InternalError, InvalidArgumentError};

use libc::{c_char, c_int, c_uint};

enum XmlSchema {}
enum XmlSchemaParserCtxt {}
enum XmlSchemaValidCtxt {}
enum XmlDoc {}

#[derive(Clone, Copy)]
struct XmlSchemaPtr(pub *mut XmlSchema);

#[link(name = "xml2")]
extern "C" {
    fn xmlParseDoc(cur: *const c_char) -> *mut XmlDoc;
    fn xmlSchemaNewMemParserCtxt(
        buffer: *const c_char,
        size: *const c_int,
    ) -> *mut XmlSchemaParserCtxt;
    fn xmlSchemaParse(ctxt: *const XmlSchemaParserCtxt) -> *mut XmlSchema;
    fn xmlSchemaFreeParserCtxt(ctxt: *mut XmlSchemaParserCtxt);
    fn xmlSchemaNewValidCtxt(schema: *const XmlSchema) -> *mut XmlSchemaValidCtxt;
    fn xmlSchemaFreeValidCtxt(ctxt: *mut XmlSchemaValidCtxt);
    fn xmlSchemaValidateDoc(
        ctxt: *const XmlSchemaValidCtxt,
        doc: *const XmlDoc,
        options: c_uint,
    ) -> c_int;
    fn xmlSchemaValidateFile(
        ctxt: *const XmlSchemaValidCtxt,
        file_name: *const c_char,
        options: c_uint,
    ) -> c_int;
}

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
    let schema = load_schema(schema_type)?;
    if is_path {
        validate_xml_by_path(data, schema)
    } else {
        validate_xml_string(data, schema)
    }
}

fn validate_xml_string(data: &str, schema: XmlSchemaPtr) -> Result<(), DataValidationError> {
    unsafe {
        let data_string = CString::new(data).map_err(|err| {
            DataValidationError::Internal(InternalError::from_source(Box::new(err)))
        })?;
        let doc = xmlParseDoc(data_string.as_ptr());

        let schema_valid_ctxt = xmlSchemaNewValidCtxt(schema.0);
        let result = xmlSchemaValidateDoc(schema_valid_ctxt, doc, 0);
        xmlSchemaFreeValidCtxt(schema_valid_ctxt);

        match result.cmp(&0) {
            Ordering::Equal => Ok(()),
            Ordering::Greater => Err(DataValidationError::InvalidArgument(
                InvalidArgumentError::new(data.to_string(), "file fails to validate".to_string()),
            )),
            Ordering::Less => Err(DataValidationError::Internal(InternalError::with_message(
                format!("validation generated an internal error: {}", data),
            ))),
        }
    }
}

fn validate_xml_by_path(data: &str, schema: XmlSchemaPtr) -> Result<(), DataValidationError> {
    let path = CString::new(data)
        .map_err(|err| DataValidationError::Internal(InternalError::from_source(Box::new(err))))?;

    unsafe {
        let schema_valid_ctxt = xmlSchemaNewValidCtxt(schema.0);
        let result = xmlSchemaValidateFile(schema_valid_ctxt, path.as_ptr(), 0);
        xmlSchemaFreeValidCtxt(schema_valid_ctxt);

        match result.cmp(&0) {
            Ordering::Equal => Ok(()),
            Ordering::Greater => Err(DataValidationError::InvalidArgument(
                InvalidArgumentError::new(data.to_string(), "file fails to validate".to_string()),
            )),
            Ordering::Less => Err(DataValidationError::Internal(InternalError::with_message(
                format!("validation generated an internal error: {}", data),
            ))),
        }
    }
}

fn load_schema(schema_type: Schema) -> Result<XmlSchemaPtr, DataValidationError> {
    static ORDER_XML_V3_4_SCHEMA: &str = include_str!("xsd/po/gs1/ecom/Order.xsd");
    static GRID_TRADE_ITEMS_SCHEMA: &str = include_str!("xsd/product/GridTradeItems.xsd");

    let cwd = current_dir()?;

    let schema = match schema_type {
        Schema::OrderXmlV3_4 => {
            let mut schema_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            schema_path.push("src/data_validation/xml/xsd/po/gs1/ecom");
            set_current_dir(schema_path)?;
            ORDER_XML_V3_4_SCHEMA
        }
        Schema::GdsnXmlV3_1 => GRID_TRADE_ITEMS_SCHEMA,
    };

    let buff = schema.as_ptr() as *const c_char;
    let size = schema.len() as *const i32;

    unsafe {
        let schema_parser_ctxt = xmlSchemaNewMemParserCtxt(buff, size);
        let schema = xmlSchemaParse(schema_parser_ctxt);
        xmlSchemaFreeParserCtxt(schema_parser_ctxt);
        set_current_dir(cwd)?;
        Ok(XmlSchemaPtr(schema))
    }
}
