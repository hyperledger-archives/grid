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

#[cfg(feature = "biome-notifications")]
use std::time::SystemTime;

use super::schema::*;
use crate::biome::user::store::SplinterUser;

#[cfg(feature = "biome-credentials")]
#[derive(Queryable, Identifiable, Associations, PartialEq, Debug)]
#[table_name = "user_credentials"]
#[belongs_to(UserModel, foreign_key = "user_id")]
pub struct UserCredentialsModel {
    pub id: i64,
    pub user_id: String,
    pub username: String,
    pub password: String,
}

#[cfg(feature = "biome-credentials")]
#[derive(Insertable, PartialEq, Debug)]
#[table_name = "user_credentials"]
pub struct NewUserCredentialsModel {
    pub user_id: String,
    pub username: String,
    pub password: String,
}

#[cfg(feature = "biome-key-management")]
#[derive(Insertable, Queryable, Identifiable, PartialEq, Debug)]
#[table_name = "keys"]
#[primary_key(public_key, user_id)]
pub struct KeyModel {
    pub public_key: String,
    pub encrypted_private_key: String,
    pub user_id: String,
    pub display_name: String,
}

#[cfg(feature = "biome-notifications")]
#[derive(Insertable, Queryable)]
#[table_name = "notifications"]
pub struct Notification {
    pub id: String,
    pub payload_title: String,
    pub payload_body: String,
    pub created: SystemTime,
    pub recipients: Vec<String>,
}

#[cfg(feature = "biome-notifications")]
#[derive(Insertable, Queryable)]
#[table_name = "user_notifications"]
pub struct UserNotification {
    pub notification_id: String,
    pub user_id: String,
    pub unread: bool,
}

#[cfg(feature = "biome-notifications")]
#[derive(Insertable, Queryable)]
#[table_name = "notification_properties"]
pub struct NotificationProperty {
    pub id: i64,
    pub notification_id: String,
    pub property: String,
    pub property_value: String,
}

#[derive(Insertable, Queryable, Identifiable, PartialEq, Debug)]
#[table_name = "splinter_user"]
#[primary_key(id)]
pub struct UserModel {
    pub id: String,
}

impl From<SplinterUser> for UserModel {
    fn from(user: SplinterUser) -> Self {
        UserModel { id: user.id() }
    }
}
