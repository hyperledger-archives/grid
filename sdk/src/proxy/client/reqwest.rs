// Copyright 2022 Cargill Incorporated
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

use std::convert::TryFrom;
use std::iter::FromIterator;

use reqwest::{
    blocking::{Body, Client as ReqwestBlockingClient, RequestBuilder, Response},
    header::{HeaderMap, HeaderName, HeaderValue},
    Method,
};
use url::Url;

use crate::error::{InternalError, InvalidArgumentError};
use crate::proxy::{
    client::ProxyClient,
    error::ProxyError,
    request::{ProxyMethod, ProxyRequest, ProxyRequestBuilder},
    response::ProxyResponse,
};
use crate::rest_api::resources::error::ErrorResponse;

/// The Reqwest implementation of the Proxy client
pub struct ReqwestProxyClient {
    url: Url,
}

impl ReqwestProxyClient {
    pub fn new(url: &str) -> Result<Self, ProxyError> {
        Ok(Self {
            url: Url::parse(url).map_err(|err| {
                ProxyError::InvalidArgumentError(InvalidArgumentError::new(
                    "url".to_string(),
                    format!("Failed to parse url: {err}"),
                ))
            })?,
        })
    }
}

impl ProxyClient for ReqwestProxyClient {
    fn proxy(&self, req_builder: ProxyRequestBuilder) -> ProxyResponse {
        let orig_request = match req_builder.with_uri(self.url.to_string()).build() {
            Ok(req) => req,
            Err(err) => {
                return err.into();
            }
        };

        let request = match RequestBuilder::try_from(orig_request) {
            Ok(req) => req,
            Err(err) => {
                return err.into();
            }
        };

        match request.send() {
            Ok(res) => ProxyResponse::from(res),
            Err(err) => ProxyResponse::new(
                502,
                format!("Request failed to send: {err}")
                    .as_bytes()
                    .to_owned(),
            ),
        }
    }

    fn cloned_box(&self) -> Box<dyn ProxyClient> {
        Box::new(self.clone())
    }
}

impl Clone for ReqwestProxyClient {
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
        }
    }
}

impl TryFrom<&ProxyMethod> for Method {
    type Error = ProxyError;

    fn try_from(method: &ProxyMethod) -> Result<Self, Self::Error> {
        let method = match *method {
            ProxyMethod::Get => Method::GET,
            ProxyMethod::Post => Method::POST,
            ProxyMethod::Put => Method::PUT,
            ProxyMethod::Delete => Method::DELETE,
            ProxyMethod::Connect => Method::CONNECT,
            ProxyMethod::Head => Method::HEAD,
            ProxyMethod::Options => Method::OPTIONS,
            ProxyMethod::Patch => Method::PATCH,
            ProxyMethod::Trace => Method::TRACE,
            ProxyMethod::Custom(ref bytes) => Method::from_bytes(bytes).map_err(|err| {
                ProxyError::InvalidArgumentError(InvalidArgumentError::new(
                    "method".to_string(),
                    format!("Invalid request method: {err}"),
                ))
            })?,
        };

        Ok(method)
    }
}

impl TryFrom<ProxyRequest> for RequestBuilder {
    type Error = ProxyError;

    fn try_from(req: ProxyRequest) -> Result<Self, Self::Error> {
        let mut url = Url::parse(req.uri()).map_err(|err| {
            ProxyError::InvalidArgumentError(InvalidArgumentError::new(
                "url".to_string(),
                format!("Unable to parse request URI: {err}"),
            ))
        })?;
        url.set_path(req.path());
        url.set_query(req.query_params());

        let mut request_builder =
            ReqwestBlockingClient::default().request(Method::try_from(req.method())?, url);

        // Generate Headers for the request
        let conv_headers = req
            .headers()
            .iter()
            .map(|(k, v)| {
                let header_name = HeaderName::from_bytes(k).map_err(|err| {
                    ProxyError::InternalError(InternalError::with_message(format!(
                        "Failed to parse header name: {err}"
                    )))
                })?;
                let header_value = HeaderValue::from_bytes(v).map_err(|err| {
                    ProxyError::InternalError(InternalError::with_message(format!(
                        "Failed to parse header value: {err}"
                    )))
                })?;
                Ok((header_name, header_value))
            })
            .collect::<Result<Vec<(_, _)>, ProxyError>>()?;
        let headers = HeaderMap::from_iter(conv_headers);
        // Set the request headers
        request_builder = request_builder.headers(headers);
        // Set the request body
        if let Some(body) = req.body() {
            request_builder = request_builder.body(Body::from(body.content().to_owned()));
        }
        Ok(request_builder)
    }
}

impl From<ErrorResponse> for ProxyResponse {
    fn from(err: ErrorResponse) -> Self {
        ProxyResponse::new(err.status_code(), err.message().as_bytes().to_owned())
    }
}

impl From<Response> for ProxyResponse {
    fn from(resp: Response) -> Self {
        let resp_status = resp.status().as_u16();
        let (status_code, response): (u16, Vec<u8>) = match resp.bytes() {
            Ok(bytes) => (resp_status, (bytes.to_vec())),
            Err(err) => (
                502,
                format!(
                    "Received status {resp_status} from upstream server, \
                but failed to load response content: {err}"
                )
                .as_bytes()
                .to_owned(),
            ),
        };
        ProxyResponse::new(status_code, response)
    }
}
