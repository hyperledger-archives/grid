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

use std::time::{Duration, Instant};

use crate::dlt_monitor::event::socket::Handler;

pub struct ThrottledHandler<T: Send + FnMut()> {
    throttle: Duration,
    last_call: Option<Instant>,
    callback: T,
}

impl<T: Send + FnMut()> ThrottledHandler<T> {
    pub fn new(throttle: Duration, callback: T) -> Self {
        ThrottledHandler {
            throttle,
            last_call: None,
            callback,
        }
    }

    fn call(&mut self, _: Vec<u8>) {
        (self.callback)();
    }
}

impl<T: Send + FnMut()> Handler for ThrottledHandler<T> {
    fn handle(&mut self, message: Vec<u8>) {
        match self.last_call {
            Some(last_call) => {
                let now = Instant::now();
                if now > last_call + self.throttle {
                    self.call(message);
                }
            }
            None => {
                self.last_call = Some(Instant::now());
                self.call(message);
            }
        }
    }
}
