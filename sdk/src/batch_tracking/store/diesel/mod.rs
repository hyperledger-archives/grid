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

pub mod models;
mod operations;
pub(in crate) mod schema;

use diesel::connection::AnsiTransactionManager;
use diesel::r2d2::{ConnectionManager, Pool};

use super::{
    BatchStatus, BatchTrackingStore, BatchTrackingStoreError, InvalidTransaction, SubmissionError,
    TrackingBatch, TrackingBatchList, TrackingTransaction, TransactionReceipt, ValidTransaction,
};

use crate::error::ResourceTemporarilyUnavailableError;

use models::{NewBatchStatusModel, NewSubmissionModel, TransactionReceiptModel};
use operations::add_batches::BatchTrackingStoreAddBatchesOperation as _;
use operations::change_batch_to_submitted::BatchTrackingStoreChangeBatchToSubmittedOperation as _;
use operations::get_batch::BatchTrackingStoreGetBatchOperation as _;
use operations::get_batch_status::BatchTrackingStoreGetBatchStatusOperation as _;
use operations::BatchTrackingStoreOperations;

/// Manages batches in the database
#[derive(Clone)]
pub struct DieselBatchTrackingStore<C: diesel::Connection + 'static> {
    connection_pool: Pool<ConnectionManager<C>>,
}

impl<C: diesel::Connection> DieselBatchTrackingStore<C> {
    /// Creates a new DieselBatchTrackingStore
    ///
    /// # Arguments
    ///
    ///  * `connection_pool`: connection pool to the database
    #[allow(dead_code)]
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> Self {
        DieselBatchTrackingStore { connection_pool }
    }
}

#[cfg(feature = "postgres")]
impl BatchTrackingStore for DieselBatchTrackingStore<diesel::pg::PgConnection> {
    fn get_batch_status(
        &self,
        id: &str,
        service_id: &str,
    ) -> Result<Option<BatchStatus>, BatchTrackingStoreError> {
        BatchTrackingStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchTrackingStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_batch_status(id, service_id)
    }

    fn update_batch_status(
        &self,
        _id: String,
        _service_id: Option<&str>,
        _status: BatchStatus,
        _errors: Vec<SubmissionError>,
    ) -> Result<(), BatchTrackingStoreError> {
        unimplemented!();
    }

    fn add_batches(&self, batches: Vec<TrackingBatch>) -> Result<(), BatchTrackingStoreError> {
        BatchTrackingStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchTrackingStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_batches(batches)
    }

    fn change_batch_to_submitted(
        &self,
        batch_id: &str,
        service_id: &str,
        transaction_receipts: Vec<TransactionReceipt>,
        dlt_status: Option<&str>,
        submission_error: Option<SubmissionError>,
    ) -> Result<(), BatchTrackingStoreError> {
        let mut batch_status = None;

        if let Some(ds) = dlt_status {
            batch_status = Some(NewBatchStatusModel {
                batch_id: batch_id.to_string(),
                service_id: service_id.to_string(),
                dlt_status: ds.to_string(),
            });
        }

        let mut submission = NewSubmissionModel {
            batch_id: batch_id.to_string(),
            service_id: service_id.to_string(),
            error_type: None,
            error_message: None,
        };

        if let Some(s) = submission_error {
            submission = NewSubmissionModel {
                batch_id: batch_id.to_string(),
                service_id: service_id.to_string(),
                error_type: Some(s.error_type().to_string()),
                error_message: Some(s.error_message().to_string()),
            };
        }

        BatchTrackingStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchTrackingStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .change_batch_to_submitted(
            batch_id,
            service_id,
            transaction_receipts
                .iter()
                .map(|r| TransactionReceiptModel::from((r, service_id)))
                .collect(),
            batch_status,
            submission,
        )
    }

    fn get_batch(
        &self,
        id: &str,
        service_id: &str,
    ) -> Result<Option<TrackingBatch>, BatchTrackingStoreError> {
        BatchTrackingStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchTrackingStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_batch(id, service_id)
    }

