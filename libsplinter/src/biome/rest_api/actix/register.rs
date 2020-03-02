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
use uuid::Uuid;

use crate::actix_web::HttpResponse;
use crate::futures::{Future, IntoFuture};
use crate::protocol;
use crate::rest_api::{into_bytes, ErrorResponse, Method, ProtocolVersionRangeGuard, Resource};

use crate::biome::credentials::store::{
    diesel::SplinterCredentialsStore, CredentialsStore, CredentialsStoreError,
    UserCredentialsBuilder,
};
use crate::biome::rest_api::BiomeRestConfig;
use crate::biome::user::store::{diesel::SplinterUserStore, SplinterUser, UserStore};

use super::super::resources::credentials::UsernamePassword;

/// Defines a REST endpoint to add a user and credentials to the database
///
/// The payload should be in the JSON format:
///   {
///       "username": <username of new user>
///       "hashed_password": <hash of the password the user will use to log in>
///   }
pub fn make_register_route(
    credentials_store: Arc<SplinterCredentialsStore>,
    user_store: Arc<SplinterUserStore>,
    rest_config: Arc<BiomeRestConfig>,
) -> Resource {
    Resource::build("/biome/register")
        .add_request_guard(ProtocolVersionRangeGuard::new(
            protocol::BIOME_REGISTER_PROTOCOL_MIN,
            protocol::BIOME_PROTOCOL_VERSION,
        ))
        .add_method(Method::Post, move |_, payload| {
            let credentials_store = credentials_store.clone();
            let user_store = user_store.clone();
            let rest_config = rest_config.clone();
            Box::new(into_bytes(payload).and_then(move |bytes| {
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
                let user_id = Uuid::new_v4().to_string();
                let splinter_user = SplinterUser::new(&user_id);
                match user_store.add_user(splinter_user) {
                    Ok(()) => {
                        let credentials_builder: UserCredentialsBuilder = Default::default();
                        let credentials = match credentials_builder
                            .with_user_id(&user_id)
                            .with_username(&username_password.username)
                            .with_password(&username_password.hashed_password)
                            .with_password_encryption_cost(rest_config.password_encryption_cost())
                            .build()
                        {
                            Ok(credential) => credential,
                            Err(err) => {
                                debug!("Failed to create credentials {}", err);
                                return HttpResponse::InternalServerError()
                                    .json(ErrorResponse::internal_error())
                                    .into_future();
                            }
                        };

                        match credentials_store.add_credentials(credentials) {
                            Ok(()) => HttpResponse::Ok()
                                .json(json!({ "message": "User created successfully" }))
                                .into_future(),
                            Err(err) => {
                                debug!("Failed to add new credentials to database {}", err);
                                match err {
                                    CredentialsStoreError::DuplicateError(err) => {
                                        HttpResponse::BadRequest()
                                            .json(ErrorResponse::bad_request(&format!(
                                                "Failed to create user: {}",
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
                        debug!("Failed to add new user to database {}", err);
                        HttpResponse::InternalServerError()
                            .json(ErrorResponse::internal_error())
                            .into_future()
                    }
                }
            }))
        })
}
