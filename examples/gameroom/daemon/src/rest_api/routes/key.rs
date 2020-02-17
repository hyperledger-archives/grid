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

use actix_web::{client::Client, dev::Body, http::StatusCode, web, Error, HttpResponse};
use splinter::protocol;

use super::ErrorResponse;

pub async fn fetch_key_info(
    client: web::Data<Client>,
    splinterd_url: web::Data<String>,
    public_key: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let public_key = public_key.into_inner();

    let mut response = client
        .get(format!("{}/keys/{}", splinterd_url.get_ref(), public_key))
        .header(
            "SplinterProtocolVersion",
            protocol::ADMIN_PROTOCOL_VERSION.to_string(),
        )
        .send()
        .await?;

    let body = response.body().await?;

    match response.status() {
        StatusCode::OK => Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(Body::Bytes(body))),
        StatusCode::NOT_FOUND => Ok(HttpResponse::NotFound().json(ErrorResponse::not_found(
            &format!("Could not find user information of key {}", public_key),
        ))),
        StatusCode::BAD_REQUEST => {
            let body_value: serde_json::Value = serde_json::from_slice(&body)?;
            let message = match body_value.get("message") {
                Some(value) => value.as_str().unwrap_or("Request was malformed."),
                None => "Request malformed.",
            };
            Ok(HttpResponse::BadRequest().json(ErrorResponse::bad_request(&message)))
        }
        _ => {
            debug!(
                "Internal Server Error. Splinterd responded with error {}",
                response.status(),
            );
            Ok(HttpResponse::InternalServerError().json(ErrorResponse::internal_error()))
        }
    }
}