    fn list_batches_by_status(
        &self,
        _status: BatchStatus,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn clean_stale_records(
        &self,
        _submitted_by: &str,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_unsubmitted_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_failed_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }
}

#[cfg(feature = "sqlite")]
impl BatchTrackingStore for DieselBatchTrackingStore<diesel::sqlite::SqliteConnection> {
    fn get_batch_status(
        &self,
        id: &str,
        service_id: &str,
    ) -> Result<Option<BatchStatus>, BatchTrackingStoreError> {
        BatchTrackingStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchTrackingStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_batch_status(id, service_id)
    }

    fn update_batch_status(
        &self,
        _id: String,
        _service_id: Option<&str>,
        _status: BatchStatus,
        _errors: Vec<SubmissionError>,
    ) -> Result<(), BatchTrackingStoreError> {
        unimplemented!();
    }

    fn add_batches(&self, batches: Vec<TrackingBatch>) -> Result<(), BatchTrackingStoreError> {
        BatchTrackingStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchTrackingStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_batches(batches)
    }

    fn change_batch_to_submitted(
        &self,
        batch_id: &str,
        service_id: &str,
        transaction_receipts: Vec<TransactionReceipt>,
        dlt_status: Option<&str>,
        submission_error: Option<SubmissionError>,
    ) -> Result<(), BatchTrackingStoreError> {
        let mut batch_status = None;

        if let Some(ds) = dlt_status {
            batch_status = Some(NewBatchStatusModel {
                batch_id: batch_id.to_string(),
                service_id: service_id.to_string(),
                dlt_status: ds.to_string(),
            });
        }

        let mut submission = NewSubmissionModel {
            batch_id: batch_id.to_string(),
            service_id: service_id.to_string(),
            error_type: None,
            error_message: None,
        };

        if let Some(s) = submission_error {
            submission = NewSubmissionModel {
                batch_id: batch_id.to_string(),
                service_id: service_id.to_string(),
                error_type: Some(s.error_type().to_string()),
                error_message: Some(s.error_message().to_string()),
            };
        }

        BatchTrackingStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchTrackingStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .change_batch_to_submitted(
            batch_id,
            service_id,
            transaction_receipts
                .iter()
                .map(|r| TransactionReceiptModel::from((r, service_id)))
                .collect(),
            batch_status,
            submission,
        )
    }

    fn get_batch(
        &self,
        id: &str,
        service_id: &str,
    ) -> Result<Option<TrackingBatch>, BatchTrackingStoreError> {
        BatchTrackingStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            BatchTrackingStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_batch(id, service_id)
    }

    fn list_batches_by_status(
        &self,
        _status: BatchStatus,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn clean_stale_records(
        &self,
        _submitted_by: &str,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_unsubmitted_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_failed_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }
}

pub struct DieselConnectionBatchTrackingStore<'a, C>
where
    C: diesel::Connection<TransactionManager = AnsiTransactionManager> + 'static,
    C::Backend: diesel::backend::UsesAnsiSavepointSyntax,
{
    connection: &'a C,
}

impl<'a, C> DieselConnectionBatchTrackingStore<'a, C>
where
    C: diesel::Connection<TransactionManager = AnsiTransactionManager> + 'static,
    C::Backend: diesel::backend::UsesAnsiSavepointSyntax,
{
    #[allow(dead_code)]
    pub fn new(connection: &'a C) -> Self {
        DieselConnectionBatchTrackingStore { connection }
    }
}

#[cfg(feature = "postgres")]
impl<'a> BatchTrackingStore for DieselConnectionBatchTrackingStore<'a, diesel::pg::PgConnection> {
    fn get_batch_status(
        &self,
        id: &str,
        service_id: &str,
    ) -> Result<Option<BatchStatus>, BatchTrackingStoreError> {
        BatchTrackingStoreOperations::new(self.connection).get_batch_status(id, service_id)
    }

