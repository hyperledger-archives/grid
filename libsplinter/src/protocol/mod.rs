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

//! Protocol versions for various endpoints provided by splinter.

pub const ADMIN_PROTOCOL_VERSION: u32 = 1;

pub(crate) const ADMIN_APPLICATION_REGISTRATION_PROTOCOL_MIN: u32 = 1;

pub(crate) const ADMIN_SUBMIT_PROTOCOL_MIN: u32 = 1;

#[cfg(feature = "proposal-read")]
pub(crate) const ADMIN_FETCH_PROPOSALS_PROTOCOL_MIN: u32 = 1;

#[cfg(feature = "proposal-read")]
pub(crate) const ADMIN_LIST_PROPOSALS_PROTOCOL_MIN: u32 = 1;

pub const SCABBARD_PROTOCOL_VERSION: u32 = 1;

pub(crate) const SCABBARD_SUBSCRIBE_PROTOCOL_MIN: u32 = 1;
pub(crate) const SCABBARD_ADD_BATCHES_PROTOCOL_MIN: u32 = 1;
pub(crate) const SCABBARD_BATCH_STATUSES_PROTOCOL_MIN: u32 = 1;

#[cfg(feature = "scabbard-get-state")]
pub(crate) const SCABBARD_GET_STATE_PROTOCOL_MIN: u32 = 1;
#[cfg(feature = "scabbard-get-state")]
pub(crate) const SCABBARD_LIST_STATE_PROTOCOL_MIN: u32 = 1;
