// Copyright 2022 Cargill Incorporated
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

use super::BatchTrackingStoreOperations;

use crate::batch_tracking::store::diesel::schema::batches;

use crate::batch_tracking::store::BatchTrackingStoreError;
use diesel::{delete, prelude::*};

pub(in crate::batch_tracking::store::diesel) trait BatchTrackingCleanStaleRecordsOperation {
    fn clean_stale_records(&self, submitted_by: i64) -> Result<(), BatchTrackingStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> BatchTrackingCleanStaleRecordsOperation
    for BatchTrackingStoreOperations<'a, diesel::pg::PgConnection>
{
    fn clean_stale_records(&self, submitted_by: i64) -> Result<(), BatchTrackingStoreError> {
        self.conn.transaction::<_, BatchTrackingStoreError, _>(|| {
            delete(batches::table.filter(batches::created_at.lt(&submitted_by)))
                .execute(self.conn)?;

            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> BatchTrackingCleanStaleRecordsOperation
    for BatchTrackingStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn clean_stale_records(&self, submitted_by: i64) -> Result<(), BatchTrackingStoreError> {
        self.conn.transaction::<_, BatchTrackingStoreError, _>(|| {
            delete(batches::table.filter(batches::created_at.lt(&submitted_by)))
                .execute(self.conn)?;

            Ok(())
        })
    }
}
