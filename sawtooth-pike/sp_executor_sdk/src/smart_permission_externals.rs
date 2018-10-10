// Copyright 2018 Cargill Incorporated
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

use std::collections::HashMap;
use std::string::FromUtf8Error;
use std::error::Error as StdError;
use std::fmt;

use sawtooth_sdk::processor::handler::{ContextError, TransactionContext};
use wasmi::{
    MemoryRef,
    FuncRef,
    FuncInstance,
    Externals,
    HostError,
    Error,
    MemoryDescriptor,
    MemoryInstance,
    ModuleImportResolver,
    RuntimeArgs,
    RuntimeValue,
    Signature,
    ValueType,
    Trap,
    TrapKind
};
use wasmi::memory_units::Pages;

// External function indices

/// Args
///
/// 1) Pointer offset in memory for address string
/// 2) Length of address string
///
const GET_STATE_IDX: usize = 0;

/// Args
///
/// 1) Pointer value
///
const GET_PTR_LEN_IDX: usize = 1;

/// Args
///
/// 1) Pointer value
///
const GET_PTR_CAP_IDX: usize = 2;

/// Args
///
/// 1) size of allocated region
///
/// Returns - raw pointer to allocated block
///
const ALLOC_IDX: usize = 3;

/// Args
///
/// 1) offset of byte in memory
///
/// Returns - byte value stored at offset
///
const READ_BYTE_IDX: usize = 4;

/// Args
///
/// 1) ptr to write to
///
/// 2) offset to realtive to ptr to write byte to
///
/// 3) byte to be written to offset
///
/// Returns - 1 if successful, or a negative value if failure
///
const WRITE_BYTE_IDX: usize = 5;

/// Args
///
/// 1) First pointer in pointer list
///
/// Returns - length of collection if collection exists, and -1 otherwise
///
const GET_COLLECTION_LEN_IDX: usize = 6;

/// Args
///
/// 1) First pointer in pointer collection
///
/// 2) index of pointer request
///
/// Returns - raw pointer at index if index and collection are
/// valid, and -1 otherwise
///
const GET_PTR_FROM_COLLECTION_IDX: usize = 7;

pub struct SmartPermissionExternals {
    pub memory_ref: MemoryRef,
    context: TransactionContext,
    ptrs: HashMap<u32, Pointer>,
    ptr_collections: HashMap<u32, Vec<u32>>,
    memory_write_offset: u32
}

impl SmartPermissionExternals {
    pub fn new(
        memory_ref: Option<MemoryRef>,
        context: TransactionContext
    ) -> Result<SmartPermissionExternals, ExternalsError> {
        let m_ref = if let Some(m) = memory_ref {
            m
        } else {
            MemoryInstance::alloc(Pages(256), None)?
        };

        Ok(SmartPermissionExternals {
            memory_ref: m_ref,
            context,
            ptrs: HashMap::new(),
            ptr_collections: HashMap::new(),
            memory_write_offset: 0
        })
    }

    fn ptr_to_string(&mut self, raw_ptr: u32) -> Result<String, ExternalsError> {
        if let Some(p) = self.ptrs.get(&raw_ptr) {
            let bytes = self
                .get_memory_ref()
                .get(p.raw, p.length)?;

            String::from_utf8(bytes)
                .map_err(ExternalsError::from)
        } else {
            Err(ExternalsError::from(format!("ptr referencing {} not found", raw_ptr)))
        }
    }

    fn ptr_to_vec(&mut self, raw_ptr: u32) -> Result<Vec<u8>, ExternalsError> {
        if let Some(p) = self.ptrs.get(&raw_ptr) {
            self.get_memory_ref()
                .get(p.raw, p.length)
                .map_err(ExternalsError::from)
        } else {
            Err(ExternalsError::from(format!("ptr referencing {} not found", raw_ptr)))
        }
    }

    fn get_memory_ref(&self) -> MemoryRef {
        self.memory_ref.clone()
    }

    pub fn write_data(&mut self, data: Vec<u8>) -> Result<u32, ExternalsError>{

        self.get_memory_ref()
            .set(self.memory_write_offset, &data)?;

        let ptr = Pointer {
            raw: self.memory_write_offset,
            length: data.len(),
            capacity: data.capacity()
        };

        let raw_ptr = ptr.raw;

        self.ptrs.insert(self.memory_write_offset, ptr);
        self.memory_write_offset += data.capacity() as u32;

        Ok(raw_ptr)
    }

    /// Takes a list of pointers and associates them,
    /// effectively creating a list
    ///
    /// Returns a result either containing the raw value
    /// of the first pointer in the list or an externals
    /// error
    pub fn collect_ptrs(&mut self, raw_ptrs: Vec<u32>) -> Result<u32, ExternalsError> {
        info!("associating pointers: {:?}", raw_ptrs);
        if raw_ptrs.iter().all(|x| self.ptrs.contains_key(&x)) {
            self.ptr_collections.insert(raw_ptrs[0], raw_ptrs.clone());
            Ok(raw_ptrs[0])
        } else {
            Err(ExternalsError::from("Attempting to create a ptr collection with nonexistant pointers"))
        }
    }
}

impl Externals for SmartPermissionExternals {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            GET_STATE_IDX => {
                let ptr: i32 = args.nth(0);

                info!("Attempting to get state\nptr: {}", ptr);

                let addr = self.ptr_to_string(ptr as u32)?;

                let state = self.context
                    .get_state(&addr)
                    .map_err(ExternalsError::from)?
                    .unwrap_or(Vec::new());

                let raw_ptr = self.write_data(state)?;

