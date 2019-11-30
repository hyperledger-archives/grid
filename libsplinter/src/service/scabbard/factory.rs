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

use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::path::Path;
use std::sync::Arc;

use serde_json;

use transact::protocol::batch::BatchPair;
use transact::protos::FromBytes;

use crate::actix_web::{web, Error as ActixError, HttpResponse};
use crate::futures::{stream::Stream, Future, IntoFuture};
use crate::rest_api::{new_websocket_event_sender, EventSender, Method, Request};
use crate::service::{FactoryCreateError, Service, ServiceEndpoint, ServiceFactory};
use crate::signing::SignatureVerifierFactory;

use super::error::StateSubscriberError;
use super::state::{StateChangeEvent, StateSubscriber};
use super::{Scabbard, SERVICE_TYPE};

const DEFAULT_STATE_DB_DIR: &str = "/var/lib/splinter";
const DEFAULT_STATE_DB_SIZE: usize = 1 << 30; // 1024 ** 3
const DEFAULT_RECEIPT_DB_DIR: &str = "/var/lib/splinter";
const DEFAULT_RECEIPT_DB_SIZE: usize = 1 << 30; // 1024 ** 3

pub struct ScabbardFactory {
    service_types: Vec<String>,
    state_db_dir: String,
    state_db_size: usize,
    receipt_db_dir: String,
    receipt_db_size: usize,
    signature_verifier_factory: Box<dyn SignatureVerifierFactory>,
}

impl ScabbardFactory {
    pub fn new(
        state_db_dir: Option<String>,
        state_db_size: Option<usize>,
        receipt_db_dir: Option<String>,
        receipt_db_size: Option<usize>,
        signature_verifier_factory: Box<dyn SignatureVerifierFactory>,
    ) -> Self {
        ScabbardFactory {
            service_types: vec![SERVICE_TYPE.into()],
            state_db_dir: state_db_dir.unwrap_or_else(|| DEFAULT_STATE_DB_DIR.into()),
            state_db_size: state_db_size.unwrap_or(DEFAULT_STATE_DB_SIZE),
            receipt_db_dir: receipt_db_dir.unwrap_or_else(|| DEFAULT_RECEIPT_DB_DIR.into()),
            receipt_db_size: receipt_db_size.unwrap_or(DEFAULT_RECEIPT_DB_SIZE),
            signature_verifier_factory,
        }
    }
}

impl ServiceFactory for ScabbardFactory {
    fn available_service_types(&self) -> &[String] {
        self.service_types.as_slice()
    }

    /// `args` should include the following:
    /// - `admin_keys`: list of public keys that are allowed to create and modify sabre contracts,
    ///   formatted as a serialized JSON array of strings
    /// - `peer_services`: list of other scabbard services on the same circuit that this service
    ///   will share state with
    fn create(
        &self,
        service_id: String,
        _service_type: &str,
        circuit_id: &str,
        args: HashMap<String, String>,
    ) -> Result<Box<dyn Service>, FactoryCreateError> {
        let peer_services_str = args.get("peer_services").ok_or_else(|| {
            FactoryCreateError::InvalidArguments("peer_services argument not provided".into())
        })?;
        let peer_services = HashSet::from_iter(
            serde_json::from_str::<Vec<_>>(peer_services_str)
                .map_err(|err| {
                    FactoryCreateError::InvalidArguments(format!(
                        "failed to parse peer_services list: {}",
                        err,
                    ))
                })?
                .into_iter(),
        );
        let state_db_dir = Path::new(&self.state_db_dir);
        let receipt_db_dir = Path::new(&self.receipt_db_dir);
        let admin_keys_str = args.get("admin_keys").ok_or_else(|| {
            FactoryCreateError::InvalidArguments("admin_keys argument not provided".into())
        })?;
        let admin_keys = serde_json::from_str(admin_keys_str).map_err(|err| {
            FactoryCreateError::InvalidArguments(format!(
                "failed to parse admin_keys list: {}",
                err,
            ))
        })?;

        let service = Scabbard::new(
            service_id,
            circuit_id,
            peer_services,
            &state_db_dir,
            self.state_db_size,
            &receipt_db_dir,
            self.receipt_db_size,
            self.signature_verifier_factory.create_verifier(),
            admin_keys,
        )
        .map_err(|err| FactoryCreateError::CreationFailed(Box::new(err)))?;

        Ok(Box::new(service))
    }

    fn get_rest_endpoints(&self) -> Vec<ServiceEndpoint> {
        vec![
            make_add_batches_to_queue_endpoint(),
            make_subscribe_endpoint(),
            make_get_batch_status_endpoint(),
        ]
    }
}

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

fn make_subscribe_endpoint() -> ServiceEndpoint {
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

            if let Some(ref id) = last_seen_event_id {
                debug!("Getting all state-delta events since {}", id);
            } else {
                debug!("Getting all state-delta events");
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

fn make_add_batches_to_queue_endpoint() -> ServiceEndpoint {
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

fn make_get_batch_status_endpoint() -> ServiceEndpoint {
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
                ids.split(',').map(String::from).collect()
            } else {
                return Box::new(
                    HttpResponse::BadRequest()
                        .json(json!({
                            "message": "No batch IDs specified"
                        }))
                        .into_future(),
                );
            };

            if let Ok(status) = scabbard.get_batch_info(ids) {
                Box::new(HttpResponse::Ok().json(status).into_future())
            } else {
                Box::new(HttpResponse::InternalServerError().finish().into_future())
            }
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::signing::hash::HashVerifier;

    #[test]
    fn scabbard_factory() {
        let peer_services = vec!["1".to_string(), "2".to_string(), "3".to_string()];
        let admin_keys: Vec<String> = vec![];
        let mut args = HashMap::new();
        args.insert(
            "peer_services".into(),
            serde_json::to_string(&peer_services).expect("failed to serialize peer_services"),
        );
        args.insert(
            "admin_keys".into(),
            serde_json::to_string(&admin_keys).expect("failed to serialize admin_keys"),
        );

        let factory = ScabbardFactory::new(
            Some("/tmp".into()),
            Some(1024 * 1024),
            Some("/tmp".into()),
            Some(1024 * 1024),
            Box::new(HashVerifier),
        );
        let service = factory
            .create("0".into(), "", "", args)
            .expect("failed to create service");

        assert_eq!(service.service_id(), "0");
    }
}
