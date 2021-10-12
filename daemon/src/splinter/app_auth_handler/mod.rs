/*
 * Copyright 2020-2021 Cargill Incorporated
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
mod sabre;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use splinter::{
    admin::messages::AdminServiceEvent,
    events::{Igniter, ParseBytes, ParseError, WebSocketClient, WebSocketError, WsResponse},
};

use crate::event::EventHandler;
use crate::splinter::{
    app_auth_handler::{error::AppAuthHandlerError, node::get_node_id, sabre::setup_grid},
    event::processors::EventProcessors,
};

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

pub fn run(
    splinterd_url: String,
    event_processors: EventProcessors,
    handler: Box<dyn EventHandler>,
    igniter: Igniter,
    scabbard_admin_key: String,
    #[cfg(feature = "cylinder-jwt-support")] authorization: String,
) -> Result<(), AppAuthHandlerError> {
    let registration_route = format!("{}/ws/admin/register/grid", &splinterd_url);
    let node_id = get_node_id(
        splinterd_url.clone(),
        #[cfg(feature = "cylinder-jwt-support")]
        &authorization,
    )?;

    let ws_handler = Arc::new(Mutex::new(handler));
    #[cfg(feature = "cylinder-jwt-support")]
    let ws_auth = authorization.clone();
    let mut ws = WebSocketClient::new(&registration_route, move |_ctx, event| {
        let handler = {
            match ws_handler.lock() {
                Ok(handler) => handler.cloned_box(),
                Err(err) => {
                    warn!("Attempting to recover from a poisoned lock in event handler",);
                    err.into_inner().cloned_box()
                }
            }
        };

        if let Err(err) = process_admin_event(
            event,
            event_processors.clone(),
            handler,
            &node_id,
            &scabbard_admin_key,
            &splinterd_url,
            #[cfg(feature = "cylinder-jwt-support")]
            &ws_auth,
        ) {
            error!("Failed to process admin event: {}", err);
        }
        WsResponse::Empty
    });

    #[cfg(feature = "cylinder-jwt-support")]
    ws.header("Authorization", authorization);

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

fn process_admin_event(
    event: AdminEvent,
    event_processors: EventProcessors,
    handler: Box<dyn EventHandler>,
    node_id: &str,
    scabbard_admin_key: &str,
    splinterd_url: &str,
    #[cfg(feature = "cylinder-jwt-support")] authorization: &str,
) -> Result<(), AppAuthHandlerError> {
    debug!("Received the event at {}", event.timestamp);
    match event.admin_event {
        AdminServiceEvent::CircuitReady(msg_proposal) => {
            let service = match msg_proposal.circuit.roster.iter().find_map(|service| {
                if service.allowed_nodes.contains(&node_id.to_string()) {
                    Some(service)
                } else {
                    None
                }
            }) {
                Some(service) => service,
                None => {
                    debug!(
                        "New circuit does not have any services for this node: {}",
                        node_id
                    );
                    return Ok(());
                }
            };

            let scabbard_args: HashMap<_, _> = service.arguments.iter().cloned().collect();

            let proposed_admin_pubkeys = scabbard_args
                .get("admin_keys")
                .ok_or_else(|| {
                    AppAuthHandlerError::with_message(
                        "Scabbard Service is not properly configured with \"admin_keys\" argument.",
                    )
                })
                .and_then(|keys_str| {
                    serde_json::from_str::<Vec<String>>(keys_str).map_err(|err| {
                        AppAuthHandlerError::with_message(&format!(
                            "unable to parse application metadata: {}",
                            err
                        ))
                    })
                })?;

            event_processors
                .add_once(
                    &msg_proposal.circuit_id,
                    &service.service_id,
                    None,
                    #[cfg(feature = "cylinder-jwt-support")]
                    authorization,
                    || vec![handler.cloned_box()],
                )
                .map_err(|err| AppAuthHandlerError::from_source(Box::new(err)))?;

            setup_grid(
                scabbard_admin_key,
                proposed_admin_pubkeys,
                splinterd_url,
                &service.service_id,
                &msg_proposal.circuit_id,
            )?;
            Ok(())
        }
        _ => Ok(()),
    }
}
