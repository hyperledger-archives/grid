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

use crate::rest_api::error::RestApiServerError;
use actix_web::{HttpRequest, HttpResponse};

#[derive(Debug, Serialize, Deserialize)]
struct Status {
    version: String,
}

#[get("/status")]
pub fn get_status(_: HttpRequest) -> HttpResponse {
    let status = Status {
        version: get_version(),
    };
    HttpResponse::Ok().json(status)
}

#[get("/openapi.yml")]
pub fn get_openapi(_: HttpRequest) -> Result<HttpResponse, RestApiServerError> {
    Ok(HttpResponse::Ok().body(include_str!("../../api/static/openapi.yml")))
}

fn get_version() -> String {
    format!(
        "{}.{}.{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH")
    )
}
