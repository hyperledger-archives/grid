// Copyright 2021 Cargill Incorporated
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

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::ApplyError;
        use sabre_sdk::TransactionContext;
    } else {
        use sawtooth_sdk::processor::handler::TransactionContext;
        use sawtooth_sdk::processor::handler::ApplyError;
    }
}

use grid_sdk::protocol::{
    pike::state::{Agent, Organization},
    purchase_order::state::PurchaseOrder,
};

pub struct PurchaseOrderState<'a> {
    _context: &'a dyn TransactionContext,
}

impl<'a> PurchaseOrderState<'a> {
    pub fn new(context: &'a dyn TransactionContext) -> Self {
        Self { _context: context }
    }

    pub fn get_purchase_order(&self, _po_uuid: &str) -> Result<Option<PurchaseOrder>, ApplyError> {
        unimplemented!();
    }

    pub fn set_purchase_order(
        &self,
        _po_uuid: &str,
        _purchase_order: PurchaseOrder,
    ) -> Result<(), ApplyError> {
        unimplemented!();
    }

    pub fn get_agent(&self, _public_key: &str) -> Result<Option<Agent>, ApplyError> {
        unimplemented!();
    }

    pub fn get_organization(&self, _id: &str) -> Result<Option<Organization>, ApplyError> {
        unimplemented!();
    }
}
