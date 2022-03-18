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

/// Represents a basic HTTP response, to be used when responding to a proxied request
#[derive(Debug)]
pub struct ProxyResponse {
    status_code: u16,
    body: ProxyResponseBody,
}

#[derive(Debug)]
/// Represents an HTTP request body
pub struct ProxyResponseBody {
    content: Vec<u8>,
}

impl ProxyResponseBody {
    pub fn new(content: Vec<u8>) -> Self {
        ProxyResponseBody { content }
    }

    pub fn content(&self) -> &[u8] {
        &self.content
    }
}

impl ProxyResponse {
    pub fn new(status_code: u16, content: Vec<u8>) -> Self {
        Self {
            status_code,
            body: ProxyResponseBody::new(content),
        }
    }

    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub fn body(&self) -> &ProxyResponseBody {
        &self.body
    }
}
