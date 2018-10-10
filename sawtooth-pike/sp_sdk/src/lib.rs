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

extern crate protobuf;

mod externs;

use std::string::FromUtf8Error;

pub use externs::{WasmPtr, WasmPtrList};

pub struct Request {
    roles: Vec<String>,
    org_id: String,
    public_key: String,
    payload: Vec<u8>
}

impl Request {
    pub fn new(
        roles: Vec<String>,
        org_id: String,
        public_key: String,
        payload: Vec<u8>
    ) -> Request {
        Request {
            roles,
            org_id,
            public_key,
            payload
        }
    }

    pub fn get_roles(&self) -> Vec<String> {
        self.roles.clone()
    }

    pub fn get_org_id(&self) -> String {
        self.org_id.clone()
    }

    pub fn get_public_key(&self) -> String {
        self.public_key.clone()
    }

    pub fn get_state(&self, address: String) -> Result<Vec<u8>, WasmSdkError> {
        unsafe {
            let wasm_buffer = WasmBuffer::new(address.as_bytes())?;
            ptr_to_vec(externs::get_state(wasm_buffer.into_raw()))
        }
    }

    pub fn get_payload<T>(&self) -> Vec<u8> {
        self.payload.clone()
    }
}

/// Error Codes:
///
/// -1: Failed to deserialize roles
/// -2: Failed to deserialize org_id
/// -3: Failed to deserialize public_key
/// -4: Failed to deserialize payload
/// -5: Failed to execute smart permission
/// -6: StateSetError
/// -7: AllocError
/// -8: MemoryRetrievalError
/// -9: Utf8EncodeError
/// -10: ProtobufError
///
pub unsafe fn execute_entrypoint<F>(
    roles_ptr: WasmPtrList,
    org_id_ptr: WasmPtr,
    public_key_ptr: WasmPtr,
    payload_ptr: WasmPtr,
    has_permission: F
) -> i32
where F: Fn(Request) -> Result<bool, WasmSdkError> {
    let roles = if let Ok(i) = WasmBuffer::from_list(roles_ptr) {
        let results: Vec<Result<String, WasmSdkError>> = i
            .iter()
            .map(|x| x.into_string())
            .collect();

        if results.iter().any(|x| x.is_err()) {
            return -1;
        } else {
            results
                .into_iter()
                .map(|x| x.unwrap())
                .collect()
        }

    } else {
        return -1;
    };

    let org_id = if let Ok(i) = WasmBuffer::from_raw(org_id_ptr) {
        match i.into_string() {
            Ok(s) => s,
            Err(_) => {
                return -2;
            }
        }
    } else {
        return -2;
    };

    let public_key = if let Ok(i) = WasmBuffer::from_raw(public_key_ptr) {
        match i.into_string() {
            Ok(s) => s,
            Err(_) => {
                return -3;
            }
        }
    } else {
        return -3;
    };

    let payload = if let Ok(i) = WasmBuffer::from_raw(payload_ptr) {
        i.into_bytes()
    } else {
        return -4;
    };

    match has_permission(Request::new(roles, org_id, public_key, payload)) {
        Ok(r) => if r {
            1
        } else {
            0
        },
        Err(WasmSdkError::StateSetError(_)) => -5,
        Err(WasmSdkError::AllocError(_)) => -6,
        Err(WasmSdkError::MemoryWriteError(_)) => -7,
        Err(WasmSdkError::MemoryRetrievalError(_)) => -8,
        Err(WasmSdkError::Utf8EncodeError(_)) => -9,
        Err(WasmSdkError::ProtobufError(_)) => -10
    }
}

/// A WasmBuffer is a wrapper around a wasm pointer.
///
/// It contains a raw wasm pointer to location in executor
/// memory and a bytes repesentation of it's contents.
///
/// It offers methods for accessing the data stored at the
/// location referenced by the raw pointer.
///
pub struct WasmBuffer {
    raw: WasmPtr,
    data: Vec<u8>
}

impl WasmBuffer {
    pub unsafe fn new(buffer: &[u8]) -> Result<WasmBuffer, WasmSdkError> {
        let raw = externs::alloc(buffer.len());

        if raw < 0 {
            return Err(WasmSdkError::AllocError("Failed to allocate host memory".into()));
        }

        for i in 0..buffer.len() {
            if externs::write_byte(raw, i as u32, buffer[i]) < 0 {
                return Err(WasmSdkError::MemoryWriteError("Failed to write data to host memory".into()));
            }
        }

        Ok(WasmBuffer {
            raw,
            data: buffer.clone().to_vec()
        })
    }

    pub unsafe fn from_raw(raw: WasmPtr) -> Result<WasmBuffer, WasmSdkError> {
        let data = ptr_to_vec(raw)?;
        Ok(WasmBuffer { raw, data })
    }

    pub unsafe fn from_list(ptr: WasmPtrList) -> Result<Vec<WasmBuffer>, WasmSdkError> {
        let mut wasm_buffers = Vec::new();

        if ptr >= 0 {
            for i in 0..externs::get_ptr_collection_len(ptr) {
                let ptr = externs::get_ptr_from_collection(ptr, i as u32);

                if ptr < 0 {
                    return Err(WasmSdkError::MemoryRetrievalError("pointer not found".into()));
                }
                wasm_buffers.push(WasmBuffer::from_raw(ptr)?);
            }
        }

        Ok(wasm_buffers)
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn into_raw(&self) -> WasmPtr {
        self.raw
    }

    pub fn into_string(&self) -> Result<String, WasmSdkError> {
        String::from_utf8(self.data.clone())
            .map_err(WasmSdkError::from)
    }
}

#[derive(Debug)]
pub enum WasmSdkError {
    StateSetError(String),
    AllocError(String),
    MemoryWriteError(String),
    MemoryRetrievalError(String),
    Utf8EncodeError(FromUtf8Error),
    ProtobufError(protobuf::ProtobufError)
}

impl From<FromUtf8Error> for WasmSdkError {
    fn from(e: FromUtf8Error) -> Self {
        WasmSdkError::Utf8EncodeError(e)
    }
}

impl From<protobuf::ProtobufError> for WasmSdkError {
    fn from(e: protobuf::ProtobufError) -> Self {
        WasmSdkError::ProtobufError(e)
    }
}

unsafe fn ptr_to_vec(ptr: WasmPtr) -> Result<Vec<u8>, WasmSdkError> {
    let mut vec = Vec::new();

    for i in 0..externs::get_ptr_len(ptr) {
        vec.push(externs::read_byte(ptr as isize + i));
    }

    Ok(vec)
}
