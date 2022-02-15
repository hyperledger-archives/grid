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
use std::thread::JoinHandle;

use tokio::runtime::Runtime;

use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    future::{self, BoxFuture},
    SinkExt, StreamExt,
};

use super::{
    event::Event, BatchError, BatchId, BatchResult, BatchStatus, BatchStatusReader, BatchUpdater,
    PendingBatchStore,
};

pub enum Message {
    Poll,
    Drain,
}

/// Represents futures to wait for
enum WaitFor<'a, Status: BatchStatus> {
    /// Fire off an HTTP request and wait for the result
    Request(
        &'a dyn BatchStatusReader<Status = Status>,
        String,
        Vec<String>,
    ),

    /// Wait for a manual poll from an external source
    Poll(UnboundedReceiver<Message>),
}

impl<'a, Status: BatchStatus> WaitFor<'a, Status> {
    async fn run(self) -> Handle<Status> {
        match self {
            WaitFor::Request(reader, service_id, ids) => {
                let service_id = service_id.to_string();
                let result = reader.get_batch_statuses(&service_id, &ids).await;
                Handle::RequestResult(service_id, result, ids)
            }
            WaitFor::Poll(mut receiver) => match receiver.next().await {
                Some(event) => match event {
                    Message::Poll => Handle::Poll(receiver),
                    Message::Drain => Handle::Drain,
                },
                None => Handle::Drain,
            },
        }
    }
}

/// Represents results of futures that need to be handled
enum Handle<Status: BatchStatus> {
    RequestResult(String, BatchResult<Vec<Status>>, Vec<String>),
    Poll(UnboundedReceiver<Message>),
    Drain,
}

pub struct PollingMonitor<Id, Status, Obs, Store, Reader, Updater>
where
    Id: BatchId,
    Status: BatchStatus,
    Obs: Observer<Event = Event<Id, Status>>,
    Store: PendingBatchStore,
    Reader: BatchStatusReader<Status = Status>,
    Updater: BatchUpdater<Status = Status>,
{
    pub store: Store,
    pub reader: Reader,
    pub updater: Updater,
    pub notifier: Notifier<Obs>,
}

impl<Id, Status, Obs, Store, Reader, Updater>
    PollingMonitor<Id, Status, Obs, Store, Reader, Updater>
