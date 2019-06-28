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

use std::any::Any;

use iron::mime::Mime;
use iron::prelude::*;
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

pub struct State<T: Any> {
    state: T,
}

impl<T: Any> State<T> {
    pub fn new(state: T) -> Self {
        Self { state }
    }
}

impl<'a, 'b, T: Any> iron::modifier::Modifier<Request<'a, 'b>> for State<T> {
    fn modify(self, req: &mut Request<'a, 'b>) {
        req.extensions.insert::<State<T>>(self.state);
    }
}

impl<T: Any> iron::typemap::Key for State<T> {
    type Value = T;
}

impl<T: Any> std::ops::Deref for State<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

struct Json<T>(T)
where
    T: Serialize;

impl<T: Serialize> iron::modifier::Modifier<Response> for Json<T> {
    fn modify(self, res: &mut Response) {
        let content_type = "application/json"
            .parse::<Mime>()
            .expect("Unable to create basic mime type");
        content_type.modify(res);

        let output = serde_json::to_string(&self.0).expect("Unable to convert to Json");
        res.body = Some(Box::new(output));
    }
}

pub fn query_param<T: std::str::FromStr>(
    req: &mut Request,
    key: &str,
) -> Result<Option<T>, T::Err> {
    let mut params = query_params(req, key)?;

    if let Some(mut values) = params.take() {
        Ok(values.pop())
    } else {
        Ok(None)
    }
}

pub fn query_params<T: std::str::FromStr>(
    req: &mut Request,
    key: &str,
) -> Result<Option<Vec<T>>, T::Err> {
    match req.get_ref::<urlencoded::UrlEncodedQuery>() {
        Ok(ref query) => match query.get(key) {
            Some(values) => Ok(Some(
                values
                    .iter()
                    .map(|s| s.parse())
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            None => Ok(None),
        },
        Err(_) => Ok(None),
    }
}
