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

/// Model for a error response to an REST request
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    code: String,
    message: String,
}

impl ErrorResponse {
    pub fn internal_error() -> ErrorResponse {
        ErrorResponse {
            code: "500".to_string(),
            message: "The server encountered an error".to_string(),
        }
    }

    pub fn bad_request(message: &str) -> ErrorResponse {
        ErrorResponse {
            code: "400".to_string(),
            message: message.to_string(),
        }
    }

    pub fn not_found(message: &str) -> ErrorResponse {
        ErrorResponse {
            code: "404".to_string(),
            message: message.to_string(),
        }
    }

    pub fn unauthorized(message: &str) -> ErrorResponse {
        ErrorResponse {
            code: "401".to_string(),
            message: message.to_string(),
        }
    }
}
