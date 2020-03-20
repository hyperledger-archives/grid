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
use std::time::Duration;

use crate::matrix::{
    Envelope, MatrixAddError, MatrixLifeCycle, MatrixReceiver, MatrixRecvError,
    MatrixRecvTimeoutError, MatrixRemoveError, MatrixSendError, MatrixSender,
};
use crate::transport::Connection;

use super::{Mesh, RecvError, RecvTimeoutError};

#[derive(Clone)]
pub struct MeshLifeCycle {
    mesh: Mesh,
}

impl MeshLifeCycle {
    pub fn new(mesh: Mesh) -> Self {
        MeshLifeCycle { mesh }
    }
}

impl MatrixLifeCycle for MeshLifeCycle {
    fn add(&self, connection: Box<dyn Connection>, id: String) -> Result<usize, MatrixAddError> {
        self.mesh.add(connection, id).map_err(|err| {
            MatrixAddError::new(
                "Unable to add connection to Matrix".to_string(),
                Some(Box::new(err)),
            )
        })
    }

    fn remove(&self, id: &str) -> Result<Box<dyn Connection>, MatrixRemoveError> {
        self.mesh.remove(id).map_err(|err| {
            MatrixRemoveError::new(
                "Unable to remove connection from Matrix".to_string(),
                Some(Box::new(err)),
            )
        })
    }
}

#[derive(Clone)]
pub struct MeshMatrixSender {
    mesh: Mesh,
}

impl MeshMatrixSender {
    pub fn new(mesh: Mesh) -> Self {
        MeshMatrixSender { mesh }
    }
}

impl MatrixSender for MeshMatrixSender {
    fn send(&self, id: String, message: Vec<u8>) -> Result<(), MatrixSendError> {
        let envelope = Envelope::new(id, message);
        self.mesh.send(envelope).map_err(|err| {
            MatrixSendError::new(
                "Unable to send message to connection".to_string(),
                Some(Box::new(err)),
            )
        })
    }
}

#[derive(Clone)]
pub struct MeshMatrixReceiver {
    mesh: Mesh,
}

impl MatrixReceiver for MeshMatrixReceiver {
    fn recv(&self) -> Result<Envelope, MatrixRecvError> {
        match self.mesh.recv() {
            Ok(envelope) => Ok(envelope),
            Err(err) => match err {
                RecvError::Disconnected => Err(MatrixRecvError::Disconnected),
                RecvError::PoisonedLock => Err(MatrixRecvError::new_internal_error(
                    "Internal state poisoned".to_string(),
                    Some(Box::new(err)),
                )),
            },
        }
    }

    fn recv_timeout(&self, timeout: Duration) -> Result<Envelope, MatrixRecvTimeoutError> {
        match self.mesh.recv_timeout(timeout) {
            Ok(envelope) => Ok(envelope),
            Err(err) => match err {
                RecvTimeoutError::Timeout => Err(MatrixRecvTimeoutError::Timeout),
                RecvTimeoutError::Disconnected => Err(MatrixRecvTimeoutError::Disconnected),
                RecvTimeoutError::PoisonedLock => Err(MatrixRecvTimeoutError::new_internal_error(
                    "Internal state poisoned".to_string(),
                    Some(Box::new(err)),
                )),
            },
        }
    }
}
