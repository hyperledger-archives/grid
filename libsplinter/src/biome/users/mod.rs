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

//! Defines a basic representation of a user and provides an API to manage users.
//!
//! Users are central entity in the biome module. They represent a real person who uses a splinter
//! application.

pub mod database;

use database::models::UserModel;

/// Represents a user of a splinter application
pub struct SplinterUser {
    id: String,
}

impl SplinterUser {
    /// Creates a new SplinterUser
    ///
    /// # Arguments
    ///
    /// * `user_id`: unique identifier for the user being created
    ///
    pub fn new(user_id: &str) -> Self {
        SplinterUser {
            id: user_id.to_string(),
        }
    }

    /// Returns the user's id.
    pub fn id(&self) -> String {
        self.id.to_string()
    }
}

impl From<UserModel> for SplinterUser {
    fn from(user: UserModel) -> Self {
        SplinterUser { id: user.id }
    }
}
impl Into<UserModel> for SplinterUser {
    fn into(self) -> UserModel {
        UserModel { id: self.id }
    }
}
