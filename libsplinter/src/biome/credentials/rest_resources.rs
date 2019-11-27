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

use std::sync::Arc;
use uuid::Uuid;

use crate::actix_web::HttpResponse;
use crate::futures::{Future, IntoFuture};
use crate::rest_api::{into_bytes, ErrorResponse, Method, Resource};

use super::super::users::{user_store::SplinterUserStore, SplinterUser, UserStore};
use super::{
    credentials_store::SplinterCredentialsStore, CredentialsStore, CredentialsStoreError,
    UserCredentialsBuilder,
};

#[derive(Deserialize)]
struct UsernamePassword {
    username: String,
    password: String,
}

/// Defines a REST endpoint to add a user and credentials to the database
pub fn make_register_route(
    credentials_store: Arc<SplinterCredentialsStore>,
    user_store: Arc<SplinterUserStore>,
) -> Resource {
    Resource::build("/biome/register").add_method(Method::Post, move |_, payload| {
        let credentials_store = credentials_store.clone();
        let user_store = user_store.clone();
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
                        .with_password(&username_password.password)
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
