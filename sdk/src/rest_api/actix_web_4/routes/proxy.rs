// Copyright 2018-2022 Cargill Incorporated
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

use actix_web_4::{http::Method, http::StatusCode, web, HttpRequest, HttpResponse};
use serde_json::value::Value;

use crate::proxy::{
    request::{HeaderList, ProxyMethod, ProxyRequestBuilder},
    response::ProxyResponse,
    ProxyClient,
};
use crate::rest_api::resources::error::ErrorResponse;

pub async fn proxy_get(
    req: HttpRequest,
    proxy_client: web::Data<Box<dyn ProxyClient>>,
) -> HttpResponse {
    proxy_client.proxy(req.into()).into()
}

impl From<HttpRequest> for ProxyRequestBuilder {
    fn from(req: HttpRequest) -> Self {
        let mut headers = req
            .headers()
            .iter()
            .map(|(k, v)| (k.as_str().as_bytes().to_owned(), v.as_bytes().to_owned()))
            .collect::<HeaderList>();
        // Add the `X-Forwarded-For` header to the `ProxyRequest`
        if let Some(addr) = req.head().peer_addr {
            headers.push((
                "X-Forwarded-For".as_bytes().to_owned(),
                addr.ip().to_string().as_bytes().to_owned(),
            ));
        }

        let mut builder = ProxyRequestBuilder::default()
            .with_headers(headers)
            .with_path(req.path().to_string())
            .with_method(ProxyMethod::from(req.method()));

        if !req.query_string().is_empty() {
            builder = builder.with_query_params(req.query_string().to_string());
        }

        builder
    }
}

impl From<&Method> for ProxyMethod {
    fn from(actix_method: &Method) -> Self {
        match *actix_method {
            Method::GET => ProxyMethod::Get,
            Method::POST => ProxyMethod::Post,
            Method::PUT => ProxyMethod::Put,
            Method::DELETE => ProxyMethod::Delete,
            Method::CONNECT => ProxyMethod::Connect,
            Method::HEAD => ProxyMethod::Head,
            Method::OPTIONS => ProxyMethod::Options,
            Method::PATCH => ProxyMethod::Patch,
            Method::TRACE => ProxyMethod::Trace,
            ref other => ProxyMethod::Custom(other.as_str().as_bytes().to_owned()),
        }
    }
}

impl From<ProxyResponse> for HttpResponse {
    fn from(p_resp: ProxyResponse) -> Self {
        let status = StatusCode::from_u16(p_resp.status_code()).unwrap_or(StatusCode::BAD_GATEWAY);
        if !p_resp.body().content().is_empty() {
            match serde_json::from_slice(p_resp.body().content()) {
                Ok(json) => {
                    let data: Value = json;
                    HttpResponse::build(status).json(data)
                }
                Err(err) => HttpResponse::build(StatusCode::BAD_GATEWAY).json({
                    ErrorResponse::new(
                        502,
                        &format!(
                            "Received {status} from proxy, \
                            but failed to retrieve response content: {err}"
                        ),
                    )
                }),
            }
        } else {
            HttpResponse::build(status).finish()
        }
    }
}
