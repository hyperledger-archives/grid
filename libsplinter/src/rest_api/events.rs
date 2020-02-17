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

use std::fmt::Debug;
use std::time::Duration;

use actix::prelude::*;
use actix_web_actors::ws::{self, CloseCode, CloseReason};
use futures::{
    stream::{iter_ok, Stream},
    sync::mpsc::{unbounded, UnboundedSender},
};
use serde::ser::Serialize;
use serde_json;

use crate::rest_api::{errors::ResponseError, Request, Response};

/// Wait time in seconds between ping messages being sent by the ws server to the ws client
const PING_INTERVAL: u64 = 30;

pub fn new_websocket_event_sender<T: Serialize + Debug>(
    req: Request,
    initial_events: Box<dyn Iterator<Item = T> + Send>,
) -> Result<(EventSender<T>, Response), ResponseError> {
    let (sender, recv) = unbounded();

    let (request, payload) = req.into();

    let stream = iter_ok::<_, ()>(initial_events.map(MessageWrapper::Message)).chain(recv);

    let res = ws::start(
        EventSenderWebSocket::new(Box::new(stream)),
        &request,
        payload,
    )
    .map_err(ResponseError::from)?;

    Ok((EventSender { sender }, Response::from(res)))
}

#[derive(Clone)]
pub struct EventSender<T: Serialize + Debug + 'static> {
    sender: UnboundedSender<MessageWrapper<T>>,
}

impl<T: Serialize + Debug + 'static> EventSender<T> {
    pub fn send(&self, event: T) -> Result<(), EventSendError<T>> {
        trace!("Event sent: {:?}", &event);
        self.sender
            .unbounded_send(MessageWrapper::Message(event))
            .map_err(|err| match err.into_inner() {
                MessageWrapper::Message(event) => EventSendError(event),
                _ => {
                    panic!("Sent an Message variant, but didn't receive the same variant on error")
                }
            })
    }

    pub fn shutdown(self) {
        if self
            .sender
            .unbounded_send(MessageWrapper::Shutdown)
            .is_err()
        {
            debug!("Attempting to shutdown an already stopped websocket");
        }
    }
}

impl<T: Serialize + Debug + 'static> Drop for EventSender<T> {
    fn drop(&mut self) {
        if self
            .sender
            .unbounded_send(MessageWrapper::Shutdown)
            .is_err()
        {
            debug!("Attempting to shutdown an already stopped websocket");
        }
    }
}

#[derive(Debug)]
pub struct EventSendError<T: Serialize + Debug + 'static>(pub T);

struct EventSenderWebSocket<T: Serialize + Debug + 'static> {
    stream: Option<Box<dyn Stream<Item = MessageWrapper<T>, Error = ()>>>,
}

impl<T: Serialize + Debug + 'static> EventSenderWebSocket<T> {
    fn new(stream: Box<dyn Stream<Item = MessageWrapper<T>, Error = ()>>) -> Self {
        Self {
            stream: Some(stream),
        }
    }
}

impl<T: Serialize + Debug + 'static> StreamHandler<MessageWrapper<T>, ()>
    for EventSenderWebSocket<T>
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

impl<T: Serialize + Debug + 'static> Actor for EventSenderWebSocket<T> {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        if let Some(stream) = self.stream.take() {
            debug!("Starting Event Websocket");
            ctx.add_stream(stream);
            ctx.run_interval(Duration::from_secs(PING_INTERVAL), move |_, ctx| {
                trace!("Sending Ping");
                ctx.ping("");
            });
        } else {
            warn!("Event dealer websocket was unexpectedly started twice; ignoring");
        }
    }
}

impl<T: Serialize + Debug + 'static> StreamHandler<ws::Message, ws::ProtocolError>
    for EventSenderWebSocket<T>
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
