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

use std::cmp::min;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};

use transact::protocol::batch::BatchPair;
use transact::protos::FromBytes;

use crate::actix_web::{web, Error as ActixError, HttpResponse};
use crate::futures::{stream::Stream, Future, IntoFuture};
use crate::rest_api::{new_websocket_event_sender, EventSender, Method, Request};
use crate::service::rest_api::ServiceEndpoint;

use super::error::StateSubscriberError;
use super::state::{BatchInfo, BatchStatus, StateChangeEvent, StateSubscriber};
use super::{Scabbard, SERVICE_TYPE};

const DEFAULT_BATCH_STATUS_WAIT_SECS: u64 = 300;
const BATCH_STATUS_RETRY_INTERVAL_MILLIS: u64 = 1000;

struct WsStateSubscriber {
    sender: EventSender<StateChangeEvent>,
}

impl StateSubscriber for WsStateSubscriber {
    fn handle_event(&self, event: StateChangeEvent) -> Result<(), StateSubscriberError> {
        self.sender.send(event).map_err(|_| {
            debug!(
                "Dropping scabbard state change event and unsubscribing due to websocket being
                 closed"
            );
            StateSubscriberError::Unsubscribe
        })
    }
}

pub fn make_subscribe_endpoint() -> ServiceEndpoint {
    ServiceEndpoint {
        service_type: SERVICE_TYPE.into(),
        route: "/ws/subscribe".into(),
        method: Method::Get,
        handler: Arc::new(move |request, payload, service| {
            let scabbard = match service.as_any().downcast_ref::<Scabbard>() {
                Some(s) => s,
                None => {
                    return Box::new(HttpResponse::InternalServerError().finish().into_future())
                }
            };

            let mut query =
                match web::Query::<HashMap<String, String>>::from_query(request.query_string()) {
                    Ok(query) => query,
                    Err(_) => return Box::new(HttpResponse::BadRequest().finish().into_future()),
                };

            let last_seen_event_id = query.remove("last_seen_event");

            match last_seen_event_id {
                Some(ref id) if id.trim().is_empty() => {
                    return Box::new(
                        HttpResponse::BadRequest()
                            .json(json!({
                                "message": "last_seen_event must not be empty",
                            }))
                            .into_future(),
                    );
                }
                Some(ref id) => debug!("Getting all state-delta events since {}", id),
                None => debug!("Getting all state-delta events"),
            }

            let unseen_events = match scabbard.get_events_since(last_seen_event_id) {
                Ok(events) => events,
                Err(err) => {
                    error!("Unable to load unseen scabbard events: {}", err);
                    return Box::new(HttpResponse::InternalServerError().finish().into_future());
                }
            };

            let request = Request::from((request, payload));
            match new_websocket_event_sender(request, Box::new(unseen_events)) {
                Ok((sender, res)) => {
                    if let Err(err) =
                        scabbard.add_state_subscriber(Box::new(WsStateSubscriber { sender }))
                    {
                        error!("Unable to add scabbard event sender: {}", err);
                        return Box::new(
                            HttpResponse::InternalServerError().finish().into_future(),
                        );
                    }
                    Box::new(res.into_future())
                }
                Err(err) => {
                    debug!("Failed to create websocket: {:?}", err);
                    Box::new(HttpResponse::InternalServerError().finish().into_future())
                }
            }
        }),
    }
}

pub fn make_add_batches_to_queue_endpoint() -> ServiceEndpoint {
    ServiceEndpoint {
        service_type: SERVICE_TYPE.into(),
        route: "/batches".into(),
        method: Method::Post,
        handler: Arc::new(move |_, payload, service| {
            let scabbard = match service.as_any().downcast_ref::<Scabbard>() {
                Some(s) => s,
                None => {
                    return Box::new(HttpResponse::InternalServerError().finish().into_future())
                }
            }
            .clone();

            Box::new(
                payload
                    .from_err::<ActixError>()
                    .fold(web::BytesMut::new(), move |mut body, chunk| {
                        body.extend_from_slice(&chunk);
                        Ok::<_, ActixError>(body)
                    })
                    .into_future()
                    .and_then(move |body| {
                        let batches: Vec<BatchPair> = match Vec::from_bytes(&body) {
                            Ok(b) => b,
                            Err(_) => return HttpResponse::BadRequest().finish().into_future(),
                        };

                        match scabbard.add_batches(batches) {
                            Ok(Some(link)) => HttpResponse::Accepted().json(link).into_future(),
                            Ok(None) => HttpResponse::BadRequest().finish().into_future(),
                            Err(_) => HttpResponse::InternalServerError().finish().into_future(),
                        }
                    }),
            )
        }),
    }
}

