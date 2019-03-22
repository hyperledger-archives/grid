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

extern crate rocket;
extern crate serde_yaml;
extern crate serde_json;

use std::fs::File;
use std::io::prelude::*;

use serde_json::Value;

const SWAGGER_FILENAME: &'static str = "openapi.yaml";

#[get("/openapi.json")]
fn openapi_json() -> String {
    let mut file = File::open(SWAGGER_FILENAME).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let deserialized_map: Value = serde_yaml::from_str(&contents).unwrap();
    let j = serde_json::to_string_pretty(&deserialized_map).unwrap();

    return j
}

#[get("/openapi.yaml")]
fn openapi_yaml() -> String {
    let mut file = File::open(SWAGGER_FILENAME).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    return contents;
}
