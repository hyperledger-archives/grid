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

//! Defines structures used in key management.

use crate::biome::key_management::Key;

#[derive(Deserialize)]
pub(crate) struct NewKey {
    pub public_key: String,
    pub encrypted_private_key: String,
    pub display_name: String,
}

#[derive(Deserialize)]
pub(crate) struct UpdatedKey {
    pub public_key: String,
    pub new_display_name: String,
}

#[derive(Serialize)]
pub(crate) struct ResponseKey<'a> {
    public_key: &'a str,
    user_id: &'a str,
    display_name: &'a str,
    encrypted_private_key: &'a str,
}

impl<'a> From<&'a Key> for ResponseKey<'a> {
    fn from(key: &'a Key) -> Self {
        ResponseKey {
            public_key: &key.public_key,
            user_id: &key.user_id,
            display_name: &key.display_name,
            encrypted_private_key: &key.encrypted_private_key,
        }
    }
}
