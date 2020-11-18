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

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::error::InternalError;
use crate::grid_db::schemas::store::{Schema, SchemaStore, SchemaStoreError};

#[derive(Clone, Default)]
pub struct MemorySchemaStore {
    inner: Arc<Mutex<HashMap<String, Schema>>>,
}

impl MemorySchemaStore {
    pub fn new() -> Self {
        MemorySchemaStore {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl SchemaStore for MemorySchemaStore {
    fn add_schema(&self, schema: Schema) -> Result<(), SchemaStoreError> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| SchemaStoreError::InternalError(
                InternalError::with_message("Cannot access schemas: mutex lock poisoned".to_string()
            ))?;

        let key = if let Some(ref service_id) = schema.service_id {
            format!("{}:{}", schema.name, service_id)
        } else {
            schema.name.clone()
        };

        inner.insert(key, schema);

        Ok(())
    }

    fn fetch_schema(
        &self,
        name: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Schema>, SchemaStoreError> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| SchemaStoreError::InternalError(
                InternalError::with_message("Cannot access schemas: mutex lock poisoned".to_string()
            ))?;

        let key = if let Some(ref service_id) = service_id {
            format!("{}:{}", name, service_id)
        } else {
            name.to_string()
        };

        Ok(inner.get(&key).map(Schema::clone))
    }

    fn list_schemas(&self, service_id: Option<&str>) -> Result<Vec<Schema>, SchemaStoreError> {
        let inner = self
            .inner
            .lock()
            .map_err(|_| SchemaStoreError::InternalError(
                InternalError::with_message("Cannot access schemas: mutex lock poisoned".to_string()
            ))?;

        if let Some(service_id) = service_id {
            Ok(inner
                .values()
                .map(Schema::clone)
                .filter(|v| v.service_id.is_some() && v.service_id.as_ref().unwrap() == service_id)
                .collect())
        } else {
            Ok(inner.values().map(Schema::clone).collect())
        }
    }
}
