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

pub(super) mod add_alternate_id;
pub(super) mod add_purchase_order;
mod add_purchase_order_version;
mod add_purchase_order_version_revision;
pub(super) mod get_latest_revision_id;
pub(super) mod get_purchase_order;
pub(super) mod get_purchase_order_version;
pub(super) mod get_purchase_order_version_revision;
mod get_uid_from_alternate_id;
pub(super) mod list_alternate_ids_for_purchase_order;
pub(super) mod list_purchase_order_version_revisions;
pub(super) mod list_purchase_order_versions;
pub(super) mod list_purchase_orders;
mod remove_alternate_id;

pub(super) struct PurchaseOrderStoreOperations<'a, C> {
    conn: &'a C,
}

impl<'a, C> PurchaseOrderStoreOperations<'a, C>
where
    C: diesel::Connection,
{
    pub fn new(conn: &'a C) -> Self {
        PurchaseOrderStoreOperations { conn }
    }
}
