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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(PartialEq)]
enum Verification {
    Pending,
    Verified,
    Mismatch(Vec<u8>),
    Failure(String),
}

struct CommitVerification {
    service_name: String,
    result: Verification,
}

struct TransactionRecord {
    verifications: Vec<CommitVerification>,
    on_commit: Box<dyn Fn() -> Result<(), TransactionError>>,
}

/// Errors that may occur during an N-Phase Commit transaction.
#[derive(Debug, PartialEq)]
pub enum TransactionError {
    /// Indicates that the transaction with the given ID has already been started.
    AlreadyStarted(String),

    /// Indicates that the transaction with the given ID is unknown.
    UnknownTransaction(String),

    /// Indicates that a service that was not required to verify the given transaction has
    /// attempted to do so.
    UnexpectedVerifyingService {
        transaction_id: String,
        service_name: String,
    },
}

/// The N-Phase Commit Tracker
///
/// This struct is used to track the status of outstanding transactions.  The statuses are stored
/// until either (a) all services required to verify the transaction have reported their agreement
/// or (b) any service has reported an error or mismatch. In the successful case, a callback is
/// executed to inform the original caller to perform its commit operation.
///
/// In both cases, the transaction is no longer tracked.
#[derive(Clone, Default)]
pub struct NPhaseCommitTracker {
    transaction_verifications: Arc<Mutex<HashMap<String, TransactionRecord>>>,
}

impl NPhaseCommitTracker {
    /// Constructs a new NPhaseCommitTracker
    pub fn new() -> Self {
        Default::default()
    }

    /// Begin a transaction.
    ///
    /// A transaction is tracked by its ID and the services expected to verify the result.  The
    /// provided callback is executed when the transaction is fully verified by the services.
    pub fn begin_txn(
        &self,
        transaction_id: String,
        validating_service_names: &[&str],
        on_commit: Box<dyn Fn() -> Result<(), TransactionError>>,
    ) -> Result<(), TransactionError> {
        let mut transaction_verifications = mutex_lock_unwrap!(self.transaction_verifications);

        if transaction_verifications.contains_key(&transaction_id) {
            return Err(TransactionError::AlreadyStarted(transaction_id.to_string()));
        }

        transaction_verifications.insert(
            transaction_id,
            TransactionRecord {
                on_commit,
                verifications: validating_service_names
                    .iter()
                    .map(|service_name| CommitVerification {
                        service_name: service_name.to_string(),
                        result: Verification::Pending,
                    })
                    .collect(),
            },
        );

        Ok(())
    }

    /// Log a verification of a transaction.
    ///
    /// Log a verification by a given service.  Returns `true` if this log is the last expected
    /// verification.  The callback associated with the transaction will also be called.
    pub fn log_verification(
        &self,
        transaction_id: &str,
        service_name: &str,
    ) -> Result<bool, TransactionError> {
        let mut transaction_verifications = mutex_lock_unwrap!(self.transaction_verifications);
        let is_fully_committed = Self::log_result(
            &mut transaction_verifications,
            transaction_id,
            service_name,
            Verification::Verified,
        )?;

        if is_fully_committed {
            let txn_record = transaction_verifications
                .remove(transaction_id)
                .expect("transaction lost while committing");
            (&*txn_record.on_commit)()?;
        }

        Ok(is_fully_committed)
    }

    /// Log a mismatch (i.e. a verification failed).
    ///
    /// Log the verification of a given transaction failed for a specific service.  The transaction
    /// will be dropped from the tracker.
    pub fn log_mismatch(
        &self,
        transaction_id: &str,
        service_name: &str,
        mismatched_result: Vec<u8>,
    ) -> Result<(), TransactionError> {
        let mut transaction_verifications = mutex_lock_unwrap!(self.transaction_verifications);
        Self::log_result(
            &mut transaction_verifications,
            transaction_id,
            service_name,
            Verification::Mismatch(mismatched_result),
        )?;

        transaction_verifications.remove(transaction_id);

        Ok(())
    }

    /// Log an error.
    ///
    /// Log the verification of a given transaction failed due to an error for a specific service.
    /// The transaction will be dropped from the tracker.
    pub fn log_error(
        &self,
        transaction_id: &str,
        service_name: &str,
        error_message: String,
    ) -> Result<(), TransactionError> {
        let mut transaction_verifications = mutex_lock_unwrap!(self.transaction_verifications);
        Self::log_result(
            &mut transaction_verifications,
            transaction_id,
            service_name,
            Verification::Failure(error_message),
        )?;

        transaction_verifications.remove(transaction_id);

        Ok(())
    }

