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

//! Defines a basic API to register and authenticate a SplinterUser using a username and a password.
//! Not recommended for use in production.

pub mod database;
mod error;
use bcrypt::{hash, DEFAULT_COST};

use database::models::{NewUserCredentialsModel, UserCredentialsModel};

pub(in crate::biome) use error::UserCredentialsBuilderError;

/// Represents crendentials used to authenticate a user
pub struct UserCredentials {
    user_id: String,
    username: String,
    password: String,
}

/// Builder for UsersCredential. It hashes the password upon build.
#[derive(Default)]
pub struct UserCredentialsBuilder {
    user_id: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

impl UserCredentialsBuilder {
    /// Sets the user_id for the credentials belong to
    ///
    /// # Arguments
    ///
    /// * `user_id`: unique identifier for the user the credentials belong to
    ///
    pub fn with_user_id(mut self, user_id: &str) -> UserCredentialsBuilder {
        self.user_id = Some(user_id.to_owned());
        self
    }

    /// Sets the username for the credentials
    ///
    /// # Arguments
    ///
    /// * `username`: username that will be used to authenticate the user
    ///
    pub fn with_username(mut self, username: &str) -> UserCredentialsBuilder {
        self.username = Some(username.to_owned());
        self
    }

    // Sets the password for the credentials
    ///
    /// # Arguments
    ///
    /// * `password`: password that will be used to authenticate the user
    ///
    pub fn with_password(mut self, password: &str) -> UserCredentialsBuilder {
        self.password = Some(password.to_owned());
        self
    }

    /// Consumes the builder, hashes the password and returns UserCredentials with the hashed
    /// password
    pub fn build(self) -> Result<UserCredentials, UserCredentialsBuilderError> {
        let user_id = self.user_id.ok_or_else(|| {
            UserCredentialsBuilderError::MissingRequiredField("Missing user_id".to_string())
        })?;
        let username = self.username.ok_or_else(|| {
            UserCredentialsBuilderError::MissingRequiredField("Missing user_id".to_string())
        })?;
        let hashed_password = hash(
            self.password.ok_or_else(|| {
                UserCredentialsBuilderError::MissingRequiredField("Missing password".to_string())
            })?,
            DEFAULT_COST,
        )?;
        Ok(UserCredentials {
            user_id,
            username,
            password: hashed_password,
        })
    }
}

impl From<UserCredentialsModel> for UserCredentials {
    fn from(user_credentials: UserCredentialsModel) -> Self {
        Self {
            user_id: user_credentials.user_id,
            username: user_credentials.username,
            password: user_credentials.password,
        }
    }
}

impl Into<NewUserCredentialsModel> for UserCredentials {
    fn into(self) -> NewUserCredentialsModel {
        NewUserCredentialsModel {
            user_id: self.user_id,
            username: self.username,
            password: self.password,
        }
    }
}
