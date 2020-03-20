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

#[cfg(feature = "biome-credentials")]
table! {
    user_credentials {
        id -> Int8,
        user_id -> Text,
        username -> Text,
        password -> Text,
    }
}

#[cfg(feature = "biome-key-management")]
table! {
    keys (public_key, user_id) {
        public_key -> Text,
        encrypted_private_key -> Text,
        user_id -> Text,
        display_name -> Text,
    }
}

#[cfg(feature = "biome-notifications")]
table! {
    notifications (id) {
        id -> Text,
        payload_title -> Text,
        payload_body -> Text,
        created -> Timestamp,
        recipients -> Array<Text>,
    }
}

#[cfg(feature = "biome-notifications")]
table! {
    user_notifications (notification_id) {
        notification_id -> Text,
        user_id -> Text,
        unread -> Bool,
    }
}

#[cfg(feature = "biome-notifications")]
table! {
    notification_properties (id) {
        id -> Int8,
        notification_id -> Text,
        property -> Text,
        property_value -> Text,
    }
}

#[cfg(feature = "biome-notifications")]
joinable!(user_notifications -> notifications (notification_id));
#[cfg(feature = "biome-notifications")]
joinable!(notification_properties -> notifications (notification_id));

#[cfg(feature = "biome-notifications")]
allow_tables_to_appear_in_same_query!(notifications, user_notifications, notification_properties);
