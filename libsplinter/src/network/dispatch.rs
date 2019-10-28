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

//! Methods for Dispatching and Handling Messages.
//!
use std::any::Any;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::channel::{Receiver, RecvTimeoutError, SendError, Sender};
use crate::network::sender::SendRequest;

// Recv timeout in secs
const TIMEOUT_SEC: u64 = 2;

/// The Message Context
///
/// The message context provides information about an incoming message beyond its parsed bytes.  It
/// includes the source peer id, the message type, the original bytes, and potentially other,
/// future items.
#[derive(Clone, Debug)]
pub struct MessageContext<MT: Hash + Eq + Debug + Clone> {
    source_peer_id: String,
    message_type: MT,
    message_bytes: Vec<u8>,
}

impl<MT: Hash + Eq + Debug + Clone> MessageContext<MT> {
    /// The Source Peer ID.
    ///
    /// This is the peer id of the original sender of the message
    pub fn source_peer_id(&self) -> &str {
        &self.source_peer_id
    }

    /// The Message Type.
    ///
    /// This is the message type that determined which handler to execute on receipt of this
    /// message.
    pub fn message_type(&self) -> &MT {
        &self.message_type
    }

    /// The raw message bytes.
    pub fn message_bytes(&self) -> &[u8] {
        &self.message_bytes
    }
}

/// A Handler for a network message.
pub trait Handler<MT, T>: Send
where
    MT: Hash + Eq + Debug + Clone,
    T: FromMessageBytes,
{
    /// Handles a given message
    ///
    /// # Errors
    ///
    /// Any issues that occur during processing of the message will result in a DispatchError.
    fn handle(
        &self,
        message: T,
        message_context: &MessageContext<MT>,
        network_sender: &dyn Sender<SendRequest>,
    ) -> Result<(), DispatchError>;
}

impl<MT, T, F> Handler<MT, T> for F
where
    MT: Hash + Eq + Debug + Clone,
    T: FromMessageBytes,
    F: Fn(T, &MessageContext<MT>, &dyn Sender<SendRequest>) -> Result<(), DispatchError> + Send,
{
    fn handle(
        &self,
        message: T,
        message_context: &MessageContext<MT>,
        network_sender: &dyn Sender<SendRequest>,
    ) -> Result<(), DispatchError> {
        (*self)(message, message_context, network_sender)
    }
}

/// Converts bytes into a concrete message instance
pub trait FromMessageBytes: Any + Sized {
    /// Converts the given bytes into the target type
    ///
    /// # Errors
    ///
    /// Any issues that occur during deserialization will result in a DispatchError.
    fn from_message_bytes(message_bytes: &[u8]) -> Result<Self, DispatchError>;
}

/// A container for the raw bytes of a message.
///
/// This is useful for handlers that don't deserialize the bytes via this process.  For example, a
/// handler that forwards the messages may utilize this as a message type.
#[derive(Debug, Clone)]
pub struct RawBytes {
    bytes: Vec<u8>,
}

impl RawBytes {
    /// Unwraps the value.
    pub fn into_inner(self) -> Vec<u8> {
        self.bytes
    }

    /// Returns a reference to the bytes
    ///
    /// Note, this same value may be returned by using `as_ref()`:
    ///
    ///     # use splinter::network::dispatch::RawBytes;
    ///     let raw_bytes = RawBytes::from("Value".as_bytes());
    ///     assert_eq!(raw_bytes.bytes(), raw_bytes.as_ref());
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl From<&[u8]> for RawBytes {
    fn from(source: &[u8]) -> Self {
        RawBytes {
            bytes: source.to_vec(),
        }
    }
}

impl AsRef<[u8]> for RawBytes {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl FromMessageBytes for RawBytes {
    fn from_message_bytes(message_bytes: &[u8]) -> Result<Self, DispatchError> {
        Ok(RawBytes::from(message_bytes))
    }
}

/// Dispatch Errors
///
/// These errors may occur when handling a dispatched message.
#[derive(Debug, PartialEq)]
pub enum DispatchError {
    /// An error occurred during message deserialization.
    DeserializationError(String),
    /// An error occurred during message serialization.
    SerializationError(String),
    /// An message was dispatched with an unknown type.
    UnknownMessageType(String),
    /// An error occurred while a handler was trying to send a message.
    NetworkSendError(SendError),
    /// An error occurred while a handler was executing.
    HandleError(String),
}

impl std::error::Error for DispatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DispatchError::NetworkSendError(err) => Some(err),
            _ => None,
        }
    }
}

