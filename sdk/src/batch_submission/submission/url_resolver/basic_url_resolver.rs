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

use crate::scope_id::{GlobalScopeId, ServiceScopeId};

use super::UrlResolver;

/// A url resolver for the `GlobalScopeId`
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUrlResolver {
    base_url: String,
}

impl GlobalUrlResolver {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

impl UrlResolver for GlobalUrlResolver {
    type Id = GlobalScopeId;
    fn url(&self, scope_id: &GlobalScopeId) -> String {
        // Batch info isn't used when there is no service_id
        let _ = scope_id;
        self.base_url.to_string()
    }
}

/// A url resolver for the `ServiceScopeId`
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceUrlResolver {
    base_url: String,
}

impl ServiceUrlResolver {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
}

impl UrlResolver for ServiceUrlResolver {
    type Id = ServiceScopeId;
    fn url(&self, scope_id: &ServiceScopeId) -> String {
        format!(
            "{base_url}?service_id={sid}",
            base_url = self.base_url,
            sid = scope_id.service_id()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_submission_global_url_resolver() {
        let resolver = GlobalUrlResolver::new("test.com".to_string());
        let scope_id = GlobalScopeId::default();

        assert_eq!(resolver.url(&scope_id), "test.com".to_string());
    }

    #[test]
    fn test_batch_submission_service_url_resolver() {
        let resolver = ServiceUrlResolver::new("test.com".to_string());
        let scope_id = ServiceScopeId::new_from_string("12345-67890::abcd".to_string()).unwrap();

        assert_eq!(
            resolver.url(&scope_id),
            "test.com?service_id=12345-67890::abcd".to_string()
        );
    }
}
