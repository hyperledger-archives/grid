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
use std::ffi::CString;

use crate::error::{InternalError, InvalidArgumentError};
use crate::products::gdsn::error::ProductGdsnError;

use libc::{c_char, c_int, c_uint};

enum XmlSchema {}
enum XmlSchemaParserCtxt {}
enum XmlSchemaValidCtxt {}

#[derive(Clone, Copy)]
struct XmlSchemaPtr(pub *mut XmlSchema);

#[link(name = "xml2")]
extern "C" {
    fn xmlSchemaNewMemParserCtxt(
        buffer: *const c_char,
        size: *const c_int,
    ) -> *mut XmlSchemaParserCtxt;
    fn xmlSchemaParse(ctxt: *const XmlSchemaParserCtxt) -> *mut XmlSchema;
    fn xmlSchemaFreeParserCtxt(ctxt: *mut XmlSchemaParserCtxt);
    fn xmlSchemaNewValidCtxt(schema: *const XmlSchema) -> *mut XmlSchemaValidCtxt;
    fn xmlSchemaFreeValidCtxt(ctxt: *mut XmlSchemaValidCtxt);
    fn xmlSchemaValidateFile(
        ctxt: *const XmlSchemaValidCtxt,
        file_name: *const c_char,
        options: c_uint,
    ) -> c_int;
}

pub fn validate_product_definitons(xml_path: &str) -> Result<(), ProductGdsnError> {
    let schema = load_schema();
    let path = CString::new(xml_path)
        .map_err(|err| ProductGdsnError::Internal(InternalError::from_source(Box::new(err))))?;

    unsafe {
        let schema_valid_ctxt = xmlSchemaNewValidCtxt(schema.0);
        let result = xmlSchemaValidateFile(schema_valid_ctxt, path.as_ptr(), 0);
        xmlSchemaFreeValidCtxt(schema_valid_ctxt);

        match result.cmp(&0) {
            Ordering::Equal => Ok(()),
            Ordering::Greater => Err(ProductGdsnError::InvalidArgument(
                InvalidArgumentError::new(
                    xml_path.to_string(),
                    "file fails to validate".to_string(),
                ),
            )),
            Ordering::Less => Err(ProductGdsnError::Internal(InternalError::with_message(
                format!("validation generated an internal error: {}", xml_path),
            ))),
        }
    }
}

fn load_schema() -> XmlSchemaPtr {
    static GRID_TRADE_ITEMS_SCHEMA: &str = include_str!("GridTradeItems.xsd");

    let buff = GRID_TRADE_ITEMS_SCHEMA.as_ptr() as *const c_char;
    let size = GRID_TRADE_ITEMS_SCHEMA.len() as *const i32;

    unsafe {
        let schema_parser_ctxt = xmlSchemaNewMemParserCtxt(buff, size);
        let schema = xmlSchemaParse(schema_parser_ctxt);
        xmlSchemaFreeParserCtxt(schema_parser_ctxt);

        XmlSchemaPtr(schema)
    }
}
