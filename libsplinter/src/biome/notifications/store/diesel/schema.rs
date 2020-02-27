/*
 * Copyright 2018-2020 Cargill Incorporated
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
table! {
    notifications (id) {
        id -> Text,
        payload_title -> Text,
        payload_body -> Text,
        created -> Timestamp,
        recipients -> Array<Text>,
    }
}

table! {
    user_notifications (notification_id) {
        notification_id -> Text,
        user_id -> Text,
        unread -> Bool,
    }
}

table! {
    notification_properties (id) {
        id -> Int8,
        notification_id -> Text,
        property -> Text,
        property_value -> Text,
    }
}

joinable!(user_notifications -> notifications (notification_id));
joinable!(notification_properties -> notifications (notification_id));

allow_tables_to_appear_in_same_query!(notifications, user_notifications, notification_properties);
