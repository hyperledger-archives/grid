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
use std::collections::HashMap;

use iron::prelude::*;
use iron::status;
use router::url_for;
use serde::Serializer;

use crate::transaction::{XoState, XoStateError};

use super::{error::StateError, query_param, DataEnvelope, Json, PagedDataEnvelope, State};

/// The handler function for the `/state/:address` endpoint.
pub fn get_state_by_address(req: &mut Request) -> IronResult<Response> {
    let xo_state = req
        .extensions
        .get::<State<XoState>>()
        .expect("Expected xo state, but none was set on the request");

    let address = req
        .extensions
        .get::<router::Router>()
        .expect("Expected router but none was set on the request")
        .find("address")
        .ok_or_else(|| StateError::BadRequest("Missing state address".into()))?;

    if address.len() != 70 {
        return Err(IronError::from(StateError::BadRequest(format!(
            "\"{}\" is not a valid address",
            address
        ))));
    }

    let state_root = xo_state.current_state_root();
    log::debug!(
        "Getting state at {}, from state root {}",
        address,
        &state_root
    );

    let mut params = HashMap::new();
    params.insert("address".to_string(), address.to_string());

    let link = url_for(&req, "get_state", params).to_string();

    let data = xo_state
        .get_state(&state_root, &address)
        .map_err(StateError::from)
        .and_then(|value| {
            value.ok_or_else(|| StateError::NotFound(format!("\"{}\" could not be found", address)))
        })?;
    Ok(Response::with((
        status::Ok,
        Json(DataEnvelope {
            data: base64::encode(&data),
            head: state_root,
            link,
        }),
    )))
}

#[derive(Debug, Default)]
pub struct ListStateRequest {
    address: Option<String>,
    head: Option<String>,
    start: Option<usize>,
    limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct StateEntry {
    address: String,

    #[serde(serialize_with = "as_base64")]
    data: Vec<u8>,
}

/// The handler function for the `/state` endpoint.
pub fn list_state_with_params(req: &mut Request) -> IronResult<Response> {
    let request: ListStateRequest = get_list_params(req)?;
    let xo_state = req
        .extensions
        .get::<State<XoState>>()
        .expect("Expected xo state, but none was set on the request");

    let state_root = request
        .head
        .as_ref()
        .cloned()
        .unwrap_or_else(|| xo_state.current_state_root());

    log::debug!(
        "Listing state with prefix {:?} from head {}",
        request.address.as_ref(),
        &state_root
    );

    let results: Result<Vec<StateEntry>, _> = xo_state
        .list_state(&state_root, request.address.as_ref().map(|s| &**s))
        .map_err(|e| {
            log::error!("Unable to list state: {}", &e);
            StateError::from(e)
        })?
        .map(|entry| entry.map(|(address, data)| StateEntry { address, data }))
        .skip(request.start.unwrap_or(0))
        .take(request.limit.unwrap_or(100))
        .collect();

    let link = url_for(&req, "list_state", into_map(request)).to_string();

    Ok(Response::with((
        status::Ok,
        Json(PagedDataEnvelope::new(
            results.map_err(StateError::from)?,
            state_root,
            link,
            None,
        )),
    )))
}

fn get_list_params(req: &mut Request) -> Result<ListStateRequest, StateError> {
    Ok(ListStateRequest {
        address: query_param(req, "address").unwrap(),
        head: query_param(req, "head").unwrap(),
        start: query_param(req, "start")
            .map_err(|err| StateError::BadRequest(format!("start must be an integer: {}", err)))?,
        limit: query_param(req, "limit")
            .map_err(|err| StateError::BadRequest(format!("limit must be an integer: {}", err)))?,
    })
}

fn into_map(list_params: ListStateRequest) -> HashMap<String, String> {
    let mut list_params = list_params;
    let mut params = HashMap::new();

    if let Some(address) = list_params.address.take() {
        params.insert("address".into(), address);
    }
    if let Some(head) = list_params.head.take() {
        params.insert("head".into(), head);
    }
    if let Some(start) = list_params.start.take() {
        params.insert("start".into(), start.to_string());
    }
    if let Some(limit) = list_params.limit.take() {
        params.insert("limit".into(), limit.to_string());
    }

    params
}

fn as_base64<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&base64::encode(&data))
}

impl From<XoStateError> for StateError {
    fn from(err: XoStateError) -> Self {
        StateError::Internal(err.to_string())
    }
}
