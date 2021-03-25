// Copyright 2018-2021 Cargill Incorporated
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

//! A Reqwest-based implementation of Client

use crate::batches::store::BatchList;
use crate::error::InternalError;

use super::Client;
#[cfg(feature = "purchase-order")]
use super::{PurchaseOrder, PurchaseOrderRevision, PurchaseOrderVersion};

#[derive(Deserialize)]
struct ServerError {
    pub message: String,
}

pub struct ReqwestClient {
    pub url: String,
}

impl ReqwestClient {
    pub fn _new(url: String) -> Self {
        ReqwestClient { url }
    }
}

impl Client for ReqwestClient {
    fn post_batches(&self, _batches: BatchList) -> Result<(), InternalError> {
        unimplemented!()
    }

    /// Retrieves the purchase order with the specified `id`.
    #[cfg(feature = "purchase-order")]
    fn get_purchase_order(&self, _id: String) -> Result<Option<PurchaseOrder>, InternalError> {
        unimplemented!()
    }

    /// Retrieves the purchase order version with the given `version_id` of the purchase order
    /// with the given `id`
    #[cfg(feature = "purchase-order")]
    fn get_purchase_order_version(
        &self,
        _id: String,
        _version_id: String,
    ) -> Result<Option<PurchaseOrderVersion>, InternalError> {
        unimplemented!()
    }

    /// Retrieves the purchase order revision with the given `revision_id` of the purchase
    /// order version with the given `version_id` of the purchase order with the given `id`
    #[cfg(feature = "purchase-order")]
    fn get_purchase_order_revision(
        &self,
        _id: String,
        _version_id: String,
        _revision_id: String,
    ) -> Result<Option<PurchaseOrderRevision>, InternalError> {
        unimplemented!()
    }

    /// lists purchase orders.
    #[cfg(feature = "purchase-order")]
    fn list_purchase_orders(
        &self,
        _filter: Option<&str>,
    ) -> Result<Vec<PurchaseOrder>, InternalError> {
        unimplemented!()
    }

    /// lists the purchase order versions of a specific purchase order.
    #[cfg(feature = "purchase-order")]
    fn list_purchase_order_versions(
        &self,
        _id: String,
        _filter: Option<&str>,
    ) -> Result<Vec<PurchaseOrderVersion>, InternalError> {
        unimplemented!()
    }

    /// lists the purchase order revisions of a specific purchase order version.
    #[cfg(feature = "purchase-order")]
    fn list_purchase_order_revisions(
        &self,
        _id: String,
        _version_id: String,
        _filter: Option<&str>,
    ) -> Result<Vec<PurchaseOrderRevision>, InternalError> {
        unimplemented!()
    }
}
