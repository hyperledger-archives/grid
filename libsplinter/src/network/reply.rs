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

use std::any::Any;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{mpsc::channel, Arc, Mutex};

use crate::channel::{Receiver, RecvError, Sender};
use crate::network::dispatch::FromMessageBytes;

pub type MessageResult<T> = Result<T, RecvError>;

pub trait HasCorrelationId {
    fn correlation_id(&self) -> &str;
}

pub trait Envelope {
    fn payload(&self) -> &[u8];
}

type ExpectedReplies<T> = Arc<Mutex<HashMap<String, Box<dyn Sender<MessageResult<T>>>>>>;

#[derive(Clone)]
pub struct InboundRouter<T>
where
    T: HasCorrelationId + Envelope + Send + Any,
{
    expected_replies: ExpectedReplies<T>,
    default_sender: Box<dyn Sender<MessageResult<T>>>,
}

impl<T> InboundRouter<T>
where
    T: HasCorrelationId + Envelope + Send + Any,
{
    pub fn new(default_sender: Box<dyn Sender<MessageResult<T>>>) -> Self {
        InboundRouter {
            expected_replies: Default::default(),
            default_sender,
        }
    }

    pub fn route(&mut self, message_result: MessageResult<T>) -> Result<(), RouteError> {
        match message_result {
            Ok(message) => {
                let mut expected_replies = self.expected_replies.lock().expect("Lock was poisened");
                if let Some(sender) = expected_replies.remove(message.correlation_id()) {
                    sender
                        .send(Ok(message))
                        .map_err(|err| RouteError(Box::new(err)))?;
                } else {
                    self.default_sender
                        .send(Ok(message))
                        .map_err(|err| RouteError(Box::new(err)))?;
                }
            }
            Err(RecvError { error }) => {
                let mut expected_replies = self.expected_replies.lock().expect("Lock was poisened");
                for (_, sender) in expected_replies.iter_mut() {
                    sender
                        .send(Err(RecvError {
                            error: error.clone(),
                        }))
                        .map_err(|err| RouteError(Box::new(err)))?;
                }
            }
        }
        Ok(())
    }

    pub fn expect_reply(&self, correlation_id: String) -> Box<dyn Receiver<MessageResult<T>>> {
        let (expect_tx, expect_rx) = channel();
        let mut expected_replies = self.expected_replies.lock().unwrap();
        expected_replies.insert(correlation_id, Box::new(expect_tx));

        Box::new(expect_rx)
    }
}
#[derive(Debug)]
pub struct RouteError(pub Box<dyn Error + Send>);

impl Error for RouteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

impl std::fmt::Display for RouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to route message: {}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct FutureError {}

/// MessageFuture is a promise for the reply to a sent message.
pub struct MessageFuture<T>
where
    T: HasCorrelationId + Envelope + Send + Any,
{
    inner: Box<dyn Receiver<MessageResult<T>>>,
    result: Option<MessageResult<T>>,
}

impl<T> MessageFuture<T>
where
    T: HasCorrelationId + Envelope + Send + Any,
{
    pub fn new(inner: Box<dyn Receiver<MessageResult<T>>>) -> Self {
        MessageFuture {
            inner,
            result: None,
        }
    }

    pub fn get<M: FromMessageBytes + Clone>(&mut self) -> Result<M, FutureError> {
        if let Some(result) = self.result.as_ref() {
            return result
                .as_ref()
                .map_err(|_recv_err| FutureError {})
                .and_then(|env| {
                    FromMessageBytes::from_message_bytes(env.payload())
                        .map_err(|_from_err| FutureError {})
                });
        }

        let result: MessageResult<T> = self.inner.recv().map_err(|_recv_err| FutureError {})?;

        self.result = Some(result);

        // This is safe because we just wrapped it in Some
        match self.result.as_ref().unwrap() {
            Ok(env) => FromMessageBytes::from_message_bytes(env.payload())
                .map_err(|_from_err| FutureError {}),
            Err(_err) => Err(FutureError {}),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use std::sync::mpsc::channel;
    use std::thread;

    #[test]
    // test that if a message is received without a matching correlation id, the message is routed
    // to the default sender
    fn test_no_correlation_id() {
        let (default_tx, default_rx) = channel();
        let mut inbound_router: InboundRouter<TestMessage> =
            InboundRouter::new(Box::new(default_tx));

        thread::Builder::new()
            .name("test_no_correlation_id".to_string())
            .spawn(move || {
                let msg_result = Ok(TestMessage {
                    payload: b"test_payload".to_vec(),
                    correlation_id: "test".to_string(),
                });
                inbound_router.route(msg_result)
            })
            .unwrap();;

        let msg = match default_rx.recv() {
            Ok(msg_result) => msg_result.unwrap(),
            Err(err) => panic!("Received error: {}", err),
        };

        assert_eq!(b"test_payload", msg.payload());
    }

    #[test]
    // test that if a message is received with a matching correlation id, the message is routed
    // to the receiver that is blocking on the reply
    fn test_expect_reply() {
        let (default_tx, _) = channel();
        let mut inbound_router: InboundRouter<TestMessage> =
            InboundRouter::new(Box::new(default_tx));

        let receiver = inbound_router.expect_reply("test".to_string());
        thread::Builder::new()
            .name("test_expect_reply".to_string())
            .spawn(move || {
                let msg_result = Ok(TestMessage {
                    payload: b"test_payload".to_vec(),
                    correlation_id: "test".to_string(),
                });
                inbound_router.route(msg_result)
            })
            .unwrap();;

        let msg = match receiver.recv() {
            Ok(msg_result) => msg_result.unwrap(),
            Err(err) => panic!("Received error: {:?}", err),
        };

        assert_eq!(b"test_payload", msg.payload());
    }

    struct TestMessage {
        pub payload: Vec<u8>,
        pub correlation_id: String,
    }

    impl HasCorrelationId for TestMessage {
        fn correlation_id(&self) -> &str {
            &self.correlation_id
        }
    }

    impl Envelope for TestMessage {
        fn payload(&self) -> &[u8] {
            &self.payload
        }
    }
}