where
    Id: BatchId,
    Status: BatchStatus,
    Obs: Observer<Event = Event<Id, Status>>,
    Store: PendingBatchStore<Id = Id>,
    Reader: BatchStatusReader<Status = Status>,
    Updater: BatchUpdater<Status = Status>,
{
    async fn handle_response(
        &self,
        service_id: String,
        statuses: BatchResult<Vec<Status>>,
        pending: &[String],
    ) {
        match statuses {
            Err(e) => {
                self.notifier
                    .notify(&Event::Error(BatchError::InternalError(format!(
                        "encountered error {e:?} fetching batch statuses for service id \
        {service_id}"
                    ))))
                    .await;
            }
            Ok(statuses) => {
                self.notifier
                    .notify(&Event::FetchStatusesComplete {
                        service_id: service_id.clone(),
                        batches: pending.to_vec(),
                        statuses: statuses.clone(),
                    })
                    .await;

                let hash_set_pending: HashSet<_> = pending.iter().cloned().collect();
                let hash_set_response: HashSet<_> = statuses
                    .iter()
                    .map(|status| status.get_id().to_string())
                    .collect();

                let only_in_pending: Vec<_> =
                    hash_set_pending.difference(&hash_set_response).collect();

                let mut statuses_to_update: Vec<Status> = Vec::new();
                let mut only_in_response: Vec<String> = Vec::new();
                let mut unknown: Vec<Status> = Vec::new();
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
                    self.notifier
                        .notify(&Event::Error(BatchError::InternalError(format!(
                            "unexpected difference between submission and response during \
                            sanity check for service {service_id}. the following batch ids \
                            were submitted but not received back: {only_in_pending:?}. the \
                            following batch ids were received back but not submitted: \
                            {only_in_response:?}. these will not be updated."
                        ))))
                        .await;
                }

                if !unknown.is_empty() {
                    self.notifier
                        .notify(&Event::Error(BatchError::InternalError(format!(
                            "batches returned unknown status: {unknown:?}"
                        ))))
                        .await;
                }

                self.notifier
                    .notify(&Event::Update {
                        service_id: service_id.to_string(),
                        statuses: statuses_to_update.clone(),
                    })
                    .await;

                if let Err(e) = self
                    .updater
                    .update_batch_statuses(&service_id, &statuses_to_update)
                {
                    self.notifier
                        .notify(&Event::Error(BatchError::InternalError(format!(
                            "encountered error {e:?} fetching batch statuses for service id \
                        {service_id}"
                        ))))
                        .await;
                } else {
                    self.notifier
                        .notify(&Event::UpdateComplete {
                            service_id: service_id.to_string(),
                            statuses: statuses_to_update,
                        })
                        .await;
                }
            }
        }
    }

    async fn get_batches_by_service_id(
        &self,
        limit: usize,
    ) -> BatchResult<HashMap<String, Vec<String>>> {
        self.notifier.notify(&Event::FetchPending).await;

        let pending = self.store.get_pending_batch_ids(limit)?;
        self.notifier
            .notify(&Event::FetchPendingComplete {
                ids: pending.clone(),
            })
            .await;

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

    async fn make_batch_requests(&self) -> impl Iterator<Item = WaitFor<'_, Status>> {
        let limit = self.reader.available_connections();

        let batches_by_service_id = match self.get_batches_by_service_id(limit).await {
            Ok(batches_by_service_id) => batches_by_service_id,
            Err(e) => {
                self.notifier
                    .notify(&Event::Error(BatchError::InternalError(format!(
                        "encountered error {e:?} fetching pending batches"
                    ))))
                    .await;
                HashMap::new()
            }
        };

        for (service_id, batches) in batches_by_service_id.iter() {
            self.notifier
                .notify(&Event::FetchStatuses {
                    service_id: service_id.clone(),
                    batches: batches.clone(),
                })
                .await;
        }

        let reader = &self.reader;
        batches_by_service_id
            .into_iter()
            .map(move |(service_id, ids)| WaitFor::Request(reader, service_id, ids))
    }
}

pub trait RunnablePollingMonitor: Send {
    type Id: BatchId;
    type Status: BatchStatus;
    type Observer: Observer<Event = Event<Self::Id, Self::Status>>;
    type Store: PendingBatchStore<Id = Self::Id>;
    type Reader: BatchStatusReader<Status = Self::Status>;
    type Updater: BatchUpdater<Status = Self::Status>;

    #[allow(clippy::type_complexity)]
    fn build(
        self,
    ) -> PollingMonitor<
        Self::Id,
        Self::Status,
        Self::Observer,
        Self::Store,
        Self::Reader,
        Self::Updater,
    >;

    /// Start the polling monitor
    ///
    /// # Return value
    ///
    /// The running polling monitor
    fn run(self) -> BatchResult<RunningPollingMonitor>
    where
        Self: Sized + 'static,
    {
        let (sender, receiver) = mpsc::unbounded::<Message>();

        // Create the runtime
        let runtime = Runtime::new().map_err(|e| BatchError::InternalError(format!("{e:?}")))?;

        // Move the async runtime to a separate thread so it doesn't block this one
        let runtime_handle = std::thread::Builder::new()
            .name("dlt_polling_monitor_async_runtime_host".to_string())
            .spawn(move || {
                runtime.block_on(async move {
                    let monitor = self.build();

                    let mut unfinished_futures: Vec<_> =
                        vec![Box::pin(WaitFor::Poll(receiver).run())];

                    loop {
                        if unfinished_futures.is_empty() {
                            break;
                        }

                        // This blocks until the next future completes
                        let (event, _index, remaining) =
                            future::select_all(unfinished_futures).await;
                        unfinished_futures = remaining;

                        match event {
                            Handle::RequestResult(service_id, statuses, pending) => {
                                monitor
                                    .handle_response(service_id, statuses, &pending)
                                    .await;
                            }
                            Handle::Poll(receiver) => {
                                unfinished_futures.extend(
                                    monitor
                                        .make_batch_requests()
                                        .await
                                        .chain(iter::once(WaitFor::Poll(receiver)))
                                        .map(|item| Box::pin(item.run())), //.collect()
                                                                           //.into_iter()
                                );
                            }
                            Handle::Drain => {
                                // Do nothing.
                                //
                                // We're just not going to add any new futures,
                                // and wait for the remaining futures to complete
                            }
                        };
                    }
                })
            })
            .map_err(|e| BatchError::InternalError(format!("{e:?}")))?;

        Ok(RunningPollingMonitor {
            sender,
            runtime_handle,
        })
    }
}

