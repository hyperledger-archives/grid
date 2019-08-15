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

use libsplinter::actix_web::{web, Error, HttpRequest, HttpResponse};
use libsplinter::futures::{Future, IntoFuture};

#[derive(Debug, Serialize, Deserialize)]
struct Status {
    node_id: String,
    endpoint: String,
    version: String,
}

pub fn get_status(
    node_id: String,
    endpoint: String,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let status = Status {
        node_id,
        endpoint,
        version: get_version(),
    };

    Box::new(HttpResponse::Ok().json(status).into_future())
}

pub fn get_openapi(
    _: HttpRequest,
    _: web::Payload,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(
        HttpResponse::Ok()
            .body(include_str!("../../api/static/openapi.yml"))
            .into_future(),
    )
}

fn get_version() -> String {
    format!(
        "{}.{}.{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH")
    )
}
