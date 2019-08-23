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

use actix_web::{client::Client, dev::Body, http::StatusCode, web, Error, HttpResponse};
use futures::Future;

pub fn submit_signed_payload(
    client: web::Data<Client>,
    splinterd_url: web::Data<String>,
    signed_payload: web::Bytes,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(
        client
            .post(format!("{}/admin/submit", *splinterd_url))
            .send_body(Body::Bytes(signed_payload))
            .map_err(Error::from)
            .and_then(|mut resp| {
                let status = resp.status();
                let body = resp.body().wait()?;
                match status {
                    StatusCode::ACCEPTED => Ok(HttpResponse::Accepted().json(json!({
                        "message": "The payload was submitted successfully"
                    }))),
                    StatusCode::BAD_REQUEST => Ok(HttpResponse::BadRequest().body(body)),
                    _ => Ok(HttpResponse::InternalServerError().finish()),
                }
            }),
    )
}
