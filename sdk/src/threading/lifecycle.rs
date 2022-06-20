// Copyright 2018-2022 Cargill Incorporated
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

//! Traits and functions related to the lifecycle of components.

use crate::error::InternalError;

/// `ShutdownHandle` is a trait which defines an interface for shutting down components which have
/// threads. It may also be used on non-threaded components which require cleanup at the end of
/// their lifecycle.
///
/// Two functions are defined which correspond to a structured two-phase shutdown sequence. The
/// first is `signal_shutdown` which instructs a component to begin the process of shutting down.
/// The second is `wait_for_shutdown` which will wait for shutdown to be complete; this typically
/// involves joining threads.
///
/// If multiple components are being shutdown, call `signal_shutdown` on all componets that can
/// safely shutdown in parallel, then call `wait_for_shutdown` on all of the components. The length
/// of time spent shutting down will be approximately the time of the slowest component.
pub trait ShutdownHandle {
    /// Instructs the component to begin shutting down.
    ///
    /// For components with threads, this should break out of any loops and ready the threads for
    /// being joined.
    fn signal_shutdown(&mut self);

    /// Waits until the the component has completely shutdown.
    ///
    /// For components with threads, the threads should be joined during the call to
    /// `wait_for_shutdown`.
    fn wait_for_shutdown(self) -> Result<(), InternalError>;
}
