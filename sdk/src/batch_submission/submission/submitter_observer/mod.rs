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

use crate::scope_id::ScopeId;

/// An interface for interpretting and recording updates from the submitter
pub trait SubmitterObserver {
    type Id: ScopeId;
    /// Notify the observer of an update. The interpretation and recording
    /// of the update is determined by the observer's implementation.
    fn notify(
        &self,
        batch_header: String,
        scope_id: Self::Id,
        status: Option<u16>,
        message: Option<String>,
    );
}
