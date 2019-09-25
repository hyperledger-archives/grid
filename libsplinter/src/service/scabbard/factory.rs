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
use crate::rest_api::{Method, Request};
use crate::service::{FactoryCreateError, Service, ServiceEndpoint, ServiceFactory};

use super::{Scabbard, SERVICE_TYPE};

const DEFAULT_DB_DIR: &str = "/var/lib/splinter";
const DEFAULT_DB_SIZE: usize = 1028 * 1028 * 1028;

pub struct ScabbardFactory {
    service_types: Vec<String>,
    db_dir: String,
    db_size: usize,
}

impl ScabbardFactory {
    pub fn new(db_dir: Option<String>, db_size: Option<usize>) -> Self {
        ScabbardFactory {
            service_types: vec![SERVICE_TYPE.into()],
            db_dir: db_dir.unwrap_or_else(|| DEFAULT_DB_DIR.into()),
            db_size: db_size.unwrap_or(DEFAULT_DB_SIZE),
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
        let db_dir = Path::new(&self.db_dir);
        let admin_keys_str = args.get("admin_keys").ok_or_else(|| {
            FactoryCreateError::InvalidArguments("admin_keys argument not provided".into())
        })?;
        let admin_keys = serde_json::from_str(admin_keys_str).map_err(|err| {
            FactoryCreateError::InvalidArguments(format!(
                "failed to parse admin_keys list: {}",
                err,
            ))
        })?;

        let service = Scabbard::new(service_id, peer_services, &db_dir, self.db_size, admin_keys)
            .map_err(|err| FactoryCreateError::CreationFailed(Box::new(err)))?;

        Ok(Box::new(service))
    }

    fn get_rest_endpoints(&self) -> Vec<ServiceEndpoint> {
        vec![
            make_add_batches_to_queue_endpoint(),
            make_subscribe_endpoint(),
        ]
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

            match scabbard.subscribe_to_state(Request::from((request, payload))) {
                Ok(Ok(response)) => Box::new(response.into_future()),
                _ => Box::new(HttpResponse::InternalServerError().finish().into_future()),
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
                            Ok(_) => HttpResponse::Accepted().finish().into_future(),
                            Err(_) => HttpResponse::InternalServerError().finish().into_future(),
                        }
                    }),
            )
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let factory = ScabbardFactory::new(Some("/tmp".into()), Some(1024 * 1024));
        let service = factory
            .create("0".into(), "", "", args)
            .expect("failed to create service");

        assert_eq!(service.service_id(), "0");
    }
}
