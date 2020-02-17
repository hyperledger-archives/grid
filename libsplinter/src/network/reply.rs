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

use std::collections::HashMap;
use std::error::Error;
use std::sync::{mpsc::channel, Arc, Mutex};

use crate::channel::{Receiver, RecvError, Sender};
use crate::network::dispatch::FromMessageBytes;

pub type MessageResult<MessageType> = Result<(MessageType, Vec<u8>), RecvError>;

type ExpectedReplies<MessageType> =
    Arc<Mutex<HashMap<String, Box<dyn Sender<MessageResult<MessageType>>>>>>;

#[derive(Clone)]
pub struct InboundRouter<MessageType>
where
    MessageType: std::fmt::Debug + PartialEq + Send + 'static,
{
    expected_replies: ExpectedReplies<MessageType>,
    default_sender: Box<dyn Sender<MessageResult<MessageType>>>,
}

impl<MessageType> InboundRouter<MessageType>
where
    MessageType: std::fmt::Debug + PartialEq + Send + 'static,
{
    pub fn new(default_sender: Box<dyn Sender<MessageResult<MessageType>>>) -> Self {
        InboundRouter {
            expected_replies: Default::default(),
            default_sender,
        }
    }

    pub fn route(
        &mut self,
        correlation_id: &str,
        message_result: MessageResult<MessageType>,
    ) -> Result<(), RouteError> {
        match message_result {
            Ok((message_type, message)) => {
                let mut expected_replies = self.expected_replies.lock().expect("Lock was poisoned");
                if let Some(sender) = expected_replies.remove(correlation_id) {
                    sender
                        .send(Ok((message_type, message)))
                        .map_err(|err| RouteError(Box::new(err)))?;
                } else {
                    self.default_sender
                        .send(Ok((message_type, message)))
                        .map_err(|err| RouteError(Box::new(err)))?;
                }
            }
            Err(RecvError { error }) => {
                let mut expected_replies = self.expected_replies.lock().expect("Lock was poisoned");
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

    pub fn expect_reply(&self, correlation_id: String) -> MessageFuture<MessageType> {
        let (expect_tx, expect_rx) = channel();
        let mut expected_replies = self.expected_replies.lock().unwrap();
        expected_replies.insert(correlation_id, Box::new(expect_tx));

        MessageFuture::new(Box::new(expect_rx))
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
pub enum FutureError {
    UnableToParseMessage(String),
    UnableToReceive,
}

impl std::error::Error for FutureError {}

impl std::fmt::Display for FutureError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FutureError::UnableToParseMessage(msg) => {
                write!(f, "unable to parse envelope: {}", msg)
            }
            FutureError::UnableToReceive => f.write_str("unable to receive future result"),
        }
    }
}

/// MessageFuture is a promise for the reply to a sent message.
pub struct MessageFuture<MessageType>
where
    MessageType: std::fmt::Debug + PartialEq,
{
    inner: Box<dyn Receiver<MessageResult<MessageType>>>,
    result: Option<MessageResult<MessageType>>,
}

impl<MessageType> MessageFuture<MessageType>
where
    MessageType: std::fmt::Debug + PartialEq,
{
    pub fn new(inner: Box<dyn Receiver<MessageResult<MessageType>>>) -> Self {
        MessageFuture {
            inner,
            result: None,
        }
    }

    pub fn get<M: FromMessageBytes + Clone>(&mut self) -> Result<M, FutureError> {
        if let Some(result) = self.result.as_ref() {
            return match result {
                Ok((_, msg)) => FromMessageBytes::from_message_bytes(msg)
                    .map_err(|e| FutureError::UnableToParseMessage(e.to_string())),
                Err(_) => Err(FutureError::UnableToReceive),
            };
        }

        let result: MessageResult<MessageType> = self
            .inner
            .recv()
            .map_err(|_| FutureError::UnableToReceive)?;

        self.result = Some(result);

        self.get()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use std::sync::mpsc::channel;
    use std::thread;

    use crate::network::dispatch::RawBytes;

    #[derive(PartialEq, Debug)]
    struct TestType;

    #[test]
    // test that if a message is received without a matching correlation id, the message is routed
    // to the default sender
    fn test_no_correlation_id() {
        let (default_tx, default_rx) = channel();
        let mut inbound_router: InboundRouter<TestType> = InboundRouter::new(Box::new(default_tx));

        thread::Builder::new()
            .name("test_no_correlation_id".to_string())
            .spawn(move || inbound_router.route("test", Ok((TestType, b"test_payload".to_vec()))))
            .unwrap();

        let msg = match default_rx.recv() {
            Ok(msg_result) => msg_result.unwrap(),
            Err(err) => panic!("Received error: {}", err),
        };

        assert_eq!((TestType, b"test_payload".to_vec()), msg);
    }

    #[test]
    // test that if a message is received with a matching correlation id, the message is routed
    // to the receiver that is blocking on the reply
    fn test_expect_reply() {
        let (default_tx, _) = channel();
        let mut inbound_router: InboundRouter<TestType> = InboundRouter::new(Box::new(default_tx));

        let mut fut = inbound_router.expect_reply("test".to_string());
        thread::Builder::new()
            .name("test_expect_reply".to_string())
            .spawn(move || inbound_router.route("test", Ok((TestType, b"test_payload".to_vec()))))
            .unwrap();

        let msg = fut
            .get::<RawBytes>()
            .expect("Unexpected error when resolving future");

        assert_eq!(b"test_payload", msg.bytes());
    }
}
