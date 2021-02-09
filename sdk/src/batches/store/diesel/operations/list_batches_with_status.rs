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
use crate::{
    batches::store::{diesel::BatchModel, BatchList},
    paging::Paging,
};
use diesel::prelude::*;

pub(in crate::batches::store::diesel) trait ListBatchesWithStatusOperation {
    fn list_batches_with_status(
        &self,
        status: &str,
        offset: i64,
        limit: i64,
    ) -> Result<BatchList, BatchStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> ListBatchesWithStatusOperation for BatchStoreOperations<'a, diesel::pg::PgConnection> {
    fn list_batches_with_status(
        &self,
        status: &str,
        offset: i64,
        limit: i64,
    ) -> Result<BatchList, BatchStoreError> {
        let batches = batches::table
            .select(batches::all_columns)
            .filter(batches::status.eq(status))
            .offset(offset)
            .limit(limit)
            .load::<BatchModel>(self.conn)
            .map(|models| models.into_iter().map(|model| model.into()).collect())
            .map_err(|err| {
                BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

        let total = batches::table
            .select(batches::all_columns)
            .filter(batches::status.eq(status))
            .count()
            .get_result(self.conn)
            .map_err(|err| {
                BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

        Ok(BatchList::new(batches, Paging::new(offset, limit, total)))
    }
}

#[cfg(feature = "sqlite")]
impl<'a> ListBatchesWithStatusOperation
    for BatchStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn list_batches_with_status(
        &self,
        status: &str,
        offset: i64,
        limit: i64,
    ) -> Result<BatchList, BatchStoreError> {
        let batches = batches::table
            .select(batches::all_columns)
            .filter(batches::status.eq(status))
            .offset(offset)
            .limit(limit)
            .load::<BatchModel>(self.conn)
            .map(|models| models.into_iter().map(|model| model.into()).collect())
            .map_err(|err| {
                BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

        let total = batches::table
            .select(batches::all_columns)
            .filter(batches::status.eq(status))
            .count()
            .get_result(self.conn)
            .map_err(|err| {
                BatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

        Ok(BatchList::new(batches, Paging::new(offset, limit, total)))
    }
}
