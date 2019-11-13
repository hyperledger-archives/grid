/*
 * Copyright 2019 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use std::time::SystemTime;

use super::schema::{notification_properties, notifications, user_notifications};

#[derive(Insertable, Queryable)]
#[table_name = "notifications"]
pub struct Notification {
    pub id: String,
    pub payload_title: String,
    pub payload_body: String,
    pub created: SystemTime,
    pub recipients: Vec<String>,
}

#[derive(Insertable, Queryable)]
#[table_name = "user_notifications"]
pub struct UserNotification {
    pub notification_id: String,
    pub user_id: String,
    pub unread: bool,
}

#[derive(Insertable, Queryable)]
#[table_name = "notification_properties"]
pub struct NotificationProperty {
    pub id: i64,
    pub notification_id: String,
    pub property: String,
    pub property_value: String,
}
