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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time;

use crate::actix_web::HttpResponse;
use crate::futures::{Future, IntoFuture};
use crate::protos::admin::CircuitManagementPayload;
use crate::rest_api::{
    into_protobuf, EventDealer, EventSender, Method, Request, Resource, Response, ResponseError,
    RestResourceProvider,
};
use crate::service::ServiceError;

use super::messages::AdminServiceEvent;
use super::service::{
    AdminCommands, AdminService, AdminServiceError, AdminServiceEventSubscriber,
    AdminSubscriberError,
};

impl RestResourceProvider for AdminService {
    fn resources(&self) -> Vec<Resource> {
        vec![
            make_application_handler_registration_route(self.commands()),
            make_submit_route(self.commands()),
        ]
    }
}

fn make_submit_route<A: AdminCommands + Clone + 'static>(admin_commands: A) -> Resource {
    Resource::build("/admin/submit").add_method(Method::Post, move |_, payload| {
        let admin_commands = admin_commands.clone();
        Box::new(
            into_protobuf::<CircuitManagementPayload>(payload).and_then(move |payload| {
                match admin_commands.submit_circuit_change(payload) {
                    Ok(()) => HttpResponse::Accepted().finish().into_future(),
                    Err(AdminServiceError::ServiceError(ServiceError::UnableToHandleMessage(
                        err,
                    ))) => HttpResponse::BadRequest()
                        .json(json!({
                            "message": format!("Unable to handle message: {}", err)
                        }))
                        .into_future(),
                    Err(AdminServiceError::ServiceError(ServiceError::InvalidMessageFormat(
                        err,
                    ))) => HttpResponse::BadRequest()
                        .json(json!({
                            "message": format!("Failed to parse payload: {}", err)
                        }))
                        .into_future(),
                    Err(err) => {
                        error!("{}", err);
                        HttpResponse::InternalServerError().finish().into_future()
                    }
                }
            }),
        )
    })
}

fn make_application_handler_registration_route<A: AdminCommands + Clone + 'static>(
    admin_commands: A,
) -> Resource {
    let admin_event_dealers = AdminEventDealers::default();
    Resource::build("/ws/admin/register/{type}").add_method(Method::Get, move |request, payload| {
        let circuit_management_type = if let Some(t) = request.match_info().get("type") {
            t.to_string()
        } else {
            return Box::new(HttpResponse::BadRequest().finish().into_future());
        };

        let initial_events = match admin_commands
            .get_events_since(&time::SystemTime::UNIX_EPOCH, &circuit_management_type)
        {
            Ok(events) => events.map(JsonAdminEvent::from),
            Err(err) => {
                error!(
                    "Unable to load initial set of admin events for {}: {}",
                    &circuit_management_type, err
                );
                return Box::new(HttpResponse::InternalServerError().finish().into_future());
            }
        };

        let request = Request::from((request, payload));
        debug!("Circuit management type \"{}\"", circuit_management_type);
        match admin_event_dealers.add_event_dealer(
            request,
            &circuit_management_type,
            Box::new(initial_events),
        ) {
            Ok((sender, res)) => {
                debug!("Websocket response: {:?}", res);
                if let Err(err) = admin_commands.add_event_subscriber(
                    &circuit_management_type,
                    Box::new(WsAdminServiceEventSubscriber { sender }),
                ) {
                    error!("Unable to add admin event subscriber: {}", err);
                    return Box::new(HttpResponse::InternalServerError().finish().into_future());
                }
                Box::new(res.into_future())
            }
            Err(err) => {
                debug!("Failed to create websocket: {:?}", err);
                Box::new(HttpResponse::InternalServerError().finish().into_future())
            }
        }
    })
}

#[derive(Clone, Default)]
struct AdminEventDealers {
    event_dealers_by_type: Arc<Mutex<HashMap<String, EventDealer<JsonAdminEvent>>>>,
}

impl AdminEventDealers {
    fn add_event_dealer(
        &self,
        request: Request,
        event_type: &str,
        initial_events: Box<dyn Iterator<Item = JsonAdminEvent> + Send>,
    ) -> Result<(EventSender<JsonAdminEvent>, Response), ResponseError> {
        let mut event_dealers = self.event_dealers_by_type.lock().unwrap();
        let dealer = event_dealers
            .entry(event_type.to_string())
            .or_insert_with(EventDealer::new);
        dealer.subscribe(request, initial_events)
    }
}

struct WsAdminServiceEventSubscriber {
    sender: EventSender<JsonAdminEvent>,
}

impl AdminServiceEventSubscriber for WsAdminServiceEventSubscriber {
    fn handle_event(
        &self,
        event: &AdminServiceEvent,
        timestamp: &time::SystemTime,
    ) -> Result<(), AdminSubscriberError> {
        let json_event = JsonAdminEvent {
            timestamp: *timestamp,
            event: event.clone(),
        };
        self.sender.send(json_event).map_err(|_| {
            debug!("Dropping admin service event and unsubscribing due to websocket being closed");
            AdminSubscriberError::Unsubscribe
        })
    }
}

#[derive(Debug, Serialize, Clone)]
struct JsonAdminEvent {
    #[serde(serialize_with = "st_as_millis")]
    timestamp: time::SystemTime,

    #[serde(flatten)]
    event: AdminServiceEvent,
}

impl From<(time::SystemTime, AdminServiceEvent)> for JsonAdminEvent {
    fn from(raw_evt: (time::SystemTime, AdminServiceEvent)) -> Self {
        Self {
            timestamp: raw_evt.0,
            event: raw_evt.1,
        }
    }
}

pub fn st_as_millis<S>(data: &time::SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let since_the_epoch = data
        .duration_since(time::UNIX_EPOCH)
        .expect("Time went backwards");

    serializer.serialize_u128(since_the_epoch.as_millis())
}
