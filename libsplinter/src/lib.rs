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

extern crate atomicwrites;
extern crate bimap;
extern crate bytes;
extern crate protobuf;
extern crate rustls;
extern crate webpki;
#[macro_use]
extern crate log;
extern crate byteorder;
extern crate messaging;
extern crate mio;
extern crate openssl;
extern crate serde;
extern crate serde_yaml;
extern crate url;
#[macro_use]
extern crate serde_derive;
extern crate crossbeam_channel;
extern crate mio_extras;
#[cfg(test)]
extern crate tempdir;
extern crate uuid;

#[macro_export]
macro_rules! rwlock_read_unwrap {
    ($lock:expr) => {
        match $lock.read() {
            Ok(d) => d,
            Err(e) => panic!("RwLock error: {:?}", e),
        }
    };
}

#[macro_export]
macro_rules! rwlock_write_unwrap {
    ($lock:expr) => {
        match $lock.write() {
            Ok(d) => d,
            Err(e) => panic!("RwLock error: {:?}", e),
        }
    };
}

pub mod circuit;
pub mod mesh;
pub mod network;
pub mod storage;
pub mod transport;
