/*
 * Copyright 2020 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

pub mod error;
mod node;

use splinter::{
    admin::messages::AdminServiceEvent,
    events::{Igniter, ParseBytes, ParseError, WebSocketClient, WebSocketError, WsResponse},
};

use crate::app_auth_handler::error::AppAuthHandlerError;

/// default value if the client should attempt to reconnet if ws connection is lost
const RECONNECT: bool = true;

/// default limit for number of consecutives failed reconnection attempts
const RECONNECT_LIMIT: u64 = 10;

/// default timeout in seconds if no message is received from server
const CONNECTION_TIMEOUT: u64 = 60;

#[derive(Deserialize, Debug, Clone)]
struct AdminEvent {
    timestamp: u64,

    #[serde(flatten)]
    admin_event: AdminServiceEvent,
}

impl ParseBytes<AdminEvent> for AdminEvent {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        serde_json::from_slice(bytes).map_err(|err| ParseError::MalformedMessage(Box::new(err)))
    }
}

pub fn run(splinterd_url: String, igniter: Igniter) -> Result<(), AppAuthHandlerError> {
    let registration_route = format!("{}/ws/admin/register/grid", &splinterd_url);

    let mut ws = WebSocketClient::new(&registration_route, move |_ctx, event| {
        if let Err(err) = process_admin_event(event) {
            error!("Failed to process admin event: {}", err);
        }
        WsResponse::Empty
    });

    ws.set_reconnect(RECONNECT);
    ws.set_reconnect_limit(RECONNECT_LIMIT);
    ws.set_timeout(CONNECTION_TIMEOUT);

    ws.on_error(move |err, ctx| {
        error!("An error occured while listening for admin events {}", err);
        match err {
            WebSocketError::ParserError { .. } => {
                debug!("Protocol error, closing connection");
                Ok(())
            }
            WebSocketError::ReconnectError(_) => {
                debug!("Failed to reconnect. Closing WebSocket.");
                Ok(())
            }
            _ => {
                debug!("Attempting to restart connection");
                ctx.start_ws()
            }
        }
    });
    igniter.start_ws(&ws).map_err(AppAuthHandlerError::from)
}

fn process_admin_event(event: AdminEvent) -> Result<(), AppAuthHandlerError> {
    debug!("Received the event at {}", event.timestamp);
    match event.admin_event {
        _ => {
            unimplemented!();
        }
    }
}
