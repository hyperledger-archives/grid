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

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::iter;
use std::time::Instant;

use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    future::Future,
    StreamExt,
};

use super::{
    event::Event, stream, BatchError, BatchId, BatchResult, BatchStatus, BatchStatusReader,
    BatchUpdater, PendingBatchStore,
};

fn handle_response<I: BatchId, T: BatchStatus>(
    emit: &mut dyn FnMut(Event<I>),
    service_id: String,
    start: Instant,
    pending: &[String],
    statuses: BatchResult<Vec<T>>,
    batch_updater: &impl BatchUpdater<T>,
) {
    match statuses {
        Err(e) => {
            emit(Event::Error(BatchError::InternalError(format!(
                "encountered error {e:?} fetching batch statuses for service id \
        {service_id}"
            ))));
        }
        Ok(statuses) => {
            emit(Event::FetchStatusesComplete(
                service_id.clone(),
                statuses.len(),
                start.elapsed(),
            ));

            let hash_set_pending: HashSet<_> = pending.iter().cloned().collect();
            let hash_set_response: HashSet<_> = statuses
                .iter()
                .map(|status| status.get_id().to_string())
                .collect();

            let only_in_pending: Vec<_> = hash_set_pending.difference(&hash_set_response).collect();

            let mut statuses_to_update: Vec<T> = Vec::new();
            let mut only_in_response: Vec<String> = Vec::new();
            let mut unknown: Vec<T> = Vec::new();
            for status in statuses {
                if !hash_set_pending.contains(status.get_id()) {
                    only_in_response.push(status.get_id().to_string());
                    continue;
                }

                if status.is_unknown() {
                    // Sawtooth will return status "Unknown" if the batch fell out of
                    // the batch cache, which lasts appx 5m.
                    //
                    // Splinter will return "Unknown" for batches that did not respond
                    // within the specified wait timeout.
                    //
                    // Because the status is technically unknown and maybe still
                    // pending, we'll ignore any status items with this status until
                    // they are removed from the store by a different process.
                    unknown.push(status);
                } else {
                    statuses_to_update.push(status);
                }
            }

            if !only_in_pending.is_empty() || !only_in_response.is_empty() {
                emit(Event::Error(BatchError::InternalError(format!(
                    "unexpected difference between submission and response during \
                            sanity check for service {service_id}. the following batch ids \
                            were submitted but not received back: {only_in_pending:?}. the \
                            following batch ids were received back but not submitted: \
                            {only_in_response:?}. these will not be updated."
                ))));
            }

            if !unknown.is_empty() {
                emit(Event::Error(BatchError::InternalError(format!(
                    "batches returned unknown status: {unknown:?}"
                ))));
            }

            let start = Instant::now();
            if let Err(e) = batch_updater.update_batch_statuses(&service_id, &statuses_to_update) {
                emit(Event::Error(BatchError::InternalError(format!(
                    "encountered error {e:?} fetching batch statuses for service id \
                        {service_id}"
                ))));
            } else {
                emit(Event::UpdateComplete(
                    service_id.to_string(),
                    statuses_to_update.len(),
                    start.elapsed(),
                ));
            }
        }
    }
}

fn get_batches_by_service_id<I: BatchId>(
    emit: &mut dyn FnMut(Event<I>),
    pending_batch_store: &impl PendingBatchStore<I>,
    limit: usize,
) -> BatchResult<HashMap<String, Vec<String>>> {
    emit(Event::FetchPending);

    let begin_time = Instant::now();
    let pending = pending_batch_store.get_pending_batch_ids(limit)?;
    emit(Event::FetchPendingComplete(
        begin_time.elapsed(),
        pending.clone(),
    ));

    Ok(pending.iter().fold(
        HashMap::new(),
        |mut init: HashMap<String, Vec<String>>, item| {
            let id = item.get_id().to_string();
            match init.entry(item.get_service_id().to_string()) {
                Entry::Occupied(o) => o.into_mut().push(id),
                Entry::Vacant(v) => {
                    v.insert(vec![id]);
                }
            };
            init
        },
    ))
}