pub struct RunningPollingMonitor {
    sender: UnboundedSender<Message>,
    runtime_handle: JoinHandle<()>,
}

impl RunningPollingMonitor {
    /// Create a Poller
    pub fn create_poller(&self) -> Poller {
        Poller {
            sender: self.sender.clone(),
        }
    }

    /// Stop the polling monitor
    pub async fn shutdown(mut self) -> BatchResult<()> {
        self.sender
            .send(Message::Drain)
            .await
            .map_err(|e| BatchError::InternalError(format!("{e:?}")))?;

        self.runtime_handle
            .join()
            .map_err(|e| BatchError::InternalError(format!("{e:?}")))
    }
}

/// Allows triggering a poll in the monitor externally
pub struct Poller {
    sender: UnboundedSender<Message>,
}

impl Poller {
    /// Trigger a poll in the monitor
    pub async fn poll(&mut self) -> BatchResult<()> {
        self.sender
            .send(Message::Poll)
            .await
            .map_err(|e| BatchError::InternalError(format!("{e:?}")))
    }
}

pub trait Observer {
    type Event;

    fn notify<'a>(&'a self, event: &'a Self::Event) -> BoxFuture<'a, ()>;
}

pub struct Notifier<O: Observer> {
    observers: Vec<O>,
}

impl<O: Observer> Notifier<O> {
    pub fn new(observers: Vec<O>) -> Self {
        Notifier { observers }
    }

    pub async fn notify(&self, event: &O::Event) {
        let _ = future::join_all(
            self.observers
                .iter()
                .map(|observer| observer.notify(event))
                .collect::<Vec<_>>(),
        )
        .await;
    }
}

#[cfg(test)]
mod mock {
    //! The mock component allows for the creation of mocks that
    //! will record and assert calls are valid.

    use std::fmt::Debug;
    use std::sync::{Arc, Mutex};

    pub struct TestBuilder<T, R> {
        expected_calls: Vec<T>,
        responses: Vec<R>,
    }

    impl<T: Debug + Eq + Clone + Ord, R> TestBuilder<T, R> {
        /// Creates a new TestBuilder
        pub fn new() -> Self {
            TestBuilder {
                expected_calls: Vec::new(),
                responses: Vec::new(),
            }
        }

        /// Adds an expectation
        ///
        /// # Arguments
        ///
        /// * call - The call to expect
        /// * response - The response that the mock should return for the call
        pub fn expect_call(mut self, call: T, response: R) -> Self {
            self.expected_calls.push(call);
            self.responses.push(response);
            self
        }

        /// Build the tester and asserter
        pub fn build(self) -> (CallTester<T, R>, CallAsserter<T, R>) {
            CallTester::new(self.expected_calls, self.responses)
        }
    }

    struct TestShared<T, R> {
        calls: Vec<Option<T>>,
        responses: Vec<Option<R>>,
    }

    /// CallTester is a mock that can record calls,
    /// give back the appropriate response, and assert
    /// some aspects of how calls are made.
    pub struct CallTester<T: Debug + Eq + Clone + Ord, R> {
        expected_calls: Vec<T>,
        actual: Arc<Mutex<TestShared<T, R>>>,
    }

    impl<T: Debug + Eq + Clone + Ord, R> CallTester<T, R> {
        fn new(
            expected_calls: Vec<T>,
            responses: Vec<R>,
        ) -> (CallTester<T, R>, CallAsserter<T, R>) {
            let actual = Arc::new(Mutex::new(TestShared {
                calls: responses.iter().map(|_| None).collect(),
                responses: responses.into_iter().map(Option::from).collect(),
            }));

            (
                CallTester {
                    expected_calls: expected_calls.clone(),
                    actual: actual.clone(),
                },
                CallAsserter {
                    expected_calls,
                    actual,
                },
            )
        }

