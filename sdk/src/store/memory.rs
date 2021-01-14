// Copyright 2018-2021 Cargill Incorporated
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

#[cfg(feature = "location")]
use crate::locations::LocationStore;
#[cfg(feature = "schema")]
use crate::schemas::SchemaStore;
#[cfg(feature = "track-and-trace")]
use crate::track_and_trace::TrackAndTraceStore;
#[cfg(feature = "pike")]
use crate::{
    agents::AgentStore, organizations::MemoryOrganizationStore, organizations::OrganizationStore,
};
use crate::{commits::CommitStore, commits::MemoryCommitStore};

use super::StoreFactory;

/// A `StoryFactory` backed by memory.
#[derive(Default)]
pub struct MemoryStoreFactory {
    grid_commit_store: MemoryCommitStore,
    #[cfg(feature = "pike")]
    grid_organization_store: MemoryOrganizationStore,
}

impl MemoryStoreFactory {
    pub fn new() -> Self {
        let grid_commit_store = MemoryCommitStore::new();
        #[cfg(feature = "pike")]
        let grid_organization_store = MemoryOrganizationStore::new();

        Self {
            grid_commit_store,
            #[cfg(feature = "pike")]
            grid_organization_store,
        }
    }
}

impl StoreFactory for MemoryStoreFactory {
    #[cfg(feature = "pike")]
    fn get_grid_agent_store(&self) -> Box<dyn AgentStore> {
        unimplemented!()
    }

    fn get_grid_commit_store(&self) -> Box<dyn CommitStore> {
        Box::new(self.grid_commit_store.clone())
    }

    #[cfg(feature = "pike")]
    fn get_grid_organization_store(&self) -> Box<dyn OrganizationStore> {
        Box::new(self.grid_organization_store.clone())
    }

    #[cfg(feature = "location")]
    fn get_grid_location_store(&self) -> Box<dyn LocationStore> {
        unimplemented!()
    }

    #[cfg(feature = "product")]
    fn get_grid_product_store(&self) -> Box<dyn crate::products::ProductStore> {
        unimplemented!()
    }

    #[cfg(feature = "schema")]
    fn get_grid_schema_store(&self) -> Box<dyn SchemaStore> {
        unimplemented!()
    }

    #[cfg(feature = "track-and-trace")]
    fn get_grid_track_and_trace_store(&self) -> Box<dyn TrackAndTraceStore> {
        unimplemented!()
    }
}
