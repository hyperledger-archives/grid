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

pub mod error;
mod processor;
mod registry;
mod sender;

pub use processor::JoinHandles;
pub use processor::ServiceProcessor;
pub use processor::ShutdownHandle;

use crate::service::error::{
    ServiceConnectionError, ServiceDestroyError, ServiceDisconnectionError, ServiceError,
    ServiceSendError, ServiceStartError, ServiceStopError,
};

/// The ServiceMessageContext is a struct that provides information about the incoming message
#[derive(Debug)]
pub struct ServiceMessageContext {
    pub sender: String,
    pub circuit: String,
    pub correlation_id: String,
}

/// The ServiceNetworkRegistry trait provides functions to register and unregister the service on
/// the network.  It does not expose the circuit membership information directly.
pub trait ServiceNetworkRegistry {
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

    /// Send the message bytes to the given recipient (another service)
    /// and await the reply.  This function blocks until the reply is
    /// returned.
    fn send_and_await(&self, recipient: &str, message: &[u8]) -> Result<Vec<u8>, ServiceSendError>;

    /// Send the message bytes back to the origin specified in the given
    /// message context.
    fn reply(
        &self,
        message_origin: &ServiceMessageContext,
        message: &[u8],
    ) -> Result<(), ServiceSendError>;
}

pub trait Service: Send {
    /// This service's id
    fn service_id(&self) -> &str;

    /// This service's type
    fn service_type(&self) -> &str;

    /// Starts the service
    fn start(
        &mut self,
        service_registry: &dyn ServiceNetworkRegistry,
    ) -> Result<(), ServiceStartError>;

    /// Stops Starts the service
    fn stop(
        &mut self,
        service_registry: &dyn ServiceNetworkRegistry,
    ) -> Result<(), ServiceStopError>;

    /// Clean-up any resources before the service is removed.
    /// Consumes the service (which, given the use of dyn traits,
    /// this must take a boxed Service instance).
    fn destroy(self: Box<Self>) -> Result<(), ServiceDestroyError>;

    fn handle_message(
        &self,
        message_bytes: &[u8],
        message_context: &ServiceMessageContext,
    ) -> Result<(), ServiceError>;
}
