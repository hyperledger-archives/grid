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

use std::sync::Arc;

use crate::actix_web::HttpResponse;
use crate::futures::{Future, IntoFuture};
use crate::protocol;
use crate::rest_api::{
    into_bytes, ErrorResponse, HandlerFunction, Method, ProtocolVersionRangeGuard, Resource,
};

use crate::biome::key_management::{
    store::{KeyStore, KeyStoreError},
    Key,
};
use crate::biome::rest_api::BiomeRestConfig;
use crate::biome::secrets::SecretManager;

use super::super::resources::authorize::AuthorizationResult;
use super::super::resources::key_management::{NewKey, ResponseKey, UpdatedKey};
use super::authorize::authorize_user;

/// Defines the `/biome/users/{user_id}/keys` REST resource for managing keys
pub fn make_key_management_route(
    rest_config: Arc<BiomeRestConfig>,
    key_store: Arc<dyn KeyStore<Key>>,
    secret_manager: Arc<dyn SecretManager>,
) -> Resource {
    Resource::build("/biome/users/{user_id}/keys")
        .add_request_guard(ProtocolVersionRangeGuard::new(
            protocol::BIOME_KEYS_PROTOCOL_MIN,
            protocol::BIOME_PROTOCOL_VERSION,
        ))
        .add_method(
            Method::Post,
            handle_post(
                rest_config.clone(),
                key_store.clone(),
                secret_manager.clone(),
            ),
        )
        .add_method(
            Method::Get,
            handle_get(
                rest_config.clone(),
                key_store.clone(),
                secret_manager.clone(),
            ),
        )
        .add_method(
            Method::Patch,
            handle_patch(rest_config, key_store, secret_manager),
        )
}

/// Defines a REST endpoint for adding a key to the underlying storage
fn handle_post(
    rest_config: Arc<BiomeRestConfig>,
    key_store: Arc<dyn KeyStore<Key>>,
    secret_manager: Arc<dyn SecretManager>,
) -> HandlerFunction {
    Box::new(move |request, payload| {
        let key_store = key_store.clone();
        let user_id = match request.match_info().get("user_id") {
            Some(id) => id.to_owned(),
            None => {
                error!("User ID is not in path request");
                return Box::new(
                    HttpResponse::InternalServerError()
                        .json(ErrorResponse::internal_error())
                        .into_future(),
                );
            }
        };

        match authorize_user(&request, &user_id, &secret_manager, &rest_config) {
            AuthorizationResult::Authorized => (),
            AuthorizationResult::Unauthorized(msg) => {
                return Box::new(
                    HttpResponse::Unauthorized()
                        .json(ErrorResponse::unauthorized(&msg))
                        .into_future(),
                )
            }
            AuthorizationResult::Failed => {
                return Box::new(
                    HttpResponse::InternalServerError()
                        .json(ErrorResponse::internal_error())
                        .into_future(),
                );
            }
        }

        Box::new(into_bytes(payload).and_then(move |bytes| {
            let new_key = match serde_json::from_slice::<NewKey>(&bytes) {
                Ok(val) => val,
                Err(err) => {
                    debug!("Error parsing payload {}", err);
                    return HttpResponse::BadRequest()
                        .json(ErrorResponse::bad_request(&format!(
                            "Failed to parse payload: {}",
                            err
                        )))
                        .into_future();
                }
            };
            let key = Key::new(
                &new_key.public_key,
                &new_key.encrypted_private_key,
                &user_id,
                &new_key.display_name,
            );
            let response_key = ResponseKey::from(&key);

            match key_store.add_key(key.clone()) {
                Ok(()) => HttpResponse::Ok()
                    .json(json!({ "message": "Key added successfully", "data": response_key }))
                    .into_future(),
                Err(err) => {
                    debug!("Failed to add new key to database {}", err);
                    match err {
                        KeyStoreError::DuplicateKeyError(msg) => HttpResponse::BadRequest()
                            .json(ErrorResponse::bad_request(&msg))
                            .into_future(),
                        KeyStoreError::UserDoesNotExistError(msg) => HttpResponse::BadRequest()
                            .json(ErrorResponse::bad_request(&msg))
                            .into_future(),
                        _ => HttpResponse::InternalServerError()
                            .json(ErrorResponse::internal_error())
                            .into_future(),
                    }
                }
            }
        }))
    })
}

