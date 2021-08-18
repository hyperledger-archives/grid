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

use grid_sdk::{
    pike::addressing::compute_agent_address,
    protocol::{
        pike::state::{Agent, AgentList, Organization},
        purchase_order::state::{PurchaseOrder, PurchaseOrderList, PurchaseOrderListBuilder},
    },
    protos::{FromBytes, IntoBytes},
    purchase_order::addressing::compute_purchase_order_address,
};

pub struct PurchaseOrderState<'a> {
    _context: &'a dyn TransactionContext,
}

impl<'a> PurchaseOrderState<'a> {
    pub fn new(context: &'a dyn TransactionContext) -> Self {
        Self { _context: context }
    }

    pub fn _get_purchase_order(&self, po_uuid: &str) -> Result<Option<PurchaseOrder>, ApplyError> {
        let address = compute_purchase_order_address(po_uuid);
        if let Some(packed) = self._context.get_state_entry(&address)? {
            let purchase_orders =
                PurchaseOrderList::from_bytes(packed.as_slice()).map_err(|_| {
                    ApplyError::InternalError("Cannot deserialize purchase order list".to_string())
                })?;
            Ok(purchase_orders
                .purchase_orders()
                .iter()
                .find(|p| p.uuid() == po_uuid)
                .cloned())
        } else {
            Ok(None)
        }
    }

    pub fn _set_purchase_order(
        &self,
        po_uuid: &str,
        purchase_order: PurchaseOrder,
    ) -> Result<(), ApplyError> {
        let address = compute_purchase_order_address(po_uuid);
        let mut purchase_orders: Vec<PurchaseOrder> =
            match self._context.get_state_entry(&address)? {
                Some(packed) => PurchaseOrderList::from_bytes(packed.as_slice())
                    .map_err(|err| {
                        ApplyError::InternalError(format!(
                            "Cannot deserialize purchase order list: {:?}",
                            err
                        ))
                    })?
                    .purchase_orders()
                    .to_vec(),
                None => vec![],
            };

        let mut index = None;
        for (i, po) in purchase_orders.iter().enumerate() {
            if po.uuid() == po_uuid {
                index = Some(i);
                break;
            }
        }

        if let Some(i) = index {
            purchase_orders.remove(i);
        }
        purchase_orders.push(purchase_order);
        purchase_orders.sort_by_key(|r| r.uuid().to_string());
        let po_list = PurchaseOrderListBuilder::new()
            .with_purchase_orders(purchase_orders)
            .build()
            .map_err(|err| {
                ApplyError::InvalidTransaction(format!(
                    "Cannot build purchase order list: {:?}",
                    err
                ))
            })?;
        let serialized = po_list.into_bytes().map_err(|err| {
            ApplyError::InvalidTransaction(format!(
                "Cannot serialize purchase order list: {:?}",
                err
            ))
        })?;

        self._context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;

        Ok(())
    }

    pub fn _get_agent(&self, public_key: &str) -> Result<Option<Agent>, ApplyError> {
        let address = compute_agent_address(public_key);
        if let Some(packed) = self._context.get_state_entry(&address)? {
            let agents = AgentList::from_bytes(packed.as_slice()).map_err(|err| {
                ApplyError::InternalError(format!("Cannot deserialize agent list: {:?}", err))
            })?;
            Ok(agents
                .agents()
                .iter()
                .find(|agent| agent.public_key() == public_key)
                .cloned())
        } else {
            Ok(None)
        }
    }

    pub fn _get_organization(&self, _id: &str) -> Result<Option<Organization>, ApplyError> {
        unimplemented!();
    }
}