impl std::fmt::Display for DispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DispatchError::DeserializationError(msg) => {
                write!(f, "unable to deserialize message: {}", msg)
            }
            DispatchError::SerializationError(msg) => {
                write!(f, "unable to serialize message: {}", msg)
            }
            DispatchError::UnknownMessageType(msg) => write!(f, "unknown message type: {}", msg),
            DispatchError::NetworkSendError(e) => write!(f, "unable to send message: {}", e),
            DispatchError::HandleError(msg) => write!(f, "unable to handle message: {}", msg),
        }
    }
}

impl From<SendError> for DispatchError {
    fn from(e: SendError) -> Self {
        DispatchError::NetworkSendError(e)
    }
}

/// Dispatches messages to handlers.
///
/// The dispatcher routes messages of a specific message type to one of a set of handlers that have
/// been supplied via the `set_handler` function.  It owns a `Sender` for sending messages on a
/// network, which is provided to the handlers. The handlers may use the sender for replying to or
/// broadcasting messages, as needed.
///
/// These messages are run in the same thread as the dispatch function is called. Any asynchronous
/// activity done by a handler must be managed by the handler.  These asynchronous operations must
/// return success for the handler immediately, as the expectation is that the dispatcher should
/// not block the current thread.
///
/// Message Types (MT) merely need to implement Hash, Eq and Debug (for unknown message type
/// results). Beyond that, there are no other requirements.
pub struct Dispatcher<MT: Any + Hash + Eq + Debug + Clone> {
    handlers: HashMap<MT, HandlerWrapper<MT>>,
    network_sender: Box<dyn Sender<SendRequest>>,
}

impl<MT: Any + Hash + Eq + Debug + Clone> Dispatcher<MT> {
    /// Creates a Dispatcher
    ///
    /// Creates a dispatcher with a given `Sender` to supply to handlers when they are executed.
    pub fn new(network_sender: Box<dyn Sender<SendRequest>>) -> Self {
        Dispatcher {
            handlers: HashMap::new(),
            network_sender,
        }
    }

    /// Set a handler for a given Message Type.
    ///
    /// This sets a handler for a given message type.  Only one handler may exist per message type.
    /// If a user wishes to run a series handlers, they must supply a single handler that composes
    /// the series.
    pub fn set_handler<T>(&mut self, message_type: MT, handler: Box<dyn Handler<MT, T>>)
    where
        T: FromMessageBytes,
    {
        self.handlers.insert(
            message_type,
            HandlerWrapper {
                inner: Box::new(move |message_bytes, message_context, network_sender| {
                    let message = FromMessageBytes::from_message_bytes(message_bytes)?;
                    handler.handle(message, message_context, network_sender)
                }),
            },
        );
    }

    /// Dispatch a message by type.
    ///
    /// This dispatches a message (in raw byte form) as a given message type.  The message will be
    /// handled by a handler that has been set previously via `set_handler`, if one exists.
    ///
    /// Errors
    ///
    /// A DispatchError is returned if either there is no handler for the given message type, or an
    /// error occurs while handling the messages (e.g. the message cannot be deserialized).
    pub fn dispatch(
        &self,
        source_peer_id: &str,
        message_type: &MT,
        message_bytes: Vec<u8>,
    ) -> Result<(), DispatchError> {
        let message_context = MessageContext {
            message_type: message_type.clone(),
            message_bytes,
            source_peer_id: source_peer_id.into(),
        };
        self.handlers
            .get(message_type)
            .ok_or_else(|| {
                DispatchError::UnknownMessageType(format!("No handler for type {:?}", message_type))
            })
            .and_then(|handler| {
                handler.handle(
                    &message_context.message_bytes,
                    &message_context,
                    self.network_sender.borrow(),
                )
            })
    }
}

/// A function that handles inbound message bytes.
type InnerHandler<MT> = Box<
    dyn Fn(&[u8], &MessageContext<MT>, &dyn Sender<SendRequest>) -> Result<(), DispatchError>
        + Send,
>;

/// The HandlerWrapper provides a typeless wrapper for typed Handler instances.
struct HandlerWrapper<MT: Hash + Eq + Debug + Clone> {
    inner: InnerHandler<MT>,
}

impl<MT: Hash + Eq + Debug + Clone> HandlerWrapper<MT> {
    fn handle(
        &self,
        message_bytes: &[u8],
        message_context: &MessageContext<MT>,
        network_sender: &dyn Sender<SendRequest>,
    ) -> Result<(), DispatchError> {
        (*self.inner)(message_bytes, message_context, network_sender)
    }
}

/// A message to be dispatched.
///
/// This struct contains information about a message that will be passed to a `Dispatcher` instance
/// via a `Sender<DispatchMessage>`.
#[derive(Clone)]
pub struct DispatchMessage<MT: Any + Hash + Eq + Debug + Clone> {
    message_type: MT,
    message_bytes: Vec<u8>,
    source_peer_id: String,
}

