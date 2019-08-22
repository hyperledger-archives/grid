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

use std::error::Error;

use crate::service::FactoryCreateError;

#[derive(Debug)]
pub struct NewOrchestratorError(pub Box<dyn Error + Send>);

impl Error for NewOrchestratorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&*self.0)
    }
}

impl std::fmt::Display for NewOrchestratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to create new orchestrator: {}", self.0)
    }
}

#[derive(Debug)]
pub enum OrchestratorError {
    Internal(Box<dyn Error + Send>),
    LockPoisoned,
}

impl Error for OrchestratorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            OrchestratorError::Internal(err) => Some(&**err),
            OrchestratorError::LockPoisoned => None,
        }
    }
}

impl std::fmt::Display for OrchestratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OrchestratorError::Internal(err) => {
                write!(f, "an orchestration error occurred: {}", err)
            }
            OrchestratorError::LockPoisoned => write!(f, "internal lock poisoned"),
        }
    }
}

#[derive(Debug)]
pub enum InitializeServiceError {
    InitializationFailed(Box<dyn Error + Send>),
    LockPoisoned,
    UnknownType,
}

impl Error for InitializeServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            InitializeServiceError::InitializationFailed(err) => Some(&**err),
            InitializeServiceError::LockPoisoned => None,
            InitializeServiceError::UnknownType => None,
        }
    }
}

impl std::fmt::Display for InitializeServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InitializeServiceError::InitializationFailed(err) => {
                write!(f, "failed to initialize service: {}", err)
            }
            InitializeServiceError::LockPoisoned => write!(f, "internal lock poisoned"),
            InitializeServiceError::UnknownType => write!(f, "service type unknown"),
        }
    }
}

impl From<FactoryCreateError> for InitializeServiceError {
    fn from(err: FactoryCreateError) -> Self {
        InitializeServiceError::InitializationFailed(Box::new(err))
    }
}

#[derive(Debug)]
pub enum ShutdownServiceError {
    LockPoisoned,
    ShutdownFailed(Box<dyn Error + Send>),
    UnknownService,
}

impl Error for ShutdownServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ShutdownServiceError::LockPoisoned => None,
            ShutdownServiceError::ShutdownFailed(err) => Some(&**err),
            ShutdownServiceError::UnknownService => None,
        }
    }
}

impl std::fmt::Display for ShutdownServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ShutdownServiceError::LockPoisoned => write!(f, "internal lock poisoned"),
            ShutdownServiceError::ShutdownFailed(err) => {
                write!(f, "failed to shutdown service: {}", err)
            }
            ShutdownServiceError::UnknownService => write!(f, "specified service not found"),
        }
    }
}

#[derive(Debug)]
pub enum ListServicesError {
    LockPoisoned,
}

impl Error for ListServicesError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ListServicesError::LockPoisoned => None,
        }
    }
}

impl std::fmt::Display for ListServicesError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ListServicesError::LockPoisoned => write!(f, "internal lock poisoned"),
        }
    }
}
