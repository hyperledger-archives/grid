/*
 * Copyright 2021 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use std::collections::{hash_map::Entry, HashMap};
use std::sync::{Arc, Mutex};

use grid_sdk::error::InternalError;

use crate::event::{EventHandler, EventProcessor};

use super::{ScabbardEventConnection, ScabbardEventConnectionFactory};

/// A collection of event processors.
#[derive(Clone)]
pub struct EventProcessors {
    inner: Arc<Mutex<Inner>>,
}

impl EventProcessors {
    pub fn new(event_connection_factory: ScabbardEventConnectionFactory) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::new(event_connection_factory))),
        }
    }

    /// Add the event processor for a given circuit_id::service_id, if it is not in the collection.
    ///
    /// The provided factory function will create the handlers if there is an event processor miss.
    ///
    /// This method is idempotent.
    pub fn add_once<F>(
        &self,
        circuit_id: &str,
        service_id: &str,
        last_seen_id: Option<&str>,
        authorization: &str,
        handlers_factory_fn: F,
    ) -> Result<(), InternalError>
    where
        F: Fn() -> Vec<Box<dyn EventHandler>>,
    {
        let mut inner = self.inner.lock().map_err(|_| {
            InternalError::with_message("EventProcessors inner mutex was poisoned".into())
        })?;
        inner.add_once(
            circuit_id,
            service_id,
            last_seen_id,
            authorization,
            handlers_factory_fn,
        )
    }
}

struct Inner {
    event_connection_factory: ScabbardEventConnectionFactory,
    processors: HashMap<String, EventProcessor<ScabbardEventConnection>>,
}

impl Inner {
    pub fn new(event_connection_factory: ScabbardEventConnectionFactory) -> Self {
        Self {
            event_connection_factory,
            processors: HashMap::new(),
        }
    }

    pub fn add_once<F>(
        &mut self,
        circuit_id: &str,
        service_id: &str,
        last_seen_id: Option<&str>,
        authorization: &str,
        factory_fn: F,
    ) -> Result<(), InternalError>
    where
        F: Fn() -> Vec<Box<dyn EventHandler>>,
    {
        let key = format!("{}::{}", circuit_id, service_id);
        if let Entry::Vacant(entry) = self.processors.entry(key) {
            let event_connection = self
                .event_connection_factory
                .create_connection(circuit_id, service_id, authorization)
                .map_err(|err| InternalError::from_source(Box::new(err)))?;

            let evt_processor = EventProcessor::start(event_connection, last_seen_id, factory_fn())
                .map_err(|err| InternalError::from_source(Box::new(err)))?;

            entry.insert(evt_processor);
        }

        Ok(())
    }
}