impl<MT: Any + Hash + Eq + Debug + Clone> DispatchMessage<MT> {
    /// Constructs a new DispatchMessage
    pub fn new(message_type: MT, message_bytes: Vec<u8>, source_peer_id: String) -> Self {
        DispatchMessage {
            message_type,
            message_bytes,
            source_peer_id,
        }
    }

    pub fn message_type(&self) -> &MT {
        &self.message_type
    }

    pub fn message_bytes(&self) -> &[u8] {
        &self.message_bytes
    }

    pub fn source_peer_id(&self) -> &str {
        &self.source_peer_id
    }
}

/// Errors that may occur during the operation of the Dispatch Loop.
#[derive(Debug)]
pub struct DispatchLoopError(String);

/// The Dispatch Loop
///
/// The dispatch loop processes messages that are pulled from a `Receiver<DispatchMessage>` and
/// passes them to a Dispatcher.  The dispatch loop only processes messages from a specific message
/// type.
pub struct DispatchLoop<MT: Any + Hash + Eq + Debug + Clone> {
    receiver: Box<dyn Receiver<DispatchMessage<MT>>>,
    dispatcher: Dispatcher<MT>,
    running: Arc<AtomicBool>,
}

impl<MT: Any + Hash + Eq + Debug + Clone> DispatchLoop<MT> {
    /// Constructs a new DispatchLoop.
    ///
    /// This constructs a new dispatch loop with a concrete Receiver implementation and a
    /// dispatcher instance.
    pub fn new(
        receiver: Box<dyn Receiver<DispatchMessage<MT>>>,
        dispatcher: Dispatcher<MT>,
        running: Arc<AtomicBool>,
    ) -> Self {
        DispatchLoop {
            receiver,
            dispatcher,
            running,
        }
    }