        pub fn call(&self, call: T) -> R {
            let indices: Vec<_> = self
                .expected_calls
                .iter()
                .enumerate()
                .filter_map(|(i, expected_call)| {
                    if &call == expected_call {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect();

            if indices.len() == 0 {
                panic!("unexpected call {:?}", call);
            }

            let mut guard = self.actual.lock().unwrap();
            let protected_value = &mut *guard;

            let index = protected_value
                .responses
                .iter()
                .enumerate()
                .position(|(i, expected_call)| indices.contains(&i) && expected_call.is_some())
                .unwrap_or_else(|| panic!("unexpected call made for {:?}", call));

            let response = protected_value
                .responses
                .get_mut(index)
                .unwrap()
                .take()
                .unwrap();

            *protected_value.calls.get_mut(index).unwrap() = Some(call);

            let got = protected_value.calls.len();
            let expected = self.expected_calls.len();

            if got > expected {
                panic!("expected {} but got {} calls", expected, got);
            }

            response
        }
    }

    pub struct CallAsserter<T: Debug + Eq + Clone + Ord, R> {
        expected_calls: Vec<T>,
        actual: Arc<Mutex<TestShared<T, R>>>,
    }

    impl<T: Debug + Eq + Clone + Ord, R> CallAsserter<T, R> {
        /// Verify that all expected calls were made.
        /// This should be called upon test completion.
        pub fn assert(&mut self) {
            let mut guard = self.actual.lock().unwrap();
            let actual = &mut *guard;

            let mut actual = actual.calls.iter().map(Option::as_ref).collect::<Vec<_>>();
            let mut expected = self
                .expected_calls
                .iter()
                .map(Option::from)
                .collect::<Vec<_>>();

            assert_eq!(
                // Sort the calls to eliminate call order differences
                actual.sort(),
                expected.sort(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::boxed::Box;
    use std::fmt::Debug;
    use std::pin::Pin;

    use super::mock::{CallAsserter, CallTester, TestBuilder};

    use futures::future::{self, BoxFuture, Future};

    #[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
    pub enum Status {
        Unknown,
        Valid,
    }

    #[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
    pub struct FakeBatchStatus {
        pub id: String,
        pub status: Status,
    }

    impl BatchStatus for FakeBatchStatus {
        fn get_id(&self) -> &str {
            &self.id
        }

        fn is_unknown(&self) -> bool {
            matches!(self.status, Status::Unknown)
        }
    }

    #[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
    pub struct FakeBatchId {
        pub id: String,
        pub service_id: String,
    }

    impl BatchId for FakeBatchId {
        fn get_id(&self) -> &str {
            &self.id
        }

        fn get_service_id(&self) -> &str {
            &self.service_id
        }
    }

    #[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
    struct BatchUpdateCall {
        service_id: String,
        statuses: Vec<FakeBatchStatus>,
    }

    #[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
    struct BatchStatusCall {
        service_id: String,
        batch_ids: Vec<String>,
    }

    type ObserverCall = Event<FakeBatchId, FakeBatchStatus>;

    type MockStore = CallTester<usize, BatchResult<Vec<FakeBatchId>>>;
    type StatusResult<'a> = BoxFuture<'a, BatchResult<Vec<FakeBatchStatus>>>;
    type MockReader<'a> = CallTester<BatchStatusCall, StatusResult<'a>>;
    type MockUpdater = CallTester<BatchUpdateCall, BatchResult<()>>;
    type MockObserver<'a> = CallTester<ObserverCall, BoxFuture<'a, ()>>;

    impl PendingBatchStore for MockStore {
        type Id = FakeBatchId;

        fn get_pending_batch_ids(&self, limit: usize) -> BatchResult<Vec<FakeBatchId>> {
            self.call(limit)
        }
    }

    impl BatchStatusReader for MockReader<'_> {
        type Status = FakeBatchStatus;

        fn get_batch_statuses(
            &self,
            service_id: &str,
            batch_ids: &[String],
        ) -> BoxFuture<'_, BatchResult<Vec<FakeBatchStatus>>> {
            self.call(BatchStatusCall {
                service_id: service_id.to_string(),
                batch_ids: batch_ids.to_vec(),
            })
        }

        fn available_connections(&self) -> usize {
            100
        }
    }

    impl BatchUpdater for MockUpdater {
        type Status = FakeBatchStatus;

        fn update_batch_statuses(
            &self,
            service_id: &str,
            batches: &[FakeBatchStatus],
        ) -> BatchResult<()> {
            self.call(BatchUpdateCall {
                service_id: service_id.to_string(),
                statuses: batches.to_vec(),
            })
        }
    }

    type MockMonitor<'a> = PollingMonitor<
        FakeBatchId,
        FakeBatchStatus,
        MockObserver<'a>,
        MockStore,
        MockReader<'a>,
        MockUpdater,
    >;

    struct MockMonitorRunnable<'a> {
        construct: Box<dyn FnOnce() -> MockMonitor<'a> + Send>,
    }

    impl<'a> RunnablePollingMonitor for MockMonitorRunnable<'a> {
        type Id = FakeBatchId;
        type Status = FakeBatchStatus;
        type Observer = MockObserver<'a>;
        type Store = MockStore;
        type Reader = MockReader<'a>;
        type Updater = MockUpdater;

        fn build(self) -> MockMonitor<'a> {
            (self.construct)()
        }
    }

    impl<'b> Observer for MockObserver<'b> {
        type Event = Event<FakeBatchId, FakeBatchStatus>;

        fn notify<'a>(
            &'a self,
            _event: &'a Event<FakeBatchId, FakeBatchStatus>,
        ) -> BoxFuture<'a, ()> {
            Box::pin(future::ready(())) as BoxFuture<'_, ()>

            // We're not checking the events in this test
            // but if we did, we would call this:
            //self.call(event.clone())
        }
    }

