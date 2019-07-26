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

use actix_web::web;
use bcrypt::verify;
use gameroom_database::{helpers, ConnectionPool};
use serde::Deserialize;

use crate::rest_api::RestApiResponseError;

#[derive(Debug, Deserialize)]
pub struct AuthData {
    pub email: String,
    pub hashed_password: String,
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
