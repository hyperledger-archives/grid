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

use super::schema::*;
use super::SplinterUser;

#[derive(Insertable, Queryable, Identifiable, PartialEq, Debug)]
#[table_name = "splinter_user"]
#[primary_key(id)]
pub struct UserModel {
    pub id: String,
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
