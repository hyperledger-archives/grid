// Copyright 2018-2021 Cargill Incorporated
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

use crate::paging;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Paging {
    current: String,
    offset: i64,
    limit: i64,
    total: i64,
    first: String,
    prev: String,
    next: Option<String>,
    last: String,
}

impl Paging {
    pub fn new(base_link: &str, paging: paging::Paging, service_id: Option<&str>) -> Self {
        let current = if let Some(service_id) = service_id {
            format!(
                "{}?offset={}&limit={}&service_id={}",
                base_link, paging.offset, paging.limit, service_id
            )
        } else {
            format!(
                "{}?offset={}&limit={}",
                base_link, paging.offset, paging.limit
            )
        };
        let first = if let Some(service_id) = service_id {
            format!(
                "{}?offset=0&limit={}&service_id={}",
                base_link, paging.limit, service_id
            )
        } else {
            format!("{}?offset=0&limit={}", base_link, paging.limit)
        };
        let previous_offset = if paging.offset > paging.limit {
            paging.offset - paging.limit
        } else {
            0
        };
        let prev = if let Some(service_id) = service_id {
            format!(
                "{}?offset={}&limit={}&service_id={}",
                base_link, previous_offset, paging.limit, service_id
            )
        } else {
            format!(
                "{}?offset={}&limit={}",
                base_link, previous_offset, paging.limit
            )
        };

        let last_offset = if paging.total > 0 {
            ((paging.total - 1) / paging.limit) * paging.limit
        } else {
            0
        };
        let last = if let Some(service_id) = service_id {
            format!(
                "{}?offset={}&limit={}&service_id={}",
                base_link, last_offset, paging.limit, service_id
            )
        } else {
            format!(
                "{}?offset={}&limit={}",
                base_link, last_offset, paging.limit
            )
        };

        let next_offset = if paging.offset + paging.limit > last_offset {
            last_offset
        } else {
            paging.offset + paging.limit
        };

        let next = if let Some(service_id) = service_id {
            format!(
                "{}?offset={}&limit={}&service_id={}",
                base_link, next_offset, paging.limit, service_id
            )
        } else {
            format!(
                "{}?offset={}&limit={}",
                base_link, next_offset, paging.limit
            )
        };

        Paging {
            current,
            offset: paging.offset,
            limit: paging.limit,
            total: paging.total,
            first,
            prev,
            next: if next == last { None } else { Some(next) },
            last,
        }
    }
}
