// Copyright 2018-2022 Cargill Incorporated
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

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryPaging {
    pub offset: Option<u64>,
    pub limit: Option<u16>,
}

impl QueryPaging {
    pub fn offset(&self) -> u64 {
        self.offset.unwrap_or(0)
    }

    pub fn limit(&self) -> u16 {
        self.limit
            .map(|l| if l > 1024 { 1024 } else { l })
            .unwrap_or(10)
    }
}
