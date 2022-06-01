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

pub mod basic_url_resolver;

use crate::scope_id::ScopeId;

/// An interface for generating the url to which a batch should be sent.
pub trait UrlResolver: std::fmt::Debug + Sync + Send {
    type Id: ScopeId;
    /// Generates an address (i.e. URL) to which the batch will be sent.
    fn url(&self, scope_id: &Self::Id) -> String;
}