/// Defines a REST endpoint for retrieving a key from the underlying storage
fn handle_get(
    rest_config: Arc<BiomeRestConfig>,
    key_store: Arc<dyn KeyStore<Key>>,
    secret_manager: Arc<dyn SecretManager>,
) -> HandlerFunction {
    Box::new(move |request, _| {
        let key_store = key_store.clone();
        let user_id = match request.match_info().get("user_id") {
            Some(id) => id.to_owned(),
            None => {
                error!("User ID is not in path request");
                return Box::new(
                    HttpResponse::InternalServerError()
                        .json(ErrorResponse::internal_error())
                        .into_future(),
                );
            }
        };

        match authorize_user(&request, &user_id, &secret_manager, &rest_config) {
            AuthorizationResult::Authorized => (),
            AuthorizationResult::Unauthorized(msg) => {
                return Box::new(
                    HttpResponse::Unauthorized()
                        .json(ErrorResponse::unauthorized(&msg))
                        .into_future(),
                )
            }
            AuthorizationResult::Failed => {
                return Box::new(
                    HttpResponse::InternalServerError()
                        .json(ErrorResponse::internal_error())
                        .into_future(),
                );
            }
        }

        match key_store.list_keys(Some(&user_id)) {
            Ok(keys) => Box::new(
                HttpResponse::Ok()
                    .json(json!({ "data": keys.iter().map(ResponseKey::from).collect::<Vec<ResponseKey>>() }))
                    .into_future(),
            ),
            Err(err) => {
                debug!("Failed to fetch keys {}", err);
                Box::new(
                    HttpResponse::InternalServerError()
                        .json(ErrorResponse::internal_error())
                        .into_future(),
                )
            }
        }
    })
}

/// Defines a REST endpoint for updating a key in the underlying storage
fn handle_patch(
    rest_config: Arc<BiomeRestConfig>,
    key_store: Arc<dyn KeyStore<Key>>,
    secret_manager: Arc<dyn SecretManager>,
) -> HandlerFunction {
    Box::new(move |request, payload| {
        let key_store = key_store.clone();
        let user_id = match request.match_info().get("user_id") {
            Some(id) => id.to_owned(),
            None => {
                error!("User ID is not in path request");
                return Box::new(
                    HttpResponse::InternalServerError()
                        .json(ErrorResponse::internal_error())
                        .into_future(),
                );
            }
        };

        match authorize_user(&request, &user_id, &secret_manager, &rest_config) {
            AuthorizationResult::Authorized => (),
            AuthorizationResult::Unauthorized(msg) => {
                return Box::new(
                    HttpResponse::Unauthorized()
                        .json(ErrorResponse::unauthorized(&msg))
                        .into_future(),
                )
            }
            AuthorizationResult::Failed => {
                return Box::new(
                    HttpResponse::InternalServerError()
                        .json(ErrorResponse::internal_error())
                        .into_future(),
                );
            }
        }

        Box::new(into_bytes(payload).and_then(move |bytes| {
            let updated_key = match serde_json::from_slice::<UpdatedKey>(&bytes) {
                Ok(val) => val,
                Err(err) => {
                    debug!("Error parsing payload {}", err);
                    return HttpResponse::BadRequest()
                        .json(ErrorResponse::bad_request(&format!(
                            "Failed to parse payload: {}",
                            err
                        )))
                        .into_future();
                }
            };

            match key_store.update_key(
                &updated_key.public_key,
                &user_id,
                &updated_key.new_display_name,
            ) {
                Ok(()) => HttpResponse::Ok()
                    .json(json!({ "message": "Key updated successfully" }))
                    .into_future(),
                Err(err) => {
                    debug!("Failed to update key {}", err);
                    match err {
                        KeyStoreError::NotFoundError(msg) => HttpResponse::NotFound()
                            .json(ErrorResponse::not_found(&msg))
                            .into_future(),
                        _ => HttpResponse::InternalServerError()
                            .json(ErrorResponse::internal_error())
                            .into_future(),
                    }
                }
            }
        }))
    })
}
