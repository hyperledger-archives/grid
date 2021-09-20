/*
 * Copyright 2019-2021 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use std::error::Error;
use std::fmt;

use grid_sdk::error::InternalError;

use grid_sdk::commits::store::CommitStoreError;
#[cfg(feature = "location")]
use grid_sdk::location::store::LocationStoreError;
#[cfg(feature = "pike")]
use grid_sdk::pike::store::PikeStoreError;
#[cfg(feature = "product")]
use grid_sdk::product::store::{ProductBuilderError, ProductStoreError};
#[cfg(feature = "purchase-order")]
use grid_sdk::purchase_order::store::{PurchaseOrderBuilderError, PurchaseOrderStoreError};
#[cfg(feature = "schema")]
use grid_sdk::schema::store::SchemaStoreError;
#[cfg(feature = "track-and-trace")]
use grid_sdk::track_and_trace::store::TrackAndTraceStoreError;

#[derive(Debug)]
pub struct EventProcessorError(pub String);

impl Error for EventProcessorError {}

impl fmt::Display for EventProcessorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Event Processor Error: {}", self.0)
    }
}

#[derive(Debug)]
pub struct EventError(pub String);

impl Error for EventError {}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Event Error: {}", self.0)
    }
}

impl From<InternalError> for EventError {
    fn from(err: InternalError) -> Self {
        EventError(format!("{}", err))
    }
}

impl From<CommitStoreError> for EventError {
    fn from(err: CommitStoreError) -> Self {
        EventError(format!("{}", err))
    }
}

#[cfg(feature = "location")]
impl From<LocationStoreError> for EventError {
    fn from(err: LocationStoreError) -> Self {
        EventError(format!("{}", err))
    }
}

#[cfg(feature = "pike")]
impl From<PikeStoreError> for EventError {
    fn from(err: PikeStoreError) -> Self {
        EventError(format!("{}", err))
    }
}

#[cfg(feature = "product")]
impl From<ProductStoreError> for EventError {
    fn from(err: ProductStoreError) -> Self {
        EventError(format!("{}", err))
    }
}

#[cfg(feature = "product")]
impl From<ProductBuilderError> for EventError {
    fn from(err: ProductBuilderError) -> Self {
        EventError(format!("{}", err))
    }
}

#[cfg(feature = "purchase-order")]
impl From<PurchaseOrderStoreError> for EventError {
    fn from(err: PurchaseOrderStoreError) -> Self {
        EventError(format!("{}", err))
    }
}

#[cfg(feature = "purchase-order")]
impl From<PurchaseOrderBuilderError> for EventError {
    fn from(err: PurchaseOrderBuilderError) -> Self {
        EventError(format!("{}", err))
    }
}

#[cfg(feature = "schema")]
impl From<SchemaStoreError> for EventError {
    fn from(err: SchemaStoreError) -> Self {
        EventError(format!("{}", err))
    }
}

#[cfg(feature = "track-and-trace")]
impl From<TrackAndTraceStoreError> for EventError {
    fn from(err: TrackAndTraceStoreError) -> Self {
        EventError(format!("{}", err))
    }
}

impl From<diesel::result::Error> for EventError {
    fn from(err: diesel::result::Error) -> Self {
        EventError(format!("{}", err))
    }
}

#[derive(Debug)]
pub enum EventIoError {
    ConnectionError(String),
    InvalidMessage(String),
}

impl Error for EventIoError {}

impl fmt::Display for EventIoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ConnectionError(err) => {
                write!(f, "event connection encountered an error: {}", err)
            }
            Self::InvalidMessage(err) => write!(f, "connection received invalid message: {}", err),
        }
    }
}