fn make_batch_requests<'a, I: BatchId, T: BatchStatus + 'a>(
    emit: &mut dyn FnMut(Event<I>),
    pending_batch_store: &impl PendingBatchStore<I>,
    batch_status_reader: &'a impl BatchStatusReader<T>,
) -> impl Iterator<Item = WaitFor<'a, T>> {
    let limit = batch_status_reader.available_connections();

    let batches_by_service_id = match get_batches_by_service_id(emit, pending_batch_store, limit) {
        Ok(batches_by_service_id) => batches_by_service_id,
        Err(e) => {
            emit(Event::Error(BatchError::InternalError(format!(
                "encountered error {e:?} fetching pending batches"
            ))));
            HashMap::new()
        }
    };

    batches_by_service_id
        .into_iter()
        .map(move |(service_id, ids)| WaitFor::Request(batch_status_reader, service_id, ids))
}

/// Represents futures to wait for
enum WaitFor<'a, T: BatchStatus> {
    /// Fire off an HTTP request and wait for the result
    Request(&'a dyn BatchStatusReader<T>, String, Vec<String>),

    /// Wait for a manual poll from an external source
    Poll(UnboundedReceiver<()>),
}

impl<'a, T: BatchStatus> WaitFor<'a, T> {
    async fn run(self) -> Handle<T> {
        match self {
            WaitFor::Request(batch_status_reader, service_id, ids) => {
                let start = Instant::now();
                let service_id = service_id.to_string();
                let result = batch_status_reader
                    .get_batch_statuses(&service_id, &ids)
                    .await;
                Handle::RequestResult(start, service_id, result, ids)
            }
            WaitFor::Poll(mut receiver) => match receiver.next().await {
                Some(()) => Handle::Poll(receiver),
                None => Handle::Drain,
            },
        }
    }
}

/// Represents results of futures that need to be handled
enum Handle<T: BatchStatus> {
    RequestResult(Instant, String, BatchResult<Vec<T>>, Vec<String>),
    Poll(UnboundedReceiver<()>),
    Drain,
}

