// Copyright 2018-2020 Cargill Incorporated
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

use std::any::Any;
use std::sync::{Arc, Mutex};

use crate::channel::{SendError, Sender};

/// The mock sender allows for tests of components or functions that take a Sender to test the sent
/// values synchronously. Removes the need for blocking (beyond the Arc/Mutex combo internally) on
/// a receiver.
#[derive(Clone)]
pub struct MockSender<T: Clone> {
    sent: Arc<Mutex<Vec<T>>>,
}

impl<T: Clone> Default for MockSender<T> {
    fn default() -> Self {
        MockSender {
            sent: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl<T: Clone> MockSender<T> {
    pub fn new(sent: Arc<Mutex<Vec<T>>>) -> Self {
        MockSender { sent }
    }

    pub fn sent(&self) -> Vec<T> {
        self.sent.lock().unwrap().clone()
    }

    /// Clear the Sent list, and return the previous items
    pub fn clear(&self) -> Vec<T> {
        let mut sent = self.sent.lock().unwrap();
        let mut current = Vec::new();

        std::mem::swap(&mut *sent, &mut current);

        current
    }
}

impl<T: Any + Clone + Send> Sender<T> for MockSender<T> {
    fn send(&self, message: T) -> Result<(), SendError> {
        self.sent.lock().unwrap().push(message);
        Ok(())
    }

    fn box_clone(&self) -> Box<dyn Sender<T>> {
        Box::new(MockSender {
            sent: self.sent.clone(),
        })
    }
}
