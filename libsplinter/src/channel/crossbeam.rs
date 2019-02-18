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

use crate::channel::{Receiver, RecvError, RecvTimeoutError, SendError, Sender, TryRecvError};

use std::time::Duration;

// Implement the Receiver and Sender Traits for crossbeam channels
impl<T> Receiver<T> for crossbeam_channel::Receiver<T>
where
    T: Send,
{
    fn recv(&self) -> Result<T, RecvError> {
        let request = crossbeam_channel::Receiver::recv(self).map_err(|err| RecvError {
            error: err.to_string(),
        })?;
        Ok(request)
    }

    fn try_recv(&self) -> Result<T, TryRecvError> {
        let request = match crossbeam_channel::Receiver::try_recv(self) {
            Ok(request) => request,
            Err(crossbeam_channel::TryRecvError::Empty) => return Err(TryRecvError::Empty),
            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                return Err(TryRecvError::Disconnected);
            }
        };
        Ok(request)
    }

    fn recv_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
        let request = match crossbeam_channel::Receiver::recv_timeout(self, timeout) {
            Ok(request) => request,
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                return Err(RecvTimeoutError::Timeout);
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                return Err(RecvTimeoutError::Disconnected);
            }
        };
        Ok(request)
    }
}

impl<T: 'static> Sender<T> for crossbeam_channel::Sender<T>
where
    T: Send,
{
    fn send(&self, request: T) -> Result<(), SendError> {
        crossbeam_channel::Sender::send(self, request).map_err(|err| SendError {
            error: err.to_string(),
        })?;
        Ok(())
    }

    fn box_clone(&self) -> Box<Sender<T>> {
        Box::new((*self).clone())
    }
}
