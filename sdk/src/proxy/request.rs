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

//! Representation of an HTTP request and its parts

use crate::error::InvalidArgumentError;
use crate::proxy::error::ProxyError;

/// Represents an HTTP request, to be used when creating a request to be proxied
pub struct ProxyRequest {
    headers: HeaderList,
    uri: String,
    path: String,
    query_params: Option<String>,
    body: Option<ProxyRequestBody>,
    method: ProxyMethod,
}

impl ProxyRequest {
    pub fn headers(&self) -> &HeaderList {
        &self.headers
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn query_params(&self) -> Option<&str> {
        self.query_params.as_deref()
    }

    pub fn body(&self) -> Option<&ProxyRequestBody> {
        self.body.as_ref()
    }

    pub fn method(&self) -> &ProxyMethod {
        &self.method
    }
}

/// Represents an HTTP request body
pub struct ProxyRequestBody {
    content_type: Vec<u8>,
    content: Vec<u8>,
}

impl ProxyRequestBody {
    pub fn new(content_type: Vec<u8>, content: Vec<u8>) -> Self {
        Self {
            content_type,
            content,
        }
    }

    pub fn content_type(&self) -> &[u8] {
        &self.content_type
    }

    pub fn content(&self) -> &[u8] {
        &self.content
    }
}

/// Represents an HTTP request method
pub enum ProxyMethod {
    Get,
    Post,
    Put,
    Delete,
    Connect,
    Head,
    Options,
    Patch,
    Trace,
    Custom(Vec<u8>),
}

pub type HeaderName = Vec<u8>;

pub type HeaderValue = Vec<u8>;

pub type HeaderList = Vec<(HeaderName, HeaderValue)>;

/// Builder for an HTTP request, represented by `ProxyRequest`
#[derive(Default)]
pub struct ProxyRequestBuilder {
    headers: Option<HeaderList>,
    uri: Option<String>,
    path: Option<String>,
    query_params: Option<String>,
    body: Option<ProxyRequestBody>,
    method: Option<ProxyMethod>,
}

impl ProxyRequestBuilder {
    pub fn with_headers(mut self, headers: HeaderList) -> Self {
        self.headers = Some(headers);
        self
    }

    pub fn with_uri(mut self, uri: String) -> Self {
        self.uri = Some(uri);
        self
    }

    pub fn with_path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    pub fn with_query_params(mut self, query_params: String) -> Self {
        self.query_params = Some(query_params);
        self
    }

    pub fn with_body(mut self, content_type: Vec<u8>, content: Vec<u8>) -> Self {
        self.body = Some(ProxyRequestBody {
            content_type,
            content,
        });
        self
    }

    pub fn with_method(mut self, method: ProxyMethod) -> Self {
        self.method = Some(method);
        self
    }

    pub fn build(self) -> Result<ProxyRequest, ProxyError> {
        let headers = self.headers.unwrap_or_default();
        let uri = self.uri.ok_or_else(|| {
            ProxyError::InvalidArgumentError(InvalidArgumentError::new(
                "uri".to_string(),
                "No request `uri` provided".to_string(),
            ))
        })?;
        let path = self.path.ok_or_else(|| {
            ProxyError::InvalidArgumentError(InvalidArgumentError::new(
                "path".to_string(),
                "No request `path` provided".to_string(),
            ))
        })?;
        let query_params = self.query_params;
        let body = self.body;
        let method = self.method.ok_or_else(|| {
            ProxyError::InvalidArgumentError(InvalidArgumentError::new(
                "method".to_string(),
                "No request `method` provided".to_string(),
            ))
        })?;

        Ok(ProxyRequest {
            headers,
            uri,
            path,
            query_params,
            body,
            method,
        })
    }
}
