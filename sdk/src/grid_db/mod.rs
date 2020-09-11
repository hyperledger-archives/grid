// Copyright 2018-2020 Cargill Incorporated
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

//! The grid_db submodule provides support for managing organizations,
//! agents, commits, schemas, locations, products, and Track and Trace
//! data.

pub mod agents;
pub mod commits;
pub mod locations;
pub mod organizations;

pub mod migrations;

#[cfg(feature = "diesel")]
pub use agents::store::diesel::DieselAgentStore;
pub use agents::store::AgentStore;

#[cfg(feature = "diesel")]
pub use commits::store::diesel::DieselCommitStore;
pub use commits::store::memory::MemoryCommitStore;
pub use commits::store::CommitStore;

#[cfg(feature = "diesel")]
pub use organizations::store::diesel::DieselOrganizationStore;
pub use organizations::store::memory::MemoryOrganizationStore;
pub use organizations::store::OrganizationStore;

#[cfg(feature = "diesel")]
pub use locations::store::diesel::DieselLocationStore;
pub use locations::store::LocationStore;
