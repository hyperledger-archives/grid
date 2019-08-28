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

use std::time::Duration;

use actix::prelude::*;
use actix_web_actors::ws;

pub struct GameroomWebSocket {}

impl GameroomWebSocket {
    fn new() -> Self {
        Self {}
    }

    fn push_updates(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(Duration::from_secs(3), |_, ctx| {
            ctx.ping("");
            debug!("Gameroom wants to sock-et to you");
        });
    }
}

impl Actor for GameroomWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("Starting Gameroom web socket");
        self.push_updates(ctx)
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for GameroomWebSocket {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        debug!("WS: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => ctx.ping(&msg),
            ws::Message::Pong(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => ctx.text(text),
            ws::Message::Binary(bin) => ctx.binary(bin),
            ws::Message::Close(_) => ctx.stop(),
            ws::Message::Nop => (),
        };
    }
}
