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

use core::marker::{PhantomData, PhantomPinned};
use std::ffi::CString;

use crate::error::InternalError;

use libc::{c_char, c_int, c_uint};

#[repr(C)]
#[derive(Clone, Copy)]
/// Represents a pointer to the `xmlSchema` C struct
struct XmlSchemaPtr {
    // This struct is not able to be instantiated outside of this module as it has private
    // fields and no constructors
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

/// Represents the schema used to validate data, created by the `XmlSchemaParserCtxt`
pub(super) struct XmlSchema(*mut XmlSchemaPtr);

impl XmlSchema {
    pub(super) fn from_parser(mut parser: XmlSchemaParserCtxt) -> Result<Self, InternalError> {
        let schema_ptr = unsafe { xmlSchemaParse(parser.as_ptr()) };
        if schema_ptr.is_null() {
            return Err(InternalError::with_message(
                "`xmlSchemaParse` returned a null pointer".to_string(),
            ));
        }
        Ok(Self(schema_ptr))
    }

    fn as_ptr(&mut self) -> *mut XmlSchemaPtr {
        self.0
    }
}

impl Drop for XmlSchema {
    fn drop(&mut self) {
        unsafe {
            xmlSchemaFree(self.0);
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
/// Represents a pointer to the `xmlSchemaParserCtxt` C struct
struct XmlSchemaParserCtxtPtr {
    // This struct is not able to be instantiated outside of this module as it has private
    // fields and no constructors
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

/// Represents the parsing context used to load an `XmlSchema`
pub(super) struct XmlSchemaParserCtxt(*mut XmlSchemaParserCtxtPtr);

impl XmlSchemaParserCtxt {
    pub(super) fn from_buffer(schema: &'static str) -> Result<Self, InternalError> {
        let buff = schema.as_ptr() as *const c_char;
        let size = schema.len() as *const i32;

        let parser_ctxt_ptr = unsafe { xmlSchemaNewMemParserCtxt(buff, size) };
        if parser_ctxt_ptr.is_null() {
            return Err(InternalError::with_message(
                "`xmlSchemaNewMemParserCtxt` returned a null pointer".to_string(),
            ));
        }
        Ok(Self(parser_ctxt_ptr))
    }

    fn as_ptr(&mut self) -> *mut XmlSchemaParserCtxtPtr {
        self.0
    }
}

impl Drop for XmlSchemaParserCtxt {
    fn drop(&mut self) {
        unsafe {
            xmlSchemaFreeParserCtxt(self.0);
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
/// Represents a pointer to the `xmlSchemaValidCtxt` C struct
struct XmlSchemaValidCtxtPtr {
    // This struct is not able to be instantiated outside of this module as it has private
    // fields and no constructors
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

/// Context used to compare the loaded schema against the given data
pub(super) struct XmlSchemaValidCtxt(*mut XmlSchemaValidCtxtPtr);

impl XmlSchemaValidCtxt {
    fn from_schema(schema: &mut XmlSchema) -> Result<Self, InternalError> {
        let ctxt_ptr = unsafe { xmlSchemaNewValidCtxt(schema.as_ptr()) };
        if ctxt_ptr.is_null() {
            return Err(InternalError::with_message(
                "Unable to build XmlSchemaValidCtxt from null pointer".to_string(),
            ));
        }
        Ok(Self(ctxt_ptr))
    }

    fn as_ptr(&mut self) -> *mut XmlSchemaValidCtxtPtr {
        self.0
    }
}

impl Drop for XmlSchemaValidCtxt {
    fn drop(&mut self) {
        unsafe {
            xmlSchemaFreeValidCtxt(self.0);
        }
    }
}

/// Validate an `XmlDoc`, representing a user's data, according to an XML schema.
pub(super) fn validate_doc(schema: &mut XmlSchema, mut doc: XmlDoc) -> Result<i32, InternalError> {
    let mut ctxt = XmlSchemaValidCtxt::from_schema(schema)?;
    let result = unsafe { xmlSchemaValidateDoc(ctxt.as_ptr(), doc.as_ptr(), 0) };
    Ok(result)
}

/// Validate a user's XML file, represented by its file path, according to an XML schema.
pub(super) fn validate_file(schema: &mut XmlSchema, data: &str) -> Result<i32, InternalError> {
    let mut ctxt = XmlSchemaValidCtxt::from_schema(schema)?;
    let path = CString::new(data).map_err(|err| InternalError::from_source(Box::new(err)))?;
    let result = unsafe { xmlSchemaValidateFile(ctxt.as_ptr(), path.as_ptr(), 0) };
    Ok(result)
}

#[repr(C)]
#[derive(Clone, Copy)]
/// Represents a pointer to the `xmlDoc` C struct, which represents a user's XML data
struct XmlDocPtr {
    // This struct is not able to be instantiated outside of this module as it has private
    // fields and no constructors
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

/// Represents a user's XML data
pub(super) struct XmlDoc(*mut XmlDocPtr);

impl XmlDoc {
    pub(super) fn from_str(data: &str) -> Result<Self, InternalError> {
        let doc_ptr = unsafe {
            let data_string =
                CString::new(data).map_err(|err| InternalError::from_source(Box::new(err)))?;
            xmlParseDoc(data_string.as_ptr())
        };
        if doc_ptr.is_null() {
            return Err(InternalError::with_message(
                "`xmlParseDoc` returned a null pointer".to_string(),
            ));
        }

        Ok(Self(doc_ptr))
    }

    fn as_ptr(&mut self) -> *mut XmlDocPtr {
        self.0
    }
}

impl Drop for XmlDoc {
    fn drop(&mut self) {
        unsafe { xmlFreeDoc(self.as_ptr()) }
    }
}

#[link(name = "xml2")]
extern "C" {
    fn xmlParseDoc(cur: *const c_char) -> *mut XmlDocPtr;

    fn xmlSchemaNewMemParserCtxt(
        buffer: *const c_char,
        size: *const c_int,
    ) -> *mut XmlSchemaParserCtxtPtr;

    fn xmlSchemaParse(ctxt: *mut XmlSchemaParserCtxtPtr) -> *mut XmlSchemaPtr;

    fn xmlSchemaNewValidCtxt(schema: *mut XmlSchemaPtr) -> *mut XmlSchemaValidCtxtPtr;

    fn xmlSchemaValidateDoc(
        ctxt: *mut XmlSchemaValidCtxtPtr,
        doc: *mut XmlDocPtr,
        options: c_uint,
    ) -> c_int;

    fn xmlSchemaValidateFile(
        ctxt: *mut XmlSchemaValidCtxtPtr,
        file_name: *const c_char,
        options: c_uint,
    ) -> c_int;

    fn xmlSchemaFreeValidCtxt(ctxt: *mut XmlSchemaValidCtxtPtr);

    fn xmlSchemaFreeParserCtxt(ctxt: *mut XmlSchemaParserCtxtPtr);

    fn xmlSchemaFree(schema: *mut XmlSchemaPtr);

    fn xmlFreeDoc(doc: *mut XmlDocPtr);
}