pub fn make_get_batch_status_endpoint() -> ServiceEndpoint {
    ServiceEndpoint {
        service_type: SERVICE_TYPE.into(),
        route: "/batch_statuses".into(),
        method: Method::Get,
        handler: Arc::new(move |req, _, service| {
            let scabbard = match service.as_any().downcast_ref::<Scabbard>() {
                Some(s) => s,
                None => {
                    return Box::new(HttpResponse::InternalServerError().finish().into_future())
                }
            }
            .clone();
            let query: web::Query<HashMap<String, String>> =
                if let Ok(q) = web::Query::from_query(req.query_string()) {
                    q
                } else {
                    return Box::new(
                        HttpResponse::BadRequest()
                            .json(json!({
                                "message": "Invalid query"
                            }))
                            .into_future(),
                    );
                };

            let ids = if let Some(ids) = query.get("ids") {
                ids.split(',').map(String::from).collect::<Vec<_>>()
            } else {
                return Box::new(
                    HttpResponse::BadRequest()
                        .json(json!({
                            "message": "No batch IDs specified"
                        }))
                        .into_future(),
                );
            };

            let wait = query
                .get("wait")
                .and_then(|wait_str| {
                    if wait_str.as_str() == "false" {
                        None
                    } else {
                        wait_str
                            .parse()
                            .ok()
                            .or(Some(DEFAULT_BATCH_STATUS_WAIT_SECS))
                    }
                })
                .map(Duration::from_secs);

            let timeout = match wait {
                Some(wait_time) => match Instant::now().checked_add(wait_time) {
                    Some(t) => Some(t),
                    None => {
                        return Box::new(
                            HttpResponse::InternalServerError()
                                .json("failed to determine wait time")
                                .into_future(),
                        )
                    }
                },
                None => None,
            };

            match get_statuses(&scabbard, &ids, timeout) {
                Ok(statuses) => Box::new(HttpResponse::Ok().json(statuses).into_future()),
                Err(err) => Box::new(HttpResponse::InternalServerError().json(err).into_future()),
            }
        }),
    }
}

#[cfg(feature = "scabbard-get-state")]
pub fn make_get_state_at_address_endpoint() -> ServiceEndpoint {
    ServiceEndpoint {
        service_type: SERVICE_TYPE.into(),
        route: "/state/{address}".into(),
        method: Method::Get,
        handler: Arc::new(move |request, _, service| {
            let scabbard = match service.as_any().downcast_ref::<Scabbard>() {
                Some(s) => s,
                None => {
                    return Box::new(
                        HttpResponse::InternalServerError()
                            .json(json!({
                                "message": "service is not Scabbard"
                            }))
                            .into_future(),
                    )
                }
            };

            let address = request
                .match_info()
                .get("address")
                .expect("address should not be none");

            Box::new(match scabbard.get_state_at_address(address) {
                Ok(Some(value)) => HttpResponse::Ok().json(value).into_future(),
                Ok(None) => HttpResponse::NotFound().finish().into_future(),
                Err(err) => HttpResponse::InternalServerError()
                    .json(json!({
                        "message": err.to_string()
                    }))
                    .into_future(),
            })
        }),
    }
}

#[cfg(feature = "scabbard-get-state")]
pub fn make_get_state_with_prefix_endpoint() -> ServiceEndpoint {
    ServiceEndpoint {
        service_type: SERVICE_TYPE.into(),
        route: "/state".into(),
        method: Method::Get,
        handler: Arc::new(move |request, _, service| {
            let scabbard = match service.as_any().downcast_ref::<Scabbard>() {
                Some(s) => s,
                None => {
                    return Box::new(
                        HttpResponse::InternalServerError()
                            .json(json!({
                                "message": "service is not Scabbard"
                            }))
                            .into_future(),
                    )
                }
            };

            let query: web::Query<HashMap<String, String>> =
                if let Ok(q) = web::Query::from_query(request.query_string()) {
                    q
                } else {
                    return Box::new(
                        HttpResponse::BadRequest()
                            .json(json!({
                                "message": "Invalid query"
                            }))
                            .into_future(),
                    );
                };

            let prefix = query.get("prefix").map(String::as_str);

            Box::new(match scabbard.get_state_with_prefix(prefix) {
                Ok(state_iter) => {
                    let res = state_iter
                        .map(|res| {
                            res.map(|(address, value)| {
                                json!({
                                    "address": address,
                                    "value": value,
                                })
                            })
                        })
                        .collect::<Result<Vec<_>, _>>();
                    match res {
                        Ok(entries) => HttpResponse::Ok().json(entries).into_future(),
                        Err(err) => HttpResponse::InternalServerError()
                            .json(json!({
                                "message": err.to_string()
                            }))
                            .into_future(),
                    }
                }
                Err(err) => HttpResponse::InternalServerError()
                    .json(json!({
                        "message": err.to_string()
                    }))
                    .into_future(),
            })
        }),
    }
}

fn get_statuses(
    scabbard: &Scabbard,
    ids: &[String],
    timeout: Option<Instant>,
) -> Result<Vec<BatchInfo>, String> {
    let batch_infos = scabbard
        .get_batch_info(ids)
        .map_err(|err| err.to_string())?;

    if batch_infos
        .iter()
        .any(|info| info.status == BatchStatus::Pending)
    {
        if let Some(timeout) = timeout {
            let now = Instant::now();
            if now < timeout {
                let normal_wait = Duration::from_millis(BATCH_STATUS_RETRY_INTERVAL_MILLIS);
                let time_remaining = timeout - now;
                sleep(min(normal_wait, time_remaining));
                return get_statuses(scabbard, ids, Some(timeout));
            } else {
                return Err("batch(es) still pending after wait time expired".into());
            }
        }
    }

    Ok(batch_infos)
}
