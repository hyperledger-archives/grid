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

pub mod batches;
mod error;
pub mod state;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DataEnvelope<T: Serialize> {
    data: T,
    head: String,
    link: String,
}

impl<T: Serialize> DataEnvelope<T> {
    pub fn new(data: T, link: String, head: String) -> Self {
        DataEnvelope { data, link, head }
    }
}

#[derive(Debug, Serialize)]
pub struct Paging {
    start: String,
    limit: i32,
    next_position: String,
    next: String,
}

#[derive(Debug, Serialize)]
pub struct PagedDataEnvelope<T: Serialize> {
    data: Vec<T>,
    head: String,
    link: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    paging: Option<Paging>,
}

impl<T: Serialize> PagedDataEnvelope<T> {
    pub fn new(data: Vec<T>, head: String, link: String, paging: Option<Paging>) -> Self {
        PagedDataEnvelope {
            data,
            head,
            link,
            paging,
        }
    }
}