    // This test mocks out the store, reader, and updater.
    // It then checks to make sure that all the correct calls
    // to those components are made for a given set of
    // data from the store.
    #[actix_rt::test]
    async fn update_sync_correctly_updates_statuses() {
        let (store, mut store_asserter): (MockStore, _) = TestBuilder::new()
            .expect_call(
                100,
                Ok(vec![
                    FakeBatchId {
                        id: "one".to_string(),
                        service_id: "a".to_string(),
                    },
                    FakeBatchId {
                        id: "two".to_string(),
                        service_id: "a".to_string(),
                    },
                    FakeBatchId {
                        id: "three".to_string(),
                        service_id: "b".to_string(),
                    },
                    FakeBatchId {
                        id: "four".to_string(),
                        service_id: "b".to_string(),
                    },
                ]),
            )
            .build();

        let (reader, mut reader_asserter): (MockReader, _) = TestBuilder::new()
            .expect_call(
                BatchStatusCall {
                    service_id: "a".to_string(),
                    batch_ids: vec!["one".to_string(), "two".to_string()],
                },
                Box::pin(future::ok(vec![
                    FakeBatchStatus {
                        id: "one".to_string(),
                        status: Status::Valid,
                    },
                    FakeBatchStatus {
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
                    FakeBatchStatus {
                        id: "three".to_string(),
                        status: Status::Valid,
                    },
                    FakeBatchStatus {
                        id: "four".to_string(),
                        status: Status::Unknown,
                    },
                ])) as StatusResult<'_>,
            )
            .build();

        let (updater, mut updater_asserter): (MockUpdater, _) = TestBuilder::new()
            .expect_call(
                BatchUpdateCall {
                    service_id: "a".to_string(),
                    statuses: vec![
                        FakeBatchStatus {
                            id: "one".to_string(),
                            status: Status::Valid,
                        },
                        FakeBatchStatus {
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
                    statuses: vec![FakeBatchStatus {
                        id: "three".to_string(),
                        status: Status::Valid,
                    }],
                },
                Ok(()),
            )
            .build();

        let (observer, mut observer_asserter): (MockObserver, _) = TestBuilder::new()
            .expect_call(
                Event::FetchPending,
                Box::pin(future::ready(())) as BoxFuture<'_, ()>,
            )
            .build();

        let runnable = MockMonitorRunnable {
            construct: Box::new(move || MockMonitor {
                notifier: Notifier::new(vec![observer]),
                store,
                reader,
                updater,
            }),
        };

        let monitor = runnable.run().expect("could not run");
        let mut poller = monitor.create_poller();

        poller.poll().await.expect("unexpected send error");
        monitor.shutdown().await.expect("could not shut down");

        store_asserter.assert();
        reader_asserter.assert();
        updater_asserter.assert();
    }
}
