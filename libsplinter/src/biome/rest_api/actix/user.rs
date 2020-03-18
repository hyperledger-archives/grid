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

use crate::actix_web::{web::Payload, Error, HttpRequest, HttpResponse};
use crate::futures::{Future, IntoFuture};
use crate::protocol;
use crate::rest_api::{into_bytes, ErrorResponse, Method, ProtocolVersionRangeGuard, Resource};

use crate::biome::credentials::store::{
    diesel::SplinterCredentialsStore, CredentialsStore, CredentialsStoreError,
};
use crate::biome::rest_api::BiomeRestConfig;
use crate::biome::user::store::{
    diesel::SplinterUserStore, SplinterUser, UserStore, UserStoreError,
};
use crate::rest_api::secrets::SecretManager;

use super::super::resources::authorize::AuthorizationResult;
use super::super::resources::credentials::UsernamePassword;
use super::authorize::authorize_user;

/// Defines a REST endpoint to list users from the db
pub fn make_list_route(credentials_store: Arc<SplinterCredentialsStore>) -> Resource {
    Resource::build("/biome/users")
        .add_request_guard(ProtocolVersionRangeGuard::new(
            protocol::BIOME_LIST_USERS_PROTOCOL_MIN,
            protocol::BIOME_PROTOCOL_VERSION,
        ))
        .add_method(Method::Get, move |_, _| {
            let credentials_store = credentials_store.clone();
            Box::new(match credentials_store.get_usernames() {
                Ok(users) => HttpResponse::Ok().json(users).into_future(),
                Err(err) => {
                    debug!("Failed to get users from the database {}", err);
                    HttpResponse::InternalServerError()
                        .json(ErrorResponse::internal_error())
                        .into_future()
                }
            })
        })
}

/// Defines the `/biome/users/{id}` REST resource for managing users
pub fn make_user_routes(
    rest_config: Arc<BiomeRestConfig>,
    secret_manager: Arc<dyn SecretManager>,
    credentials_store: Arc<SplinterCredentialsStore>,
    user_store: SplinterUserStore,
) -> Resource {
    let credentials_store_modify = credentials_store.clone();
    let credentials_store_fetch = credentials_store;
    let user_store_modify = user_store.clone();
    let user_store_delete = user_store;
    Resource::build("/biome/users/{id}")
        .add_request_guard(ProtocolVersionRangeGuard::new(
            protocol::BIOME_USER_PROTOCOL_MIN,
            protocol::BIOME_PROTOCOL_VERSION,
        ))
        .add_method(Method::Put, move |request, payload| {
            add_modify_user_method(
                request,
                payload,
                credentials_store_modify.clone(),
                user_store_modify.clone(),
            )
        })
        .add_method(Method::Get, move |request, _| {
            add_fetch_user_method(request, credentials_store_fetch.clone())
        })
        .add_method(Method::Delete, move |request, _| {
            add_delete_user_method(
                request,
                rest_config.clone(),
                secret_manager.clone(),
                user_store_delete.clone(),
            )
        })
}

/// Defines a REST endpoint to fetch a user from the database
/// returns the user's ID and username
fn add_fetch_user_method(
    request: HttpRequest,
    credentials_store: Arc<SplinterCredentialsStore>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let user_id = if let Some(t) = request.match_info().get("id") {
        t.to_string()
    } else {
        return Box::new(
            HttpResponse::BadRequest()
                .json(ErrorResponse::bad_request(
                    &"Failed to process request: no user id".to_string(),
                ))
                .into_future(),
        );
    };
    Box::new(match credentials_store.fetch_username_by_id(&user_id) {
        Ok(user) => HttpResponse::Ok().json(user).into_future(),
        Err(err) => {
            debug!("Failed to get user from the database {}", err);
            match err {
                CredentialsStoreError::NotFoundError(_) => HttpResponse::NotFound()
                    .json(ErrorResponse::not_found(&format!(
                        "User ID not found: {}",
                        &user_id
                    )))
                    .into_future(),
                _ => HttpResponse::InternalServerError()
                    .json(ErrorResponse::internal_error())
                    .into_future(),
            }
        }
    })
}

