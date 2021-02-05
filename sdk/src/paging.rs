// Copyright 2018-2020 Cargill Incorporated
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

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Paging {
    pub offset: i64,
    pub limit: i64,
    pub total: i64,
}

impl Paging {
    pub fn new(offset: i64, limit: i64, total: i64) -> Self {
        Paging {
            offset,
            limit,
            total,
        }
    }
}
