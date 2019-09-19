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

use std::default::Default;
use std::fmt::Debug;
use std::time::Duration;

use actix::prelude::*;
use actix_web_actors::ws::{self, CloseCode, CloseReason};
use crossbeam_channel::{unbounded, Receiver, Sender, TryRecvError};
use serde::ser::Serialize;
use serde_json;

use crate::rest_api::{errors::ResponseError, Request, Response};

/// `EventDealer` is responsible for creating and managing WebSockets for Services that need to
/// push messages to a web based client.
#[derive(Debug, Clone)]
pub struct EventDealer<T: Serialize + Debug + Clone + 'static> {
    senders: Vec<Sender<MessageWrapper<T>>>,
}

impl<T: Serialize + Debug + Clone + 'static> EventDealer<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new Websocket and sender receiver pair
    pub fn subscribe(&mut self, req: Request) -> Result<Response, ResponseError> {
        let (send, recv) = unbounded();

        let (request, payload) = req.into();
        let res = ws::start(EventDealerWebSocket::new(recv), &request, payload)
            .map_err(ResponseError::from)?;

        self.add_sender(send);

        Ok(Response::from(res))
    }

    /// Send message to all created WebSockets
    pub fn dispatch(&mut self, msg: T) {
        self.senders.retain(|sender| {
            if let Err(err) = sender.send(MessageWrapper::Message(msg.clone())) {
                warn!("Dropping sender due to error: {}", err);
                false
            } else {
                trace!("Message sent: {:?}", msg);
                true
            }
        });
    }

    fn add_sender(&mut self, sender: Sender<MessageWrapper<T>>) {
        self.senders.push(sender);
    }
}

impl<T: Serialize + Debug + Clone + 'static> Default for EventDealer<T> {
    fn default() -> Self {
        Self {
            senders: Vec::new(),
        }
    }
}

struct EventDealerWebSocket<T: Serialize + Debug + 'static> {
    recv: Receiver<MessageWrapper<T>>,
}

impl<T: Serialize + Debug + 'static> EventDealerWebSocket<T> {
    fn new(recv: Receiver<MessageWrapper<T>>) -> Self {
        Self { recv }
    }

    fn push_updates(&self, recv: Receiver<MessageWrapper<T>>, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(Duration::from_secs(5), move |_, ctx| {
            match recv.try_recv() {
                Ok(MessageWrapper::Message(msg)) => {
                    debug!("Received a message: {:?}", msg);
                    match serde_json::to_string(&msg) {
                        Ok(text) => ctx.text(text),
                        Err(err) => {
                            debug!("Failed to serialize payload: {:?}", err);
                        }
                    }
                }
                Ok(MessageWrapper::Shutdown) => {
                    debug!("Shutting down websocket");
                    ctx.close(Some(CloseReason {
                        description: None,
                        code: CloseCode::Away,
                    }));
                    ctx.stop()
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    debug!("Received channel disconnect");
                    ctx.close(Some(CloseReason {
                        description: Some("Unexpected disconnect from service".into()),
                        code: CloseCode::Error,
                    }));
                    ctx.stop();
                }
            };
        });
    }
}

impl<T: Serialize + Debug + 'static> Actor for EventDealerWebSocket<T> {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("Starting Event Websocket");
        let recv = self.recv.clone();
        self.push_updates(recv, ctx)
    }
}

impl<T: Serialize + Debug + 'static> StreamHandler<ws::Message, ws::ProtocolError>
    for EventDealerWebSocket<T>
{
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Ping(msg) => ctx.ping(&msg),
            ws::Message::Pong(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => ctx.text(text),
            ws::Message::Binary(bin) => ctx.binary(bin),
            ws::Message::Close(_) => {
                ctx.close(Some(CloseReason {
                    description: Some("Received close frame closing normally".into()),
                    code: CloseCode::Normal,
                }));
                ctx.stop()
            }
            ws::Message::Nop => (),
        };
    }
}

#[derive(Debug)]
enum MessageWrapper<T: Serialize + Debug + 'static> {
    Message(T),
    Shutdown,
}
