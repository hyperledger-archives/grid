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

//!  Splinter services are a fundamental component of a Splinter network.  They provide the
//!  application-level business logic, abstracted above the underlying network and circuit layers.
//!  The Service API provides a framework for defining services from a Splinter perspective.
//!
//!  Splinter services are effectively message handlers.  They receive the bytes of a message and a
//!  context for the message (including the originating sender).  It is up to the service to parse
//!  the message bytes into the format desired.
//!
//!  A service is identified by an ID, which must be unique within a Splinter circuit.  It also
//!  provides a service type, which indicates what kinds of capabilities the service provides.
//!
//!  There may be more than one service of a given service type on a circuit, though each instance
//!  must continue to have a unique ID.
//!
//!  Services are started and stopped explicitly.  At these times, the service must either register
//!  (at start) or unregister (at stop) itself.  In splinter terms, these two operations connect
//!  and disconnect the service from a circuit, but the Service API keeps the service circuit-
//!  agnostic.
//!
//!  A stand-alone service implementation may be wrapped in a ServiceProcessor, which will manage
//!  lower-level messaging and networking needs to talk to applications that implement Splinter
//!  node capabilities, such as the Splinter daemon.

pub mod error;
mod factory;
mod processor;
mod registry;
#[cfg(feature = "rest-api")]
mod rest_api;
pub mod scabbard;
mod sender;
#[cfg(feature = "service-arg-validation")]
pub mod validation;

use std::any::Any;

pub use factory::ServiceFactory;
pub use processor::JoinHandles;
pub use processor::ServiceProcessor;
pub use processor::ShutdownHandle;
pub use registry::StandardServiceNetworkRegistry;

pub use error::{
    FactoryCreateError, ServiceConnectionError, ServiceDestroyError, ServiceDisconnectionError,
    ServiceError, ServiceProcessorError, ServiceSendError, ServiceStartError, ServiceStopError,
};

/// The ServiceMessageContext is a struct that provides information about an incoming message.
#[derive(Clone, Debug)]
pub struct ServiceMessageContext {
    pub sender: String,
    pub circuit: String,
    pub correlation_id: String,
}

/// The ServiceNetworkRegistry trait provides functions to register and unregister the service on
/// the network.  It does not expose the circuit membership information directly.
pub trait ServiceNetworkRegistry: Send {
    fn connect(
        &self,
        service_id: &str,
    ) -> Result<Box<dyn ServiceNetworkSender>, ServiceConnectionError>;
    fn disconnect(&self, service_id: &str) -> Result<(), ServiceDisconnectionError>;
}

/// The ServiceNetworkSender trait allows a service to send its own messages, such as replies to
/// the original message or forwarding the message to other services on the same circuit.  It does
/// not expose the circuit information directly.
pub trait ServiceNetworkSender: Send {
    /// Send the message bytes to the given recipient (another service)
    fn send(&self, recipient: &str, message: &[u8]) -> Result<(), ServiceSendError>;

    /// Send the message bytes to the given recipient (another service) and await the reply.  This
    /// function blocks until the reply is returned.
    fn send_and_await(&self, recipient: &str, message: &[u8]) -> Result<Vec<u8>, ServiceSendError>;

    /// Send the message bytes back to the origin specified in the given message context.
    fn reply(
        &self,
        message_origin: &ServiceMessageContext,
        message: &[u8],
    ) -> Result<(), ServiceSendError>;

    /// Clone this instance into Boxed, dynamic trait
    fn clone_box(&self) -> Box<dyn ServiceNetworkSender>;
}

impl Clone for Box<dyn ServiceNetworkSender> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// A Service provides message handling for a given service type.
pub trait Service: Send {
    /// This service's ID
    ///
    /// This ID must be unique within the context of a circuit, but not necessarily unique within
    /// the context of a splinter node, as a whole.
    fn service_id(&self) -> &str;

    /// This service's type
    ///
    /// A service type broadly identifies the kinds of messages that this service handles or emits.
    fn service_type(&self) -> &str;

    /// Starts the service
    ///
    /// At start time, the service should register itself with the network when its ready to
    /// receive messages.
    fn start(
        &mut self,
        service_registry: &dyn ServiceNetworkRegistry,
    ) -> Result<(), ServiceStartError>;

    /// Stops Starts the service
    ///
    /// The service should unregister itself with the network.
    fn stop(
        &mut self,
        service_registry: &dyn ServiceNetworkRegistry,
    ) -> Result<(), ServiceStopError>;

