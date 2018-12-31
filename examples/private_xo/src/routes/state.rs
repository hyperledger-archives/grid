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
use rocket_contrib::json::Json;

use crate::routes::error::StateError;
use crate::routes::{DataEnvelope, PagedDataEnvelope};

#[get("/state/<address>")]
pub fn get_state_by_address(address: String) -> Result<Json<DataEnvelope<String>>, StateError> {
    if address.len() != 70 {
        return Err(StateError::BadRequest(format!(
            "\"{}\" is not a valid address",
            address
        )));
    }

    log::debug!("Getting state at {}", &address);

    Err(StateError::NotFound(format!(
        "\"{}\" could not be found",
        address
    )))
}

#[derive(Debug, Default, FromForm, UriDisplayQuery)]
pub struct ListStateRequest {
    address: Option<String>,
    head: Option<String>,
    start: Option<String>,
    limit: Option<i32>,
}

impl Ignorable<Query> for ListStateRequest {}

#[derive(Debug, Serialize)]
pub struct StateEntry {}

#[get("/state?<parameters..>")]
pub fn list_state_with_params(
    parameters: Form<ListStateRequest>,
) -> Result<Json<PagedDataEnvelope<StateEntry>>, StateError> {
    let request = parameters.into_inner();

    log::debug!("Listing state with prefix {:?}", request.address.as_ref());

    Ok(Json(PagedDataEnvelope::new(
        vec![],
        "".into(),
        uri!(list_state_with_params: parameters = request).to_string(),
        None,
    )))
}
