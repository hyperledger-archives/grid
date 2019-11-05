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

use std::collections::VecDeque;
use std::default::Default;
use std::fmt::Debug;
use std::time::Duration;

use actix::prelude::*;
use actix_web_actors::ws::{self, CloseCode, CloseReason};
use futures::sync::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use serde::ser::Serialize;
use serde_json;

use crate::rest_api::{
    errors::{EventHistoryError, ResponseError},
    Request, Response,
};

/// Wait time in seconds between ping messages being sent by the ws server to the ws client
const PING_INTERVAL: u64 = 30;

/// `EventDealer` is responsible for creating and managing WebSockets for Services that need to
/// push messages to a web based client.
#[derive(Debug, Clone)]
pub struct EventDealer<
    T: Serialize + Debug + Clone + 'static,
    H: EventHistory<T> + Send + Sync + Default,
> {
    senders: Vec<UnboundedSender<MessageWrapper<T>>>,
    history: H,
}

impl<T: Serialize + Debug + Clone + 'static, H: EventHistory<T> + Send + Sync + Default>
    EventDealer<T, H>
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new Websocket and sender receiver pair
    pub fn subscribe(&mut self, req: Request) -> Result<Response, ResponseError> {
        let (send, recv) = unbounded();

        let (request, payload) = req.into();
        let res = ws::start(EventDealerWebSocket::new(recv), &request, payload)
            .map_err(ResponseError::from)?;

        self.add_sender(send)?;

        Ok(Response::from(res))
    }

    /// Send message to all created WebSockets
    pub fn dispatch(&mut self, msg: T) -> Result<(), EventHistoryError> {
        self.history.store(msg.clone())?;
        self.senders.retain(|sender| {
            if let Err(err) = sender.unbounded_send(MessageWrapper::Message(msg.clone())) {
                warn!("Dropping sender due to error: {}", err);
                false
            } else {
                trace!("Message sent: {:?}", msg);
                true
            }
        });
        Ok(())
    }

    pub fn stop(&self) {
        debug!("Stoping WebSockets...");
        self.senders.iter().for_each(|sender| {
            if let Err(err) = sender.unbounded_send(MessageWrapper::Shutdown) {
                error!("Failed to shutdown webocket: {:?}", err);
            }
        });
    }

    fn add_sender(
        &mut self,
        sender: UnboundedSender<MessageWrapper<T>>,
    ) -> Result<(), EventHistoryError> {
        debug!("Catching up new connection");
        self.history.events()?.into_iter().for_each(|msg| {
            if let Err(err) = sender.unbounded_send(MessageWrapper::Message(msg.clone())) {
                error!(
                    "Failed to send message to Websocket Message: {:?}, Error: {}",
                    msg, err
                );
            }
        });
        self.senders.push(sender);
        Ok(())
    }
}

impl<T: Serialize + Debug + Clone + 'static, H: EventHistory<T> + Send + Sync + Default> Default
    for EventDealer<T, H>
{
    fn default() -> Self {
        Self {
            senders: Vec::new(),
            history: H::default(),
        }
    }
}

struct EventDealerWebSocket<T: Serialize + Debug + 'static> {
    recv: Option<UnboundedReceiver<MessageWrapper<T>>>,
}

impl<T: Serialize + Debug + 'static> EventDealerWebSocket<T> {
    fn new(recv: UnboundedReceiver<MessageWrapper<T>>) -> Self {
        Self { recv: Some(recv) }
    }
}

impl<T: Serialize + Debug + 'static> StreamHandler<MessageWrapper<T>, ()>
    for EventDealerWebSocket<T>
{
    fn handle(&mut self, msg: MessageWrapper<T>, ctx: &mut Self::Context) {
        match msg {
            MessageWrapper::Message(msg) => {
                debug!("Received a message: {:?}", msg);
                match serde_json::to_string(&msg) {
                    Ok(text) => ctx.text(text),
                    Err(err) => {
                        debug!("Failed to serialize payload: {:?}", err);
                    }
                }
            }
            MessageWrapper::Shutdown => {
                debug!("Shutting down websocket");
                ctx.close(Some(CloseReason {
                    description: None,
                    code: CloseCode::Away,
                }));
                ctx.stop();
            }
        }
    }

    fn error(&mut self, _: (), ctx: &mut Self::Context) -> Running {
        debug!("Received channel disconnect");
        ctx.close(Some(CloseReason {
            description: None,
            code: CloseCode::Error,
        }));

        Running::Stop
    }
}

impl<T: Serialize + Debug + 'static> Actor for EventDealerWebSocket<T> {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        if let Some(recv) = self.recv.take() {
            debug!("Starting Event Websocket");
            ctx.add_stream(recv);
            ctx.run_interval(Duration::from_secs(PING_INTERVAL), move |_, ctx| {
                debug!("Sending Ping");
                ctx.ping("");
            });
        } else {
            warn!("Event dealer websocket was unexpectedly started twice; ignoring");
        }
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
                debug!("Received close message");
                ctx.stop()
            }
            ws::Message::Nop => (),
        };
    }
}

#[derive(Debug, Message)]
enum MessageWrapper<T: Serialize + Debug + 'static> {
    Message(T),
    Shutdown,
}

/// A trait used for implementing different schemes for
/// storing events.
pub trait EventHistory<T: Clone + Debug>: Clone + Debug {
    /// Add an event to the event history
    fn store(&mut self, event: T) -> Result<(), EventHistoryError>;

    /// Retrieves a list of events
    fn events(&self) -> Result<Vec<T>, EventHistoryError>;
}

/// An implementation of EventHistory for storing
/// events in memory. Only the n most recent events
/// are stored.
#[derive(Clone, Debug)]
pub struct LocalEventHistory<T: Clone + Debug> {
    history: VecDeque<T>,
    limit: usize,
}

impl<T: Clone + Debug> LocalEventHistory<T> {
    pub fn with_limit(limit: usize) -> Self {
        Self {
            history: VecDeque::new(),
            limit,
        }
    }

    /// Creates a LocalEventHistory with a default limit
    /// of 100 events.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: Clone + Debug> EventHistory<T> for LocalEventHistory<T> {
    fn store(&mut self, event: T) -> Result<(), EventHistoryError> {
        self.history.push_back(event);
        if self.history.len() > self.limit {
            self.history.pop_front();
        }
        Ok(())
    }

    fn events(&self) -> Result<Vec<T>, EventHistoryError> {
        Ok(self.history.clone().into_iter().collect())
    }
}

impl<T: Clone + Debug> Default for LocalEventHistory<T> {
    fn default() -> Self {
        Self {
            history: VecDeque::new(),
            limit: 100,
        }
    }
}
