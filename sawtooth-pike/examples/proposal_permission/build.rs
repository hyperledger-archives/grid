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

extern crate protoc_rust;

use std::fs;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    fs::create_dir_all("src/protos").unwrap();
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/protos",
        input: &[
            "../../../protos/payload.proto",
            "../../../protos/agent.proto",
            "../../../protos/property.proto",
            "../../../protos/proposal.proto",
            "../../../protos/record.proto"
        ],
        includes: &["../../../protos"],
    }).expect("protoc");

    let mut file = File::create("src/protos/mod.rs").unwrap();
    file.write_all(b"pub mod payload;\n").unwrap();
    file.write_all(b"pub mod agent;\n").unwrap();
    file.write_all(b"pub mod property;\n").unwrap();
    file.write_all(b"pub mod proposal;\n").unwrap();
    file.write_all(b"pub mod record;\n").unwrap();
}
