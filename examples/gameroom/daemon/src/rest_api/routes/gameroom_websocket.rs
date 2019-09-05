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
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use futures::{Future, IntoFuture};
use gameroom_database::{helpers, ConnectionPool};

use crate::rest_api::RestApiResponseError;

pub struct GameroomWebSocket {
    db_pool: web::Data<ConnectionPool>,
}

impl GameroomWebSocket {
    fn new(pool: web::Data<ConnectionPool>) -> Self {
        Self { db_pool: pool }
    }

    fn push_updates(&self, ctx: &mut <Self as Actor>::Context) {
        trace!("Gameroom wants to sock-et to you");
        ctx.run_interval(Duration::from_secs(3), |ws, ctx| match check_notifications(
            ws.db_pool.clone(),
        ) {
            Ok(true) => match serde_json::to_string(
                &json!({"namespace": "notifications", "action": "listNotifications"}),
            ) {
                Ok(text) => ctx.text(text),
                Err(err) => {
                    debug!("Failed to serialize payload: {:?}", err);
                }
            },

            Ok(false) => trace!("No new notifications"),
            Err(err) => debug!("Error getting notification: {:?}", err),
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

pub fn connect_socket(
    req: HttpRequest,
    pool: web::Data<ConnectionPool>,
    stream: web::Payload,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(ws::start(GameroomWebSocket::new(pool), &req, stream).into_future())
}

fn check_notifications(pool: web::Data<ConnectionPool>) -> Result<bool, RestApiResponseError> {
    Ok(helpers::get_unread_notification_count(&*pool.get()?)? > 0)
}
