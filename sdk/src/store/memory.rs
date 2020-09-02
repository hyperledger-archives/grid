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

use crate::grid_db::{CommitStore, MemoryCommitStore};

use super::StoreFactory;

/// A `StoryFactory` backed by memory.
#[derive(Default)]
pub struct MemoryStoreFactory {
    grid_commit_store: MemoryCommitStore,
}

impl MemoryStoreFactory {
    pub fn new() -> Self {
        let grid_commit_store = MemoryCommitStore::new();

        Self { grid_commit_store }
    }
}

impl StoreFactory for MemoryStoreFactory {
    fn get_grid_commit_store(&self) -> Box<dyn CommitStore> {
        Box::new(self.grid_commit_store.clone())
    }
}