/// Create a new polling monitor
///
/// # Arguments
///
/// * emit - A function that receives and handles status events
/// * pending_batch_store - A store that can retrieve pending batches
/// * batch_status_reader - A reader that can retrieve batch statuses
/// * batch_updater - An updater for updating the batch statuses
///
/// # Return value
///
/// The response is a channel that can be used to cause the
/// polling monitor to poll, and a future that causes the
/// monitor to run.
pub fn create_polling_monitor<'a, I: BatchId, T: BatchStatus + 'a>(
    emit: &'a mut dyn FnMut(Event<I>),
    pending_batch_store: &'a impl PendingBatchStore<I>,
    batch_status_reader: &'a impl BatchStatusReader<T>,
    batch_updater: &'a impl BatchUpdater<T>,
) -> (UnboundedSender<()>, impl Future<Output = ()> + 'a) {
    let (poll_listener, receiver) = mpsc::unbounded::<()>();

    let async_loop = stream::create_async_loop(
        vec![Box::pin(WaitFor::Poll(receiver).run())],
        move |event: Handle<T>, add_items: &mut dyn FnMut(_)| match event {
            Handle::RequestResult(start, service_id, statuses, pending) => {
                handle_response(emit, service_id, start, &pending, statuses, batch_updater);
            }
            Handle::Poll(receiver) => {
                add_items(
                    make_batch_requests(emit, pending_batch_store, batch_status_reader)
                        .chain(iter::once(WaitFor::Poll(receiver)))
                        .map(|item| Box::pin(item.run()))
                        .collect(),
                );
            }
            Handle::Drain => {
                // Do nothing.
                //
                // We're just not going to add any new futures,
                // and wait for the remaining futures to complete
            }
        },
    );

    (poll_listener, async_loop)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::boxed::Box;
    use std::fmt::Debug;
    use std::sync::{Arc, Mutex};

    use futures::{
        future::{self, BoxFuture},
        SinkExt,
    };

    #[derive(PartialEq, Debug, Clone)]
    pub enum Status {
        Unknown,
        Valid,
    }

    #[derive(PartialEq, Debug, Clone)]
    pub struct TestBatchStatus {
        pub id: String,
        pub status: Status,
    }

    impl BatchStatus for TestBatchStatus {
        fn get_id(&self) -> &str {
            &self.id
        }

        fn is_unknown(&self) -> bool {
            matches!(self.status, Status::Unknown)
        }
    }

    #[derive(PartialEq, Debug, Clone)]
    pub struct TestBatchId {
        pub id: String,
        pub service_id: String,
    }

    impl BatchId for TestBatchId {
        fn get_id(&self) -> &str {
            &self.id
        }

        fn get_service_id(&self) -> &str {
            &self.service_id
        }
    }

    #[derive(PartialEq, Debug, Clone)]
    struct BatchUpdateCall {
        service_id: String,
        statuses: Vec<TestBatchStatus>,
    }

    #[derive(PartialEq, Debug, Clone)]
    struct BatchStatusCall {
        service_id: String,
        batch_ids: Vec<String>,
    }

    struct TestShared<T, R> {
        actual_calls: Vec<Option<T>>,
        responses: Vec<Option<R>>,
    }

    struct CallTester<T: Debug + PartialEq, R> {
        expected_calls: Vec<T>,
        shared: Arc<Mutex<TestShared<T, R>>>,
    }

    impl<T: Debug + PartialEq, R> CallTester<T, R> {
        fn new(expected_calls: Vec<T>, responses: Vec<R>) -> Self {
            CallTester {
                expected_calls,
                shared: Arc::new(Mutex::new(TestShared {
                    actual_calls: responses.iter().map(|_| None).collect(),
                    responses: responses.into_iter().map(Option::from).collect(),
                })),
            }
        }

        fn assert(self) {
            let mut guard = self.shared.lock().unwrap();
            let shared = &mut *guard;
            assert_eq!(
                shared.actual_calls,
                self.expected_calls
                    .into_iter()
                    .map(Option::from)
                    .collect::<Vec<_>>()
            );
        }

        fn call(&self, call: T) -> R {
            let index = self
                .expected_calls
                .iter()
                .position(|expected_call| expected_call == &call)
                .unwrap_or_else(|| panic!("unexpected call {:?}", call));

            let mut guard = self.shared.lock().unwrap();
            let protected_value = &mut *guard;

            let response = protected_value
                .responses
                .get_mut(index)
                .unwrap()
                .take()
                .unwrap_or_else(|| panic!("duplicate call made for {:?}", call));

            *protected_value.actual_calls.get_mut(index).unwrap() = Some(call);

            let got = protected_value.actual_calls.len();
            let expected = self.expected_calls.len();

            if got > expected {
                panic!("expected {} but got {} calls", expected, got);
            }

            response
        }
    }

    struct TestBuilder<T, R> {
        expected_calls: Vec<T>,
        responses: Vec<R>,
    }

    impl<T: Debug + PartialEq, R> TestBuilder<T, R> {
        fn new() -> Self {
            TestBuilder {
                expected_calls: Vec::new(),
                responses: Vec::new(),
            }
        }

        fn expect_call(mut self, call: T, response: R) -> Self {
            self.expected_calls.push(call);
            self.responses.push(response);
            self
        }

        fn build(self) -> CallTester<T, R> {
            CallTester::new(self.expected_calls, self.responses)
        }
    }

    type TestPendingBatchStore = CallTester<(), BatchResult<Vec<TestBatchId>>>;
    type StatusResult<'a> = BoxFuture<'a, BatchResult<Vec<TestBatchStatus>>>;
    type TestBatchStatusStore<'a> = CallTester<BatchStatusCall, StatusResult<'a>>;
    type TestUpdateBatchStore = CallTester<BatchUpdateCall, BatchResult<()>>;

    impl PendingBatchStore<TestBatchId> for TestPendingBatchStore {
        fn get_pending_batch_ids(&self, _limit: usize) -> BatchResult<Vec<TestBatchId>> {
            self.call(())
        }
    }

    impl BatchStatusReader<TestBatchStatus> for TestBatchStatusStore<'_> {
        fn get_batch_statuses(
            &self,
            service_id: &str,
            batch_ids: &[String],
        ) -> BoxFuture<'_, BatchResult<Vec<TestBatchStatus>>> {
            self.call(BatchStatusCall {
                service_id: service_id.to_string(),
                batch_ids: batch_ids.to_vec(),
            })
        }

        fn available_connections(&self) -> usize {
            100
        }
    }

    impl BatchUpdater<TestBatchStatus> for TestUpdateBatchStore {
        fn update_batch_statuses(
            &self,
            service_id: &str,
            batches: &[TestBatchStatus],
        ) -> BatchResult<()> {
            self.call(BatchUpdateCall {
                service_id: service_id.to_string(),
                statuses: batches.to_vec(),
            })
        }
    }

    #[tokio::test]
    async fn update_sync_correctly_updates_statuses() {
        let pending_batch_store: TestPendingBatchStore = TestBuilder::new()
            .expect_call(
                (),
                Ok(vec![
                    TestBatchId {
                        id: "one".to_string(),
                        service_id: "a".to_string(),
                    },
                    TestBatchId {
                        id: "two".to_string(),
                        service_id: "a".to_string(),
                    },
                    TestBatchId {
                        id: "three".to_string(),
                        service_id: "b".to_string(),
                    },
                    TestBatchId {
                        id: "four".to_string(),
                        service_id: "b".to_string(),
                    },
                ]),
            )
            .build();

        let batch_status_reader: TestBatchStatusStore = TestBuilder::new()
            .expect_call(
                BatchStatusCall {
                    service_id: "a".to_string(),
                    batch_ids: vec!["one".to_string(), "two".to_string()],
                },
                Box::pin(future::ok(vec![
                    TestBatchStatus {
                        id: "one".to_string(),
                        status: Status::Valid,
                    },
                    TestBatchStatus {
                        id: "two".to_string(),
                        status: Status::Valid,
                    },
                ])) as StatusResult<'_>,
            )
            .expect_call(
                BatchStatusCall {
                    service_id: "b".to_string(),
                    batch_ids: vec!["three".to_string(), "four".to_string()],
                },
                Box::pin(future::ok(vec![
                    TestBatchStatus {
                        id: "three".to_string(),
                        status: Status::Valid,
                    },
                    TestBatchStatus {
                        id: "four".to_string(),
                        status: Status::Unknown,
                    },
                ])) as StatusResult<'_>,
            )
            .build();

        let batch_updater: TestUpdateBatchStore = TestBuilder::new()
            .expect_call(
                BatchUpdateCall {
                    service_id: "a".to_string(),
                    statuses: vec![
                        TestBatchStatus {
                            id: "one".to_string(),
                            status: Status::Valid,
                        },
                        TestBatchStatus {
                            id: "two".to_string(),
                            status: Status::Valid,
                        },
                    ],
                },
                Ok(()),
            )
            .expect_call(
                BatchUpdateCall {
                    service_id: "b".to_string(),
                    statuses: vec![TestBatchStatus {
                        id: "three".to_string(),
                        status: Status::Valid,
                    }],
                },
                Ok(()),
            )
            .build();

        let mut event = |_: Event<TestBatchId>| {};
        let (mut sender, poller) = create_polling_monitor(
            &mut event,
            &pending_batch_store,
            &batch_status_reader,
            &batch_updater,
        );

        sender.send(()).await.expect("unexpected send error");
        sender.disconnect();

        poller.await;

        pending_batch_store.assert();
        batch_status_reader.assert();
        batch_updater.assert();
    }
}