                Ok(Some(RuntimeValue::I32(raw_ptr as i32)))

            },
            GET_PTR_LEN_IDX => {
                let addr = args.nth(0);
                info!("Getting pointer length\nraw {}", addr);

                if let Some(ptr) = self.ptrs.get(&addr) {
                    info!("ptr: {:?}", ptr);
                    Ok(Some(RuntimeValue::I32(ptr.length as i32)))
                } else {
                    Ok(Some(RuntimeValue::I32(-1)))
                }
            },
            GET_PTR_CAP_IDX => {
                let addr = args.nth(0);

                if let Some(ptr) = self.ptrs.get(&addr) {
                    Ok(Some(RuntimeValue::I32(ptr.capacity as i32)))
                } else {
                    Ok(Some(RuntimeValue::I32(-1)))
                }
            },
            ALLOC_IDX => {
                let len: i32 = args.nth(0);

                info!("Allocating memory block of length: {}", len);
                let raw_ptr = self.write_data(vec![0; len as usize])?;
                info!("Block successfully allocated ptr: {}", raw_ptr as i32);

                Ok(Some(RuntimeValue::I32(raw_ptr as i32)))
            },
            READ_BYTE_IDX => {
                let offset: i32 = args.nth(0);
                let byte = self.get_memory_ref()
                    .get(offset as u32, 1)
                    .map_err(ExternalsError::from)?[0];

                Ok(Some(RuntimeValue::I32(byte as i32)))
            },
            WRITE_BYTE_IDX => {
                let ptr: u32 = args.nth(0);
                let offset: u32 = args.nth(1);
                let data: i32 = args.nth(2);

                if let Some(p) = self.ptrs.get(&ptr) {
                    self.get_memory_ref()
                        .set(p.raw + offset, vec![data as u8].as_slice())
                        .map_err(ExternalsError::from)?;

                    Ok(Some(RuntimeValue::I32(1)))
                } else {
                    Ok(Some(RuntimeValue::I32(-1)))
                }
            },
            GET_COLLECTION_LEN_IDX => {
                let head_ptr: u32 = args.nth(0);

                info!("Retrieving collection length. Head pointer {}", head_ptr);

                if let Some(v) = self.ptr_collections.get(&head_ptr) {
                    info!("Collection found elements in collection: {}", v.len());
                    Ok(Some(RuntimeValue::I32(v.len() as i32)))
                } else {
                    Ok(Some(RuntimeValue::I32(-1)))
                }
            },
            GET_PTR_FROM_COLLECTION_IDX => {
                let head_ptr: u32 = args.nth(0);
                let index: u32 = args.nth(1);

                info!("Retrieving pointer head_ptr: {} index: {}", head_ptr, index);

                if let Some(v) = self.ptr_collections.get(&head_ptr) {
                    if index as usize >= v.len() {
                        info!("Invalid index");
                        Ok(Some(RuntimeValue::I32(-1)))
                    } else {
                        info!("Pointer retrieved: {}", v[index as usize]);
                        Ok(Some(RuntimeValue::I32(v[index as usize] as i32)))
                    }
                } else {
                    Ok(Some(RuntimeValue::I32(-1)))
                }
            },
            _ => Err(ExternalsError::to_trap("Function does not exist".to_string()))
        }
    }
}

impl ModuleImportResolver for SmartPermissionExternals {
    fn resolve_func(
        &self,
        field_name: &str,
        _signature: &Signature
    ) -> Result<FuncRef, Error> {
        match field_name {
            "get_state" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
                GET_STATE_IDX)),
            "get_ptr_len" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
                GET_PTR_LEN_IDX)),
            "get_ptr_capacity" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
                GET_PTR_CAP_IDX)),
            "alloc" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
                ALLOC_IDX)),
            "read_byte" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
                READ_BYTE_IDX)),
            "write_byte" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                WRITE_BYTE_IDX)),
            "get_ptr_collection_len" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
                GET_COLLECTION_LEN_IDX)),
            "get_ptr_from_collection" => Ok(FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                GET_PTR_FROM_COLLECTION_IDX)),

            _ => Err(
                Error::Instantiation(format!("Export {} not found", field_name)))
        }
    }

    fn resolve_memory(
        &self,
        field_name: &str,
        _memory_type: &MemoryDescriptor,
    ) -> Result<MemoryRef, Error> {
        match field_name {
            "memory" => Ok(self.get_memory_ref()),
            _ => Err(Error::Instantiation(format!("env module doesn't provide memory '{}'",field_name)))
        }
    }
}

#[derive(Clone)]
struct Pointer {
    raw: u32,
    length: usize,
    capacity: usize
}

impl fmt::Debug for Pointer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Pointer {{ raw: {}, length: {}, capacity {} }}", self.raw, self.length, self.capacity)
    }
}

#[derive(Debug)]
pub struct ExternalsError {
    message: String,
}

impl ExternalsError {
    fn to_trap(msg: String) -> Trap {
        Trap::from(TrapKind::Host(Box::new(ExternalsError::from(msg))))
    }
}

impl fmt::Display for ExternalsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl HostError for ExternalsError {}


impl <'a> From<&'a str> for ExternalsError {
    fn from(s: &'a str) -> Self {
        ExternalsError {
            message: String::from(s)
        }
    }
}

impl From<Error> for ExternalsError {
    fn from(e: Error) -> Self {
        ExternalsError {
            message: format!("{:?}", e)
        }
    }
}


impl From<String> for ExternalsError {
    fn from(s: String) -> Self {
        ExternalsError {
            message: s
        }
    }
}

impl From<FromUtf8Error> for ExternalsError {
    fn from(e: FromUtf8Error) -> Self {
        ExternalsError {
            message: e.description().to_string()
        }
    }
}

impl From<ContextError> for ExternalsError {
    fn from(e: ContextError) -> Self {
        ExternalsError {
            message: format!("{:?}", e)
        }
    }
}
