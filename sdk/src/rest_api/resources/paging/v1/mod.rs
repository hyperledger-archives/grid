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
use std::cmp;
use url::Url;

/// Paging data for a REST API dataset, intended to be returned with REST response data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Paging {
    /// URL for the current page of records
    current: Url,

    /// Numerical index of the first record
    offset: i64,

    /// Max number of records per page
    limit: i64,

    /// Total number of records
    total: i64,

    /// URL for the first page of records
    first: Url,

    /// URL for the previous page of records
    prev: Option<Url>,

    /// URL for the next page of records, if one exists
    next: Option<Url>,

    /// URL for the last page of records
    last: Url,
}

impl Paging {
    /// Create a new Paging object
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL for paging
    /// * `paging` - Struct with dataset size, page size, and index
    /// * `service_id` - The service id on the circuit, if applicable
    pub fn new(mut base_url: Url, paging: paging::Paging, service_id: Option<&str>) -> Self {
        let limit = paging.limit.to_string();
        base_url.query_pairs_mut().append_pair("limit", &limit);

        if let Some(service_id) = service_id {
            base_url
                .query_pairs_mut()
                .append_pair("service_id", service_id);
        }

        let generator = PageUrlGenerator { url: base_url };
        let offsets = Offsets::new(&paging);

        Paging {
            current: generator.url_with_offset(paging.offset),
            offset: paging.offset,
            limit: paging.limit,
            total: paging.total,
            first: generator.url_with_offset(offsets.first),
            prev: offsets.prev.map(|v| generator.url_with_offset(v)),
            last: generator.url_with_offset(offsets.last),
            next: offsets.next.map(|v| generator.url_with_offset(v)),
        }
    }
}

/// Numerical representation of pagination offsets for any given page
#[derive(Debug, Eq, PartialEq)]
struct Offsets {
    /// Offset for first page of records
    first: i64,

    /// Offset for the previous page of records
    prev: Option<i64>,

    /// Offset for the next page of records, if one exists
    next: Option<i64>,

    /// Offset for the last page of records
    last: i64,
}

impl Offsets {
    fn new(paging: &paging::Paging) -> Self {
        let last_offset = cmp::max(((paging.total - 1) / paging.limit) * paging.limit, 0);

        Offsets {
            first: 0,
            prev: if paging.offset <= 0 {
                None
            } else if paging.offset > paging.total {
                Some(last_offset)
            } else {
                Some(cmp::max(paging.offset - paging.limit, 0))
            },
            last: last_offset,
            next: if paging.offset >= last_offset {
                None
            } else {
                Some(if paging.offset + paging.limit > last_offset {
                    last_offset
                } else {
                    paging.offset + paging.limit
                })
            },
        }
    }
}

/// Utility to generate a URL at a given offset
struct PageUrlGenerator {
    url: Url,
}

impl PageUrlGenerator {
    fn url_with_offset<T: ToString>(&self, offset: T) -> Url {
        let mut url = self.url.clone();
        url.query_pairs_mut()
            .append_pair("offset", &offset.to_string());
        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_second_page() {
        assert_eq!(
            Offsets::new(&paging::Paging {
                offset: 20,
                limit: 10,
                total: 80
            }),
            Offsets {
                first: 0,
                prev: Some(10),
                next: Some(30),
                last: 70
            }
        );
    }

    #[test]
    fn test_offset_first_page() {
        assert_eq!(
            Offsets::new(&paging::Paging {
                offset: 0,
                limit: 10,
                total: 80
            }),
            Offsets {
                first: 0,
                prev: None,
                next: Some(10),
                last: 70
            }
        );
    }

    #[test]
    fn test_offset_last_page() {
        assert_eq!(
            Offsets::new(&paging::Paging {
                offset: 70,
                limit: 10,
                total: 80
            }),
            Offsets {
                first: 0,
                prev: Some(60),
                next: None,
                last: 70
            }
        );
    }

    #[test]
    fn test_offset_beyond_limit_prev_is_offset_to_last_page() {
        assert_eq!(
            Offsets::new(&paging::Paging {
                offset: 100,
                limit: 10,
                total: 80
            }),
            Offsets {
                first: 0,
                prev: Some(70),
                next: None,
                last: 70
            }
        );
    }

    #[test]
    fn test_offset_not_aligned_with_limit() {
        assert_eq!(
            Offsets::new(&paging::Paging {
                offset: 5,
                limit: 10,
                total: 80
            }),
            Offsets {
                first: 0,
                prev: Some(0),
                next: Some(15),
                last: 70
            }
        );
    }

    #[test]
    fn test_offset_total_smaller_than_limit() {
        assert_eq!(
            Offsets::new(&paging::Paging {
                offset: 5,
                limit: 100,
                total: 80
            }),
            Offsets {
                first: 0,
                prev: Some(0),
                next: None,
                last: 0
            }
        );
    }

    #[test]
    fn test_paging_absolute_url() {
        assert_eq!(
            Paging::new(
                Url::parse("http://base/").unwrap(),
                paging::Paging {
                    offset: 20,
                    limit: 10,
                    total: 80
                },
                Some("fakeserviceid"),
            ),
            Paging {
                current: Url::parse("http://base/?limit=10&service_id=fakeserviceid&offset=20")
                    .unwrap(),
                offset: 20,
                limit: 10,
                total: 80,
                first: Url::parse("http://base/?limit=10&service_id=fakeserviceid&offset=0")
                    .unwrap(),
                prev: Some(
                    Url::parse("http://base/?limit=10&service_id=fakeserviceid&offset=10").unwrap()
                ),
                next: Some(
                    Url::parse("http://base/?limit=10&service_id=fakeserviceid&offset=30").unwrap()
                ),
                last: Url::parse("http://base/?limit=10&service_id=fakeserviceid&offset=70")
                    .unwrap(),
            }
        );
    }

    #[test]
    fn test_paging_query_params() {
        assert_eq!(
            Paging::new(
                Url::parse("http://base/?unrelated_filter=9").unwrap(),
                paging::Paging {
                    offset: 20,
                    limit: 10,
                    total: 80
                },
                Some("fakeserviceid"),
            ),
            Paging {
                current: Url::parse(
                    "http://base/?unrelated_filter=9&limit=10&service_id=fakeserviceid&offset=20"
                ).unwrap(),
                offset: 20,
                limit: 10,
                total: 80,
                first: Url::parse(
                    "http://base/?unrelated_filter=9&limit=10&service_id=fakeserviceid&offset=0"
                ).unwrap(),
                prev: Some(Url::parse(
                    "http://base/?unrelated_filter=9&limit=10&service_id=fakeserviceid&offset=10"
                ).unwrap()),
                next: Some(Url::parse(
                    "http://base/?unrelated_filter=9&limit=10&service_id=fakeserviceid&offset=30"
                ).unwrap()),
                last: Url::parse(
                    "http://base/?unrelated_filter=9&limit=10&service_id=fakeserviceid&offset=70"
                ).unwrap(),
            }
        );
    }
}