    fn update_batch_status(
        &self,
        _id: String,
        _service_id: Option<&str>,
        _status: BatchStatus,
        _errors: Vec<SubmissionError>,
    ) -> Result<(), BatchTrackingStoreError> {
        unimplemented!();
    }

    fn add_batches(&self, batches: Vec<TrackingBatch>) -> Result<(), BatchTrackingStoreError> {
        BatchTrackingStoreOperations::new(self.connection).add_batches(batches)
    }

    fn change_batch_to_submitted(
        &self,
        batch_id: &str,
        service_id: &str,
        transaction_receipts: Vec<TransactionReceipt>,
        dlt_status: Option<&str>,
        submission_error: Option<SubmissionError>,
    ) -> Result<(), BatchTrackingStoreError> {
        let mut batch_status = None;

        if let Some(ds) = dlt_status {
            batch_status = Some(NewBatchStatusModel {
                batch_id: batch_id.to_string(),
                service_id: service_id.to_string(),
                dlt_status: ds.to_string(),
            });
        }

        let mut submission = NewSubmissionModel {
            batch_id: batch_id.to_string(),
            service_id: service_id.to_string(),
            error_type: None,
            error_message: None,
        };

        if let Some(s) = submission_error {
            submission = NewSubmissionModel {
                batch_id: batch_id.to_string(),
                service_id: service_id.to_string(),
                error_type: Some(s.error_type().to_string()),
                error_message: Some(s.error_message().to_string()),
            };
        }

        BatchTrackingStoreOperations::new(self.connection).change_batch_to_submitted(
            batch_id,
            service_id,
            transaction_receipts
                .iter()
                .map(|r| TransactionReceiptModel::from((r, service_id)))
                .collect(),
            batch_status,
            submission,
        )
    }

    fn get_batch(
        &self,
        id: &str,
        service_id: &str,
    ) -> Result<Option<TrackingBatch>, BatchTrackingStoreError> {
        BatchTrackingStoreOperations::new(self.connection).get_batch(id, service_id)
    }

