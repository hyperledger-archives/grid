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

use super::PurchaseOrderStoreOperations;
use crate::purchase_order::store::diesel::{PurchaseOrderAlternateIdList, PurchaseOrderStoreError};

pub(in crate::purchase_order::store::diesel) trait PurchaseOrderStoreListAlternateIdsForPurchaseOrderOperation
{
    fn list_alternate_ids_for_purchase_order(
        &self,
        purchase_order_uuid: &str,
        org_id: &str,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<PurchaseOrderAlternateIdList, PurchaseOrderStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> PurchaseOrderStoreListAlternateIdsForPurchaseOrderOperation
    for PurchaseOrderStoreOperations<'a, diesel::pg::PgConnection>
{
    fn list_alternate_ids_for_purchase_order(
        &self,
        _purchase_order_uuid: &str,
        _org_id: &str,
        _service_id: Option<&str>,
        _offset: i64,
        _limit: i64,
    ) -> Result<PurchaseOrderAlternateIdList, PurchaseOrderStoreError> {
        unimplemented!()
    }
}

#[cfg(feature = "sqlite")]
impl<'a> PurchaseOrderStoreListAlternateIdsForPurchaseOrderOperation
    for PurchaseOrderStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn list_alternate_ids_for_purchase_order(
        &self,
        _purchase_order_uuid: &str,
        _org_id: &str,
        _service_id: Option<&str>,
        _offset: i64,
        _limit: i64,
    ) -> Result<PurchaseOrderAlternateIdList, PurchaseOrderStoreError> {
        unimplemented!()
    }
}
