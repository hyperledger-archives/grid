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

use actix_web::{error, web, Error, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use futures::Future;
use gameroom_database::{helpers, models::GameroomUser, ConnectionPool};
use serde::Deserialize;

use crate::rest_api::RestApiResponseError;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthData {
    pub email: String,
    pub hashed_password: String,
}

pub fn login(
    auth_data: web::Json<AuthData>,
    pool: web::Data<ConnectionPool>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(
        web::block(move || authenticate_user(pool, auth_data.into_inner())).then(|res| match res {
            Ok(()) => Ok(HttpResponse::Ok().finish()),
            Err(err) => match err {
                error::BlockingError::Error(_) => Ok(HttpResponse::Unauthorized().into()),
                error::BlockingError::Canceled => Ok(HttpResponse::InternalServerError().into()),
            },
        }),
    )
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserCreate {
    pub email: String,
    pub hashed_password: String,
    pub encrypted_private_key: String,
    pub public_key: String,
}

pub fn register(
    new_user: web::Json<UserCreate>,
    pool: web::Data<ConnectionPool>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    Box::new(
        web::block(move || create_user(pool, new_user.into_inner())).then(|res| match res {
            Ok(()) => Ok(HttpResponse::Ok().finish()),
            Err(err) => match err {
                error::BlockingError::Error(err) => {
                    Ok(HttpResponse::BadRequest().json(err.to_string()))
                }
                error::BlockingError::Canceled => Ok(HttpResponse::InternalServerError().into()),
            },
        }),
    )
}

fn create_user(
    pool: web::Data<ConnectionPool>,
    new_user: UserCreate,
) -> Result<(), RestApiResponseError> {
    if helpers::fetch_user_by_email(&*pool.get()?, &new_user.email)?.is_some() {
        return Err(RestApiResponseError::BadRequest(
            "User already exists".to_string(),
        ));
    } else {
        let gameroom_user = GameroomUser {
            public_key: new_user.public_key,
            encrypted_private_key: new_user.encrypted_private_key,
            email: new_user.email,
            hashed_password: hash_password(&new_user.hashed_password)?,
        };
        helpers::insert_user(&*pool.get()?, gameroom_user)?
    }
    Ok(())
}

fn hash_password(password: &str) -> Result<String, RestApiResponseError> {
    hash(password, DEFAULT_COST).map_err(RestApiResponseError::from)
}

fn authenticate_user(
    pool: web::Data<ConnectionPool>,
    auth_data: AuthData,
) -> Result<(), RestApiResponseError> {
    if let Some(user) = helpers::fetch_user_by_email(&*pool.get()?, &auth_data.email)? {
        if verify(&auth_data.hashed_password, &user.hashed_password)? {
            return Ok(());
        }
    }
    Err(RestApiResponseError::Unauthorized)
}
