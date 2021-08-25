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
    pike::addressing::{compute_agent_address, compute_organization_address},
    protocol::{
        pike::state::{Agent, AgentList, Organization, OrganizationList},
        purchase_order::state::{
            PurchaseOrder, PurchaseOrderList, PurchaseOrderListBuilder, PurchaseOrderVersion,
        },
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

    pub fn _get_purchase_order(&self, po_uid: &str) -> Result<Option<PurchaseOrder>, ApplyError> {
        let address = compute_purchase_order_address(po_uid);
        if let Some(packed) = self._context.get_state_entry(&address)? {
            let purchase_orders =
                PurchaseOrderList::from_bytes(packed.as_slice()).map_err(|_| {
                    ApplyError::InternalError("Cannot deserialize purchase order list".to_string())
                })?;
            Ok(purchase_orders
                .purchase_orders()
                .iter()
                .find(|p| p.uid() == po_uid)
                .cloned())
        } else {
            Ok(None)
        }
    }

    pub fn _set_purchase_order(
        &self,
        po_uid: &str,
        purchase_order: PurchaseOrder,
    ) -> Result<(), ApplyError> {
        let address = compute_purchase_order_address(po_uid);
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
            if po.uid() == po_uid {
                index = Some(i);
                break;
            }
        }

        if let Some(i) = index {
            purchase_orders.remove(i);
        }
        purchase_orders.push(purchase_order);
        purchase_orders.sort_by_key(|r| r.uid().to_string());
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

    pub fn _get_purchase_order_version(
        &self,
        po_uid: &str,
        version_id: &str,
    ) -> Result<Option<PurchaseOrderVersion>, ApplyError> {
        let address = compute_purchase_order_address(po_uid);
        if let Some(packed) = self._context.get_state_entry(&address)? {
            let purchase_orders =
                PurchaseOrderList::from_bytes(packed.as_slice()).map_err(|_| {
                    ApplyError::InternalError("Cannot deserialize purchase order list".to_string())
                })?;
            let po = purchase_orders
                .purchase_orders()
                .iter()
                .find(|p| p.uid() == po_uid)
                .cloned()
                .ok_or_else(|| {
                    ApplyError::InternalError(format!(
                        "Purchase order with UID {} does not exist",
                        po_uid
                    ))
                })?;
            Ok(po
                .versions()
                .iter()
                .find(|v| v.version_id() == version_id)
                .cloned())
        } else {
            Ok(None)
        }
    }

    pub fn _set_purchase_order_version(
        &self,
        po_uid: &str,
        purchase_order_version: PurchaseOrderVersion,
    ) -> Result<(), ApplyError> {
        let address = compute_purchase_order_address(po_uid);
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
            if po.uid() == po_uid {
                index = Some(i);
                break;
            }
        }
        // Get the original purchase order in state
        let orig_po = match index {
            Some(i) => Ok(purchase_orders.remove(i)),
            _ => Err(ApplyError::InvalidTransaction(format!(
                "Purchase Order with UID {} does not exist",
                po_uid
            ))),
        }?;
        // Add the `PurchaseOrderVersion` to the purchase order
        let mut versions = orig_po.versions().to_vec();
        versions.push(purchase_order_version);
        let purchase_order = orig_po
            .into_builder()
            .with_versions(versions)
            .build()
            .map_err(|err| {
                ApplyError::InvalidTransaction(format!("Cannot build purchase order: {:?}", err))
            })?;
        purchase_orders.push(purchase_order);
        purchase_orders.sort_by_key(|r| r.uid().to_string());
        // Add the updated purchase order to state
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

    pub fn _get_organization(&self, id: &str) -> Result<Option<Organization>, ApplyError> {
        let address = compute_organization_address(id);
        if let Some(packed) = self._context.get_state_entry(&address)? {
            let orgs = OrganizationList::from_bytes(packed.as_slice()).map_err(|err| {
                ApplyError::InternalError(format!(
                    "Cannot deserialize organization list: {:?}",
                    err
                ))
            })?;
            Ok(orgs
                .organizations()
                .iter()
                .find(|org| org.org_id() == id)
                .cloned())
        } else {
            Ok(None)
        }
    }
}
