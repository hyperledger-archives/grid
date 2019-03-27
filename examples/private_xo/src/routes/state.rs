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

use rocket::http::uri::{Ignorable, Query};
use rocket::request::Form;
use rocket::State;
use rocket_contrib::json::Json;
use serde::Serializer;

use crate::routes::error::StateError;
use crate::routes::{DataEnvelope, PagedDataEnvelope};
use crate::transaction::{XoState, XoStateError};

#[get("/state/<address>")]
pub fn get_state_by_address(
    xo_state: State<XoState>,
    address: String,
) -> Result<Json<DataEnvelope<String>>, StateError> {
    if address.len() != 70 {
        return Err(StateError::BadRequest(format!(
            "\"{}\" is not a valid address",
            address
        )));
    }

    let state_root = xo_state.current_state_root();
    log::debug!(
        "Getting state at {}, from state root {}",
        &address,
        &state_root
    );

    xo_state
        .get_state(&state_root, &address)
        .map_err(StateError::from)
        .and_then(|value| {
            value.ok_or_else(|| StateError::NotFound(format!("\"{}\" could not be found", address)))
        })
        .map(|data| {
            Json(DataEnvelope {
                data: base64::encode(&data),
                head: state_root,
                link: uri!(get_state_by_address: address = address).to_string(),
            })
        })
}

#[derive(Debug, Default, FromForm, UriDisplayQuery)]
pub struct ListStateRequest {
    address: Option<String>,
    head: Option<String>,
    start: Option<usize>,
    limit: Option<usize>,
}

impl Ignorable<Query> for ListStateRequest {}

#[derive(Debug, Serialize)]
pub struct StateEntry {
    address: String,

    #[serde(serialize_with = "as_base64")]
    data: Vec<u8>,
}

#[get("/state?<parameters..>")]
pub fn list_state_with_params(
    xo_state: State<XoState>,
    parameters: Form<ListStateRequest>,
) -> Result<Json<PagedDataEnvelope<StateEntry>>, StateError> {
    let request = parameters.into_inner();

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
            e
        })?
        .map(|entry| entry.map(|(address, data)| StateEntry { address, data }))
        .skip(request.start.unwrap_or(0))
        .take(request.limit.unwrap_or(100))
        .collect();

    Ok(Json(PagedDataEnvelope::new(
        results.map_err(StateError::from)?,
        state_root,
        uri!(list_state_with_params: parameters = request).to_string(),
        None,
    )))
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
