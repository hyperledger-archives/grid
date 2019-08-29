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

mod error;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::service::{
    JoinHandles, ServiceFactory, ServiceProcessor, ServiceProcessorError, ShutdownHandle,
};
use crate::transport::inproc::InprocTransport;
use crate::transport::Transport;

pub use self::error::{InitializeServiceError, ListServicesError, ShutdownServiceError};

const PROCESSOR_INCOMING_CAPACITY: usize = 8;
const PROCESSOR_OUTGOING_CAPACITY: usize = 8;
const PROCESSOR_CHANNEL_CAPACITY: usize = 8;

/// Identifies a unique service instance from the perspective of the orchestrator
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ServiceDefinition {
    pub circuit: String,
    pub service_id: String,
    pub service_type: String,
}

/// Handle used to shutdown the service processor that is running a service
struct ServiceProcessorHandle {
    running: Arc<AtomicBool>,
    shutdown_handle: ShutdownHandle,
    join_handles: JoinHandles<Result<(), ServiceProcessorError>>,
}

/// The `ServiceOrchestrator` manages initialization and shutdown of services.
pub struct ServiceOrchestrator {
    /// A (ServiceDefinition, ServiceProcessorHandle) map
    services: Arc<Mutex<HashMap<ServiceDefinition, ServiceProcessorHandle>>>,
    /// Factories used to create new services.
    service_factories: Vec<Box<dyn ServiceFactory>>,
    /// The endpoint of the splinter node that will be used to create connections for new services
    service_endpoint: String,
    /// The transport that will be used to create connections for new services
    transport: Arc<Mutex<InprocTransport>>,
}

impl ServiceOrchestrator {
    /// Create a new `ServiceOrchestrator` that will provide connections to new services that it
    /// creates using the given service endpoint and transport.
    pub fn new(
        service_factories: Vec<Box<dyn ServiceFactory>>,
        service_endpoint: String,
        transport: InprocTransport,
    ) -> Self {
        Self {
            services: Arc::new(Mutex::new(HashMap::new())),
            service_factories,
            service_endpoint,
            transport: Arc::new(Mutex::new(transport)),
        }
    }

    /// Initialize (create and start) a service according to the specified definition. The
    /// arguments provided must match those required to create the service.
    pub fn initialize_service(
        &self,
        service_definition: ServiceDefinition,
        args: HashMap<String, String>,
    ) -> Result<(), InitializeServiceError> {
        let factory = self
            .service_factories
            .iter()
            .find(|factory| {
                factory
                    .available_service_types()
                    .contains(&service_definition.service_type)
            })
            .ok_or(InitializeServiceError::UnknownType)?;

        let service = factory.create(
            service_definition.service_id.clone(),
            service_definition.service_type.as_str(),
            args,
        )?;

        let connection = self
            .transport
            .lock()
            .map_err(|_| InitializeServiceError::LockPoisoned)?
            .connect(&self.service_endpoint)
            .map_err(|err| InitializeServiceError::InitializationFailed(Box::new(err)))?;
        let running = Arc::new(AtomicBool::new(true));
        let mut processor = ServiceProcessor::new(
            connection,
            service_definition.circuit.clone(),
            PROCESSOR_INCOMING_CAPACITY,
            PROCESSOR_OUTGOING_CAPACITY,
            PROCESSOR_CHANNEL_CAPACITY,
            running.clone(),
        )
        .map_err(|err| InitializeServiceError::InitializationFailed(Box::new(err)))?;

        processor
            .add_service(service)
            .map_err(|err| InitializeServiceError::InitializationFailed(Box::new(err)))?;

        let (shutdown_handle, join_handles) = processor
            .start()
            .map_err(|err| InitializeServiceError::InitializationFailed(Box::new(err)))?;

        self.services
            .lock()
            .map_err(|_| InitializeServiceError::LockPoisoned)?
            .insert(
                service_definition,
                ServiceProcessorHandle {
                    running,
                    shutdown_handle,
                    join_handles,
                },
            );

        Ok(())
    }

    /// Shut down (stop and destroy) the specified service.
    pub fn shutdown_service(
        &self,
        service_definition: &ServiceDefinition,
    ) -> Result<(), ShutdownServiceError> {
        let processor_handle = self
            .services
            .lock()
            .map_err(|_| ShutdownServiceError::LockPoisoned)?
            .remove(service_definition)
            .ok_or(ShutdownServiceError::UnknownService)?;

        processor_handle.running.store(false, Ordering::SeqCst);
        processor_handle
            .shutdown_handle
            .shutdown()
            .map_err(|err| ShutdownServiceError::ShutdownFailed(Box::new(err)))?;
        if let Err(err) = processor_handle.join_handles.join_all() {
            error!("service processor thread(s) failed: {:?}", err)
        }
        Ok(())
    }

    /// List services managed by this `ServiceOrchestrator`; filters may be provided to only show
    /// services on specified circuit(s) and of given service type(s).
    pub fn list_services(
        &self,
        circuits: Vec<String>,
        service_types: Vec<String>,
    ) -> Result<Vec<ServiceDefinition>, ListServicesError> {
        Ok(self
            .services
            .lock()
            .map_err(|_| ListServicesError::LockPoisoned)?
            .iter()
            .filter_map(|(service, _)| {
                if (circuits.is_empty() || circuits.contains(&service.circuit))
                    && (service_types.is_empty() || service_types.contains(&service.service_type))
                {
                    Some(service)
                } else {
                    None
                }
            })
            .cloned()
            .collect())
    }
}
