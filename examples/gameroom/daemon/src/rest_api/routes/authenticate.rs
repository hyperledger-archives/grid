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
use bcrypt::{hash, verify};
use futures::Future;
use gameroom_database::{helpers, models::GameroomUser, ConnectionPool};
use serde::{Deserialize, Serialize};

use crate::rest_api::RestApiResponseError;

// Default cost is 12. This value should not be used in a production
// environment.
const HASH_COST: u32 = 4;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponseData {
    email: String,
    public_key: String,
    encrypted_private_key: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthData {
    pub email: String,
    pub hashed_password: String,
}

pub fn login(
    auth_data: web::Json<AuthData>,
    pool: web::Data<ConnectionPool>,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(
        web::block(move || authenticate_user(pool, auth_data.into_inner())).then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok().json(user)),
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
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    Box::new(
        web::block(move || create_user(pool, new_user.into_inner())).then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok().json(user)),
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
) -> Result<AuthResponseData, RestApiResponseError> {
    if helpers::fetch_user_by_email(&*pool.get()?, &new_user.email)?.is_some() {
        return Err(RestApiResponseError::BadRequest(
            "User already exists".to_string(),
        ));
    } else {
        let gameroom_user = GameroomUser {
            public_key: new_user.public_key.to_string(),
            encrypted_private_key: new_user.encrypted_private_key.to_string(),
            email: new_user.email.to_string(),
            hashed_password: hash_password(&new_user.hashed_password)?,
        };
        helpers::insert_user(&*pool.get()?, gameroom_user)?
    }
    Ok(AuthResponseData {
        email: new_user.email,
        public_key: new_user.public_key,
        encrypted_private_key: new_user.encrypted_private_key,
    })
}

fn hash_password(password: &str) -> Result<String, RestApiResponseError> {
    hash(password, HASH_COST).map_err(RestApiResponseError::from)
}

fn authenticate_user(
    pool: web::Data<ConnectionPool>,
    auth_data: AuthData,
) -> Result<AuthResponseData, RestApiResponseError> {
    if let Some(user) = helpers::fetch_user_by_email(&*pool.get()?, &auth_data.email)? {
        if verify(&auth_data.hashed_password, &user.hashed_password)? {
            return Ok(AuthResponseData {
                email: user.email.to_string(),
                public_key: user.public_key.to_string(),
                encrypted_private_key: user.encrypted_private_key.to_string(),
            });
        }
    }
    Err(RestApiResponseError::Unauthorized)
}
