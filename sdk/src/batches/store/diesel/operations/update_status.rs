// Copyright 2018-2021 Cargill Incorporated
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

use super::BatchStoreOperations;
use crate::batches::store::{diesel::schema::batches, BatchStoreError};

use crate::error::InternalError;
use diesel::{dsl::update, prelude::*};

pub(in crate::batches::store::diesel) trait UpdateStatusOperation {
    fn update_status(&self, id: &str, status: &str) -> Result<(), BatchStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> UpdateStatusOperation for BatchStoreOperations<'a, diesel::pg::PgConnection> {
    fn update_status(&self, id: &str, status: &str) -> Result<(), BatchStoreError> {
        update(batches::table)
            .filter(batches::id.eq(id))
            .set(batches::status.eq(status))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|err| {
                BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> UpdateStatusOperation for BatchStoreOperations<'a, diesel::sqlite::SqliteConnection> {
    fn update_status(&self, id: &str, status: &str) -> Result<(), BatchStoreError> {
        update(batches::table)
            .filter(batches::id.eq(id))
            .set(batches::status.eq(status))
            .execute(self.conn)
            .map(|_| ())
            .map_err(|err| {
                BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })
    }
}
