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

use std::time::{Duration, SystemTime};

use gameroom_database::models::GameroomNotification;

use super::Paging;

#[derive(Debug, Serialize)]
struct NotificationListResponse {
    data: Vec<ApiNotification>,
    paging: Paging,
}

#[derive(Debug, Serialize)]
struct ApiNotification {
    id: i64,
    notification_type: String,
    org: String,
    target: String,
    timestamp: u64,
    read: bool,
}

impl ApiNotification {
    fn from(db_notification: GameroomNotification) -> ApiNotification {
        ApiNotification {
            id: db_notification.id,
            notification_type: db_notification.notification_type.to_string(),
            org: db_notification.requester.to_string(),
            target: db_notification.target.to_string(),
            timestamp: db_notification
                .created_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_else(|_| Duration::new(0, 0))
                .as_secs(),
            read: db_notification.read,
        }
    }
}