    fn log_result(
        transaction_verifications: &mut HashMap<String, TransactionRecord>,
        transaction_id: &str,
        service_name: &str,
        verification: Verification,
    ) -> Result<bool, TransactionError> {
        if let Some(transaction_record) = transaction_verifications.get_mut(transaction_id) {
            for commit_verification in transaction_record.verifications.iter_mut() {
                if commit_verification.service_name == service_name {
                    commit_verification.result = verification;
                    return Ok(!transaction_record.verifications.iter().any(
                        |commit_verification| commit_verification.result != Verification::Verified,
                    ));
                }
            }

            Err(TransactionError::UnexpectedVerifyingService {
                service_name: service_name.to_string(),
                transaction_id: transaction_id.to_string(),
            })
        } else {
            Err(TransactionError::UnknownTransaction(
                transaction_id.to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    /// Test that a simple two-phase commit status results in correct commit response.
    #[test]
    fn basic_commit() {
        let committed = Arc::new(AtomicBool::new(false));
        let commit_mgr = NPhaseCommitTracker::new();

        let on_commit_flag = committed.clone();
        assert_eq!(
            Ok(()),
            commit_mgr.begin_txn(
                "1".into(),
                &["service_b"],
                Box::new(move || {
                    on_commit_flag.store(true, Ordering::SeqCst);
                    Ok(())
                })
            )
        );

        assert_eq!(Ok(true), commit_mgr.log_verification("1", "service_b"));
        assert_eq!(true, committed.load(Ordering::SeqCst));

        // verify that it is no longer known
        assert_eq!(
            Err(TransactionError::UnknownTransaction("1".into())),
            commit_mgr.log_verification("1", "service_b")
        );
    }

    /// Test that a multi-phase commit status results in the correct commit response.
    #[test]
    fn basic_multi_phase_commit() {
        let committed = Arc::new(AtomicBool::new(false));
        let commit_mgr = NPhaseCommitTracker::new();

        let on_commit_flag = committed.clone();
        assert_eq!(
            Ok(()),
            commit_mgr.begin_txn(
                "1".into(),
                &["service_b", "service_c"],
                Box::new(move || {
                    on_commit_flag.store(true, Ordering::SeqCst);
                    Ok(())
                })
            )
        );

        assert_eq!(Ok(false), commit_mgr.log_verification("1", "service_b"));
        assert_eq!(false, committed.load(Ordering::SeqCst));
        assert_eq!(Ok(true), commit_mgr.log_verification("1", "service_c"));
        assert_eq!(true, committed.load(Ordering::SeqCst));

        // verify that it is no longer known
        assert_eq!(
            Err(TransactionError::UnknownTransaction("1".into())),
            commit_mgr.log_verification("1", "service_b")
        );
    }

    /// Test that a multi-phase commit status where one service logs a mismatch properly removes
    /// the transaction.
    #[test]
    fn log_mismatch() {
        let committed = Arc::new(AtomicBool::new(false));
        let commit_mgr = NPhaseCommitTracker::new();

        let on_commit_flag = committed.clone();
        assert_eq!(
            Ok(()),
            commit_mgr.begin_txn(
                "1".into(),
                &["service_b", "service_c"],
                Box::new(move || {
                    on_commit_flag.store(true, Ordering::SeqCst);
                    Ok(())
                })
            )
        );
        assert_eq!(
            Ok(()),
            commit_mgr.log_mismatch("1", "service_b", b"1234".to_vec())
        );
        assert_eq!(false, committed.load(Ordering::SeqCst));
        // The transaction should no longer exist
        assert_eq!(
            Err(TransactionError::UnknownTransaction("1".into())),
            commit_mgr.log_verification("1", "service_c")
        );

        assert_eq!(false, committed.load(Ordering::SeqCst));
    }

    /// Test that a multi-phase commit status where one service logs a failure properly removes the
    /// transaction.
    #[test]
    fn log_error() {
        let committed = Arc::new(AtomicBool::new(false));
        let commit_mgr = NPhaseCommitTracker::new();

        let on_commit_flag = committed.clone();
        assert_eq!(
            Ok(()),
            commit_mgr.begin_txn(
                "1".into(),
                &["service_b", "service_c"],
                Box::new(move || {
                    on_commit_flag.store(true, Ordering::SeqCst);
                    Ok(())
                })
            )
        );
        assert_eq!(
            Ok(()),
            commit_mgr.log_error("1", "service_b", "Some info on failure".to_string())
        );
        assert_eq!(false, committed.load(Ordering::SeqCst));
        // The transaction should no longer exist
        assert_eq!(
            Err(TransactionError::UnknownTransaction("1".into())),
            commit_mgr.log_verification("1", "service_c")
        );

        assert_eq!(false, committed.load(Ordering::SeqCst));
    }

    /// Test that a multi-phase commit returns an error if the service logging a message is
    /// unknown. Verify that it can still continue the transaction if the remaining services log a
    /// verification.
    #[test]
    fn ignore_unexpected_service() {
        let committed = Arc::new(AtomicBool::new(false));
        let commit_mgr = NPhaseCommitTracker::new();

        let on_commit_flag = committed.clone();
        assert_eq!(
            Ok(()),
            commit_mgr.begin_txn(
                "1".into(),
                &["service_b", "service_c"],
                Box::new(move || {
                    on_commit_flag.store(true, Ordering::SeqCst);
                    Ok(())
                })
            )
        );
        assert_eq!(Ok(false), commit_mgr.log_verification("1", "service_b"));
        assert_eq!(false, committed.load(Ordering::SeqCst));

        assert_eq!(
            Err(TransactionError::UnexpectedVerifyingService {
                transaction_id: "1".into(),
                service_name: "service_foo".into()
            }),
            commit_mgr.log_verification("1", "service_foo")
        );

        assert_eq!(Ok(true), commit_mgr.log_verification("1", "service_c"));
        assert_eq!(true, committed.load(Ordering::SeqCst));
    }
}
