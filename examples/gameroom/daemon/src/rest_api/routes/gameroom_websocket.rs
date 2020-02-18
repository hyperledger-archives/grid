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

use std::time::{Duration, SystemTime};

use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
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

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for GameroomWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.ping(&msg),
            Ok(ws::Message::Pong(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(_)) => ctx.stop(),
            Ok(ws::Message::Continuation(_)) => (),
            Ok(ws::Message::Nop) => (),
            Err(err) => {
                error!("{}", err);
                ctx.stop()
            }
        };
    }
}

pub async fn connect_socket(
    req: HttpRequest,
    pool: web::Data<ConnectionPool>,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    ws::start(GameroomWebSocket::new(pool), &req, stream)
}

fn check_notifications(pool: web::Data<ConnectionPool>) -> Result<bool, RestApiResponseError> {
    let now = SystemTime::now();
    if let Some(earlier) = now.checked_sub(Duration::new(3, 0)) {
        let new_notifications = helpers::fetch_notifications_by_time(&*pool.get()?, now, earlier)?;
        return Ok(!new_notifications.is_empty());
    }
    Err(RestApiResponseError::InternalError(format!(
        "Unable to find new notifications since last check from now: {:?}",
        now.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::new(0, 0))
            .as_secs(),
    )))
}
