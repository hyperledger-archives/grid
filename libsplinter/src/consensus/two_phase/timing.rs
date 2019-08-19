// Copyright 2018 Bitwise IO, Inc.
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

use std::time::{Duration, Instant};

#[derive(Debug, PartialEq)]
enum TimeoutState {
    Active,
    Inactive,
    Expired,
}

/// A timer that expires after a given duration. Check back on this timer every so often to see if
/// it's expired.
#[derive(Debug)]
pub struct Timeout {
    state: TimeoutState,
    duration: Duration,
    start: Instant,
}

impl Timeout {
    pub fn new(duration: Duration) -> Self {
        Timeout {
            state: TimeoutState::Inactive,
            duration,
            start: Instant::now(),
        }
    }

    /// Update the timer state, and check if the timer is expired
    pub fn check_expired(&mut self) -> bool {
        if self.state == TimeoutState::Active && Instant::now() - self.start > self.duration {
            self.state = TimeoutState::Expired;
        }
        match self.state {
            TimeoutState::Active | TimeoutState::Inactive => false,
            TimeoutState::Expired => true,
        }
    }

    pub fn start(&mut self) {
        self.state = TimeoutState::Active;
        self.start = Instant::now();
    }

    pub fn stop(&mut self) {
        self.state = TimeoutState::Inactive;
    }
}