    /// Runs the loop.
    ///
    /// Errors
    ///
    /// An error will be returned if the receiver no longer can return messages. This is
    /// effectively an exit signal for the loop.
    pub fn run(&self) -> Result<(), DispatchLoopError> {
        let timeout = Duration::from_secs(TIMEOUT_SEC);
        while self.running.load(Ordering::SeqCst) {
            let dispatch_msg = match self.receiver.recv_timeout(timeout) {
                Ok(dispatch_msg) => dispatch_msg,
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => {
                    error!("Received Disconnected Error from receiver");
                    return Err(DispatchLoopError(String::from(
                        "Received Disconnected Error from receiver",
                    )));
                }
            };

            match self.dispatcher.dispatch(
                &dispatch_msg.source_peer_id,
                &dispatch_msg.message_type,
                dispatch_msg.message_bytes,
            ) {
                Ok(_) => (),
                Err(err) => warn!("Unable to dispatch message: {:?}", err),
            }
        }

        // finish handling any incoming messages
        while let Ok(dispatch_msg) = self.receiver.try_recv() {
            match self.dispatcher.dispatch(
                &dispatch_msg.source_peer_id,
                &dispatch_msg.message_type,
                dispatch_msg.message_bytes,
            ) {
                Ok(_) => (),
                Err(err) => warn!("Unable to dispatch message: {:?}", err),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    use protobuf::Message;

    use crate::channel::mock::MockSender;
    use crate::network::sender::SendRequest;
    use crate::protos::network::{NetworkEcho, NetworkMessageType};

    /// Verify that messages can be dispatched to handlers implemented as closures.
    ///
    /// This test does the following:
    ///
    /// * Create a Dispatcher
    /// * Add a handler implemented as a closure over an atomic Boolean
    /// * Dispatch a message of an unknown type and verify the error result
    /// * Dispatch a message of the expected type and verify that it was called
    #[test]
    fn dispatch_to_closure() {
        let flag = Arc::new(AtomicBool::new(false));

        let mut dispatcher = Dispatcher::new(Box::new(MockSender::default()));
        let handler_flag = flag.clone();
        dispatcher.set_handler(
            NetworkMessageType::NETWORK_ECHO,
            Box::new(
                move |_: NetworkEcho,
                      _: &MessageContext<NetworkMessageType>,
                      _: &dyn Sender<SendRequest>| {
                    handler_flag.store(true, Ordering::SeqCst);
                    Ok(())
                },
            ),
        );

        assert_eq!(
            Err(DispatchError::UnknownMessageType(format!(
                "No handler for type {:?}",
                NetworkMessageType::CIRCUIT
            ))),
            dispatcher.dispatch("TestPeer", &NetworkMessageType::CIRCUIT, Vec::new())
        );
        assert_eq!(false, flag.load(Ordering::SeqCst));

        assert_eq!(
            Ok(()),
            dispatcher.dispatch("TestPeer", &NetworkMessageType::NETWORK_ECHO, Vec::new())
        );
        assert_eq!(true, flag.load(Ordering::SeqCst));
    }

    /// Verify that messages can be dispatched to handlers via the trait.
    ///
    /// This test does the following:
    ///
    /// * Create a Dispatcher
    /// * Add a handler implemented as a struct with the Handler trait
    /// * Dispatch a message of the expected type and verify that it was called
    #[test]
    fn dispatch_to_handler() {
        let mut dispatcher = Dispatcher::new(Box::new(MockSender::default()));

        let handler = NetworkEchoHandler::default();
        let echos = handler.echos.clone();

        dispatcher.set_handler(NetworkMessageType::NETWORK_ECHO, Box::new(handler));

        let mut outgoing_message = NetworkEcho::new();
        outgoing_message.set_payload(b"test_dispatcher".to_vec());
        let outgoing_message_bytes = outgoing_message.write_to_bytes().unwrap();

        assert_eq!(
            Ok(()),
            dispatcher.dispatch(
                "TestPeer",
                &NetworkMessageType::NETWORK_ECHO,
                outgoing_message_bytes
            )
        );

        assert_eq!(
            vec!["test_dispatcher".to_string()],
            echos.lock().unwrap().clone()
        );
    }

    /// Verify that messages can be dispatched to handlers implemented as named function.
    ///
    /// This test does the following:
    ///
    /// * Create a sent message container for replies
    /// * Create a Dispatcher with that sent container
    /// * Add a handler implemented defined as a static, named function
    /// * Dispatch a message of the expected type and verify that it was called by checking that it
    ///   submitted the reply message
    #[test]
    fn dispatch_to_fn() {
        let network_sender: MockSender<SendRequest> = MockSender::default();
        let mut dispatcher = Dispatcher::new(Box::new(network_sender.clone()));

        dispatcher.set_handler(NetworkMessageType::NETWORK_ECHO, Box::new(handle_echo));

        assert_eq!(
            Ok(()),
            dispatcher.dispatch("TestPeer", &NetworkMessageType::NETWORK_ECHO, Vec::new())
        );

        assert_eq!(
            &vec![SendRequest::new("TestPeer".into(), vec![])],
            &network_sender.sent()
        );
    }

    /// Verify that a dispatcher can be moved to a thread.
    ///
    /// This test does the following:
    ///
    /// * Create a Dispatcher in the main thread
    /// * Add a handler implemented as a struct with the Handler trait
    /// * Spawn a thread and move the dispatcher to this thread
    /// * Dispatch a message of the expected type in the spawned thread
    /// * Join the thread and verify the dispatched message was handled
    #[test]
    fn move_dispatcher_to_thread() {
        let mut dispatcher = Dispatcher::new(Box::new(MockSender::default()));

        let handler = NetworkEchoHandler::default();
        let echos = handler.echos.clone();
        dispatcher.set_handler(NetworkMessageType::NETWORK_ECHO, Box::new(handler));

        std::thread::spawn(move || {
            let mut outgoing_message = NetworkEcho::new();
            outgoing_message.set_payload(b"thread_echo".to_vec());
            let outgoing_message_bytes = outgoing_message.write_to_bytes().unwrap();

            assert_eq!(
                Ok(()),
                dispatcher.dispatch(
                    "TestPeer",
                    &NetworkMessageType::NETWORK_ECHO,
                    outgoing_message_bytes
                )
            );
        })
        .join()
        .unwrap();

        assert_eq!(
            vec!["thread_echo".to_string()],
            echos.lock().unwrap().clone()
        );
    }

    #[derive(Default)]
    struct NetworkEchoHandler {
        echos: Arc<Mutex<Vec<String>>>,
    }

    impl Handler<NetworkMessageType, NetworkEcho> for NetworkEchoHandler {
        fn handle(
            &self,
            message: NetworkEcho,
            _message_context: &MessageContext<NetworkMessageType>,
            _: &dyn Sender<SendRequest>,
        ) -> Result<(), DispatchError> {
            let echo_string = String::from_utf8(message.get_payload().to_vec()).unwrap();
            self.echos.lock().unwrap().push(echo_string);
            Ok(())
        }
    }

    /// This test handler
    fn handle_echo(
        message: RawBytes,
        message_context: &MessageContext<NetworkMessageType>,
        network_sender: &dyn Sender<SendRequest>,
    ) -> Result<(), DispatchError> {
        let expected_message: Vec<u8> = vec![];
        assert_eq!(expected_message, message.bytes());

        network_sender.send(SendRequest::new(
            message_context.source_peer_id().to_string(),
            vec![],
        ))?;

        Ok(())
    }
}