    fn list_batches_by_status(
        &self,
        _status: BatchStatus,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn clean_stale_records(
        &self,
        _submitted_by: &str,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_unsubmitted_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_failed_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }
}

#[cfg(feature = "sqlite")]
impl<'a> BatchTrackingStore
    for DieselConnectionBatchTrackingStore<'a, diesel::sqlite::SqliteConnection>
{
    fn get_batch_status(
        &self,
        id: &str,
        service_id: &str,
    ) -> Result<Option<BatchStatus>, BatchTrackingStoreError> {
        BatchTrackingStoreOperations::new(self.connection).get_batch_status(id, service_id)
    }

    fn update_batch_status(
        &self,
        _id: String,
        _service_id: Option<&str>,
        _status: BatchStatus,
        _errors: Vec<SubmissionError>,
    ) -> Result<(), BatchTrackingStoreError> {
        unimplemented!();
    }

    fn add_batches(&self, batches: Vec<TrackingBatch>) -> Result<(), BatchTrackingStoreError> {
        BatchTrackingStoreOperations::new(self.connection).add_batches(batches)
    }

    fn change_batch_to_submitted(
        &self,
        batch_id: &str,
        service_id: &str,
        transaction_receipts: Vec<TransactionReceipt>,
        dlt_status: Option<&str>,
        submission_error: Option<SubmissionError>,
    ) -> Result<(), BatchTrackingStoreError> {
        let mut batch_status = None;

        if let Some(ds) = dlt_status {
            batch_status = Some(NewBatchStatusModel {
                batch_id: batch_id.to_string(),
                service_id: service_id.to_string(),
                dlt_status: ds.to_string(),
            });
        }

        let mut submission = NewSubmissionModel {
            batch_id: batch_id.to_string(),
            service_id: service_id.to_string(),
            error_type: None,
            error_message: None,
        };

        if let Some(s) = submission_error {
            submission = NewSubmissionModel {
                batch_id: batch_id.to_string(),
                service_id: service_id.to_string(),
                error_type: Some(s.error_type().to_string()),
                error_message: Some(s.error_message().to_string()),
            };
        }

        BatchTrackingStoreOperations::new(self.connection).change_batch_to_submitted(
            batch_id,
            service_id,
            transaction_receipts
                .iter()
                .map(|r| TransactionReceiptModel::from((r, service_id)))
                .collect(),
            batch_status,
            submission,
        )
    }

    fn get_batch(
        &self,
        id: &str,
        service_id: &str,
    ) -> Result<Option<TrackingBatch>, BatchTrackingStoreError> {
        BatchTrackingStoreOperations::new(self.connection).get_batch(id, service_id)
    }

    fn list_batches_by_status(
        &self,
        _status: BatchStatus,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn clean_stale_records(
        &self,
        _submitted_by: &str,
    ) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_unsubmitted_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }

    fn get_failed_batches(&self) -> Result<TrackingBatchList, BatchTrackingStoreError> {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cylinder::{secp256k1::Secp256k1Context, Context, Signer};
    use diesel::r2d2::{ConnectionManager, Pool};
    use diesel::sqlite::SqliteConnection;
    use transact::protocol::{
        batch::BatchBuilder,
        transaction::{HashMethod, TransactionBuilder},
    };

    use crate::batch_tracking::store::{
        SubmissionErrorBuilder, TrackingBatchBuilder, TransactionReceiptBuilder,
    };
    use crate::hex;
    use crate::migrations::run_sqlite_migrations;

    static FAMILY_NAME: &str = "test_family";
    static FAMILY_VERSION: &str = "0.1";
    static KEY1: &str = "111111111111111111111111111111111111111111111111111111111111111111";
    static KEY2: &str = "222222222222222222222222222222222222222222222222222222222222222222";
    static KEY3: &str = "333333333333333333333333333333333333333333333333333333333333333333";
    static KEY4: &str = "444444444444444444444444444444444444444444444444444444444444444444";
    static KEY5: &str = "555555555555555555555555555555555555555555555555555555555555555555";
    static KEY6: &str = "666666666666666666666666666666666666666666666666666666666666666666";
    static KEY7: &str = "777777777777777777777777777777777777777777777777777777777777777777";
    static NONCE: &str = "f9kdzz";
    static BYTES2: [u8; 4] = [0x05, 0x06, 0x07, 0x08];

    #[test]
    fn add_and_fetch() {
        let pool = create_connection_pool_and_migrate();

        let store = DieselBatchTrackingStore::new(pool);

        let signer = new_signer();

        let pair = TransactionBuilder::new()
            .with_batcher_public_key(hex::parse_hex(KEY1).unwrap())
            .with_dependencies(vec![KEY2.to_string(), KEY3.to_string()])
            .with_family_name(FAMILY_NAME.to_string())
            .with_family_version(FAMILY_VERSION.to_string())
            .with_inputs(vec![
                hex::parse_hex(KEY4).unwrap(),
                hex::parse_hex(&KEY5[0..4]).unwrap(),
            ])
            .with_nonce(NONCE.to_string().into_bytes())
            .with_outputs(vec![
                hex::parse_hex(KEY6).unwrap(),
                hex::parse_hex(&KEY7[0..4]).unwrap(),
            ])
            .with_payload_hash_method(HashMethod::Sha512)
            .with_payload(BYTES2.to_vec())
            .build(&*signer)
            .unwrap();

        let batch_1 = BatchBuilder::new()
            .with_transactions(vec![pair])
            .build(&*signer)
            .unwrap();

        let tracking_batch = TrackingBatchBuilder::default()
            .with_batch(batch_1)
            .with_service_id("TEST".to_string())
            .with_signer_public_key(KEY1.to_string())
            .with_submitted(false)
            .with_created_at(111111)
            .build()
            .unwrap();

        let id = tracking_batch.batch_header();

        store
            .add_batches(vec![tracking_batch.clone()])
            .expect("Failed to add batch");
        assert_eq!(
            store.get_batch(&id, "TEST").expect("Failed to get batch"),
            Some(tracking_batch)
        );
    }

    #[test]
    fn change_batch_to_submitted() {
        let pool = create_connection_pool_and_migrate();

        let store = DieselBatchTrackingStore::new(pool);

        let signer = new_signer();

        let pair = TransactionBuilder::new()
            .with_batcher_public_key(hex::parse_hex(KEY1).unwrap())
            .with_dependencies(vec![KEY2.to_string(), KEY3.to_string()])
            .with_family_name(FAMILY_NAME.to_string())
            .with_family_version(FAMILY_VERSION.to_string())
            .with_inputs(vec![
                hex::parse_hex(KEY4).unwrap(),
                hex::parse_hex(&KEY5[0..4]).unwrap(),
            ])
            .with_nonce(NONCE.to_string().into_bytes())
            .with_outputs(vec![
                hex::parse_hex(KEY6).unwrap(),
                hex::parse_hex(&KEY7[0..4]).unwrap(),
            ])
            .with_payload_hash_method(HashMethod::Sha512)
            .with_payload(BYTES2.to_vec())
            .build(&*signer)
            .unwrap();

        let transaction_header = pair.header_signature().to_string();

        let batch_1 = BatchBuilder::new()
            .with_transactions(vec![pair])
            .build(&*signer)
            .unwrap();

        let tracking_batch = TrackingBatchBuilder::default()
            .with_batch(batch_1)
            .with_service_id("TEST".to_string())
            .with_signer_public_key(KEY1.to_string())
            .with_submitted(false)
            .with_created_at(111111)
            .build()
            .unwrap();

        let id = tracking_batch.batch_header();

        let txn_receipts = vec![TransactionReceiptBuilder::default()
            .with_transaction_id(transaction_header)
            .with_result_valid(true)
            .with_serialized_receipt(std::str::from_utf8(&BYTES2).unwrap().to_string())
            .build()
            .unwrap()];

        let submission_error = SubmissionErrorBuilder::default()
            .with_error_type("test".to_string())
            .with_error_message("test message".to_string())
            .build()
            .unwrap();

        store
            .add_batches(vec![tracking_batch.clone()])
            .expect("Failed to add batch");

        store
            .change_batch_to_submitted(
                &id,
                "TEST",
                txn_receipts,
                Some("Pending"),
                Some(submission_error),
            )
            .expect("Failed to change batch to submitted");

        let batch = store
            .get_batch(&id, "TEST")
            .expect("Failed to get batch")
            .unwrap();

        assert_eq!(batch.submitted(), true)
    }

    #[test]
    fn change_batch_to_submitted_no_batch() {
        let pool = create_connection_pool_and_migrate();

        let store = DieselBatchTrackingStore::new(pool);

        let res = store
            .change_batch_to_submitted("id", "TEST", Vec::new(), Some("Pending"), None)
            .unwrap_err();

        assert_eq!(
            res.to_string(),
            BatchTrackingStoreError::NotFoundError("Could not find batch with ID id".to_string())
                .to_string()
        );
    }

    /// Creates a connection pool for an in-memory SQLite database with only a single connection
    /// available. Each connection is backed by a different in-memory SQLite database, so limiting
    /// the pool to a single connection ensures that the same DB is used for all operations.
    fn create_connection_pool_and_migrate() -> Pool<ConnectionManager<SqliteConnection>> {
        let connection_manager = ConnectionManager::<SqliteConnection>::new(":memory:");
        let pool = Pool::builder()
            .max_size(1)
            .build(connection_manager)
            .expect("Failed to build connection pool");

        run_sqlite_migrations(&*pool.get().expect("Failed to get connection for migrations"))
            .expect("Failed to run migrations");

        pool
    }

    fn new_signer() -> Box<dyn Signer> {
        let context = Secp256k1Context::new();
        let key = context.new_random_private_key();
        context.new_signer(key)
    }
}
