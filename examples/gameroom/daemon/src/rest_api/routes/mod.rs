// Copyright 2019 Cargill Incorporated
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

mod authenticate;
mod gameroom;
mod node;
mod proposal;

pub use authenticate::*;
pub use gameroom::*;
pub use node::*;
pub use proposal::*;

use percent_encoding::{AsciiSet, CONTROLS};

pub const DEFAULT_LIMIT: usize = 100;
pub const DEFAULT_OFFSET: usize = 0;
const QUERY_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'=')
    .add(b'!')
    .add(b'{')
    .add(b'}')
    .add(b'[')
    .add(b']')
    .add(b':')
    .add(b',');

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Paging {
    current: String,
    offset: usize,
    limit: usize,
    total: usize,
    first: String,
    prev: String,
    next: String,
    last: String,
}