    /// Clean-up any resources before the service is removed.
    /// Consumes the service (which, given the use of dyn traits,
    /// this must take a boxed Service instance).
    fn destroy(self: Box<Self>) -> Result<(), ServiceDestroyError>;

    /// Handle any incoming message intended for this service instance.
    ///
    /// Messages recevied by this service are provided in raw bytes.  The format of the service
    fn handle_message(
        &self,
        message_bytes: &[u8],
        message_context: &ServiceMessageContext,
    ) -> Result<(), ServiceError>;

    /// Cast the service as `&dyn Any`.
    ///
    /// This allows for downcasting the `Service` to a specific implementation.
    fn as_any(&self) -> &dyn Any;
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use std::collections::HashSet;
    use std::error::Error;
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    pub struct MockServiceNetworkRegistryError(pub String);

    impl Error for MockServiceNetworkRegistryError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            None
        }
    }

    impl std::fmt::Display for MockServiceNetworkRegistryError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    pub struct MockServiceNetworkRegistry {
        pub connected_ids: Arc<Mutex<HashSet<String>>>,
        network_sender: MockServiceNetworkSender,
    }

    impl MockServiceNetworkRegistry {
        pub fn new() -> Self {
            MockServiceNetworkRegistry {
                connected_ids: Arc::new(Mutex::new(HashSet::new())),
                network_sender: MockServiceNetworkSender::new(),
            }
        }

        pub fn network_sender(&self) -> &MockServiceNetworkSender {
            &self.network_sender
        }
    }

    impl ServiceNetworkRegistry for MockServiceNetworkRegistry {
        fn connect(
            &self,
            service_id: &str,
        ) -> Result<Box<dyn ServiceNetworkSender>, ServiceConnectionError> {
            if self
                .connected_ids
                .lock()
                .expect("connected_ids lock poisoned")
                .insert(service_id.into())
            {
                Ok(Box::new(self.network_sender.clone()))
            } else {
                Err(ServiceConnectionError::RejectedError(format!(
                    "service with id {} already connected",
                    service_id
                )))
            }
        }

        fn disconnect(&self, service_id: &str) -> Result<(), ServiceDisconnectionError> {
            if self
                .connected_ids
                .lock()
                .expect("connected_ids lock poisoned")
                .remove(service_id)
            {
                Ok(())
            } else {
                Err(ServiceDisconnectionError::RejectedError(format!(
                    "service with id {} not connected",
                    service_id
                )))
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct MockServiceNetworkSender {
        pub sent: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
        pub sent_and_awaited: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
        pub replied: Arc<Mutex<Vec<(ServiceMessageContext, Vec<u8>)>>>,
    }

    impl MockServiceNetworkSender {
        pub fn new() -> Self {
            MockServiceNetworkSender {
                sent: Arc::new(Mutex::new(vec![])),
                sent_and_awaited: Arc::new(Mutex::new(vec![])),
                replied: Arc::new(Mutex::new(vec![])),
            }
        }
    }

    impl ServiceNetworkSender for MockServiceNetworkSender {
        fn send(&self, recipient: &str, message: &[u8]) -> Result<(), ServiceSendError> {
            self.sent
                .lock()
                .expect("sent lock poisoned")
                .push((recipient.to_string(), message.to_vec()));
            Ok(())
        }

        fn send_and_await(
            &self,
            recipient: &str,
            message: &[u8],
        ) -> Result<Vec<u8>, ServiceSendError> {
            self.sent_and_awaited
                .lock()
                .expect("sent_and_awaited lock poisoned")
                .push((recipient.to_string(), message.to_vec()));
            Ok(vec![])
        }

        fn reply(
            &self,
            message_origin: &ServiceMessageContext,
            message: &[u8],
        ) -> Result<(), ServiceSendError> {
            self.replied
                .lock()
                .expect("replied lock poisoned")
                .push((message_origin.clone(), message.to_vec()));
            Ok(())
        }

        fn clone_box(&self) -> Box<dyn ServiceNetworkSender> {
            Box::new(self.clone())
        }
    }

    /// Verifies that the given service connects on start and disconnects on stop.
    pub fn test_connect_and_disconnect(service: &mut dyn Service) {
        let registry = MockServiceNetworkRegistry::new();
        service.start(&registry).expect("failed to start engine");
        assert!(registry
            .connected_ids
            .lock()
            .expect("connected_ids lock poisoned")
            .contains(service.service_id()));
        service.stop(&registry).expect("failed to stop engine");
        assert!(registry
            .connected_ids
            .lock()
            .expect("connected_ids lock poisoned")
            .is_empty());
    }
}