/// Defines a REST endpoint to edit a user's credentials in the database
///
/// The payload should be in the JSON format:
///   {
///       "username": <existing username of the user>
///       "hashed_password": <hash of the user's existing password>
///       "new_password": OPTIONAL <hash of the user's updated password>
///   }
fn add_modify_user_method(
    request: HttpRequest,
    payload: Payload,
    credentials_store: Arc<SplinterCredentialsStore>,
    mut user_store: SplinterUserStore,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let user_id = if let Some(t) = request.match_info().get("id") {
        t.to_string()
    } else {
        return Box::new(
            HttpResponse::BadRequest()
                .json(ErrorResponse::bad_request(
                    &"Failed to parse payload: no user id".to_string(),
                ))
                .into_future(),
        );
    };
    Box::new(into_bytes(payload).and_then(move |bytes| {
        let body = match serde_json::from_slice::<serde_json::Value>(&bytes) {
            Ok(val) => val,
            Err(err) => {
                debug!("Error parsing request body {}", err);
                return HttpResponse::BadRequest()
                    .json(ErrorResponse::bad_request(&format!(
                        "Failed to parse payload body: {}",
                        err
                    )))
                    .into_future();
            }
        };
        let username_password = match serde_json::from_slice::<UsernamePassword>(&bytes) {
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

        let credentials =
            match credentials_store.fetch_credential_by_username(&username_password.username) {
                Ok(credentials) => credentials,
                Err(err) => {
                    debug!("Failed to fetch credentials {}", err);
                    match err {
                        CredentialsStoreError::NotFoundError(_) => {
                            return HttpResponse::NotFound()
                                .json(ErrorResponse::not_found(&format!(
                                    "Username not found: {}",
                                    username_password.username
                                )))
                                .into_future();
                        }
                        _ => {
                            return HttpResponse::InternalServerError()
                                .json(ErrorResponse::internal_error())
                                .into_future()
                        }
                    }
                }
            };
        let splinter_user = SplinterUser::new(&user_id);
        match credentials.verify_password(&username_password.hashed_password) {
            Ok(is_valid) => {
                if is_valid {
                    let new_password = match body.get("new_password") {
                        Some(val) => match val.as_str() {
                            Some(val) => val,
                            None => &username_password.hashed_password,
                        },
                        None => &username_password.hashed_password,
                    };

                    match user_store.update_user(splinter_user) {
                        Ok(()) => {
                            match credentials_store.update_credentials(
                                &user_id,
                                &username_password.username,
                                &new_password,
                            ) {
                                Ok(()) => HttpResponse::Ok()
                                    .json(json!({ "message": "User updated successfully" }))
                                    .into_future(),
                                Err(err) => {
                                    debug!("Failed to update credentials in database {}", err);
                                    match err {
                                        CredentialsStoreError::DuplicateError(err) => {
                                            HttpResponse::BadRequest()
                                                .json(ErrorResponse::bad_request(&format!(
                                                    "Failed to update user: {}",
                                                    err
                                                )))
                                                .into_future()
                                        }
                                        _ => HttpResponse::InternalServerError()
                                            .json(ErrorResponse::internal_error())
                                            .into_future(),
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            debug!("Failed to update user in database {}", err);
                            HttpResponse::InternalServerError()
                                .json(ErrorResponse::internal_error())
                                .into_future()
                        }
                    }
                } else {
                    HttpResponse::BadRequest()
                        .json(ErrorResponse::bad_request("Invalid password"))
                        .into_future()
                }
            }
            Err(err) => {
                debug!("Failed to verify password {}", err);
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::internal_error())
                    .into_future()
            }
        }
    }))
}

/// Defines a REST endpoint to delete a user from the database
fn add_delete_user_method(
    request: HttpRequest,
    rest_config: Arc<BiomeRestConfig>,
    secret_manager: Arc<dyn SecretManager>,
    mut user_store: SplinterUserStore,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let user_id = if let Some(t) = request.match_info().get("id") {
        t.to_string()
    } else {
        return Box::new(
            HttpResponse::BadRequest()
                .json(ErrorResponse::bad_request(
                    &"Failed to parse payload: no user id".to_string(),
                ))
                .into_future(),
        );
    };
    match authorize_user(&request, &user_id, &secret_manager, &rest_config) {
        AuthorizationResult::Authorized => match user_store.remove_user(&user_id) {
            Ok(()) => Box::new(
                HttpResponse::Ok()
                    .json(json!({ "message": "User deleted sucessfully" }))
                    .into_future(),
            ),
            Err(err) => match err {
                UserStoreError::NotFoundError(msg) => {
                    debug!("User not found: {}", msg);
                    Box::new(
                        HttpResponse::NotFound()
                            .json(ErrorResponse::not_found(&format!(
                                "User ID not found: {}",
                                &user_id
                            )))
                            .into_future(),
                    )
                }
                _ => {
                    error!("Failed to delete user in database {}", err);
                    Box::new(
                        HttpResponse::InternalServerError()
                            .json(ErrorResponse::internal_error())
                            .into_future(),
                    )
                }
            },
        },
        AuthorizationResult::Unauthorized(msg) => Box::new(
            HttpResponse::Unauthorized()
                .json(ErrorResponse::unauthorized(&msg))
                .into_future(),
        ),
        AuthorizationResult::Failed => {
            error!("Failed to authorize user");
            Box::new(
                HttpResponse::InternalServerError()
                    .json(ErrorResponse::internal_error())
                    .into_future(),
            )
        }
    }
}
