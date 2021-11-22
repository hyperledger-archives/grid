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
    context: &'a dyn TransactionContext,
}

impl<'a> PurchaseOrderState<'a> {
    pub fn new(context: &'a dyn TransactionContext) -> Self {
        Self { context }
    }

    pub fn get_purchase_order(&self, po_uid: &str) -> Result<Option<PurchaseOrder>, ApplyError> {
        let address = compute_purchase_order_address(po_uid);
        if let Some(packed) = self.context.get_state_entry(&address)? {
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

    pub fn set_purchase_order(
        &self,
        po_uid: &str,
        purchase_order: PurchaseOrder,
    ) -> Result<(), ApplyError> {
        let address = compute_purchase_order_address(po_uid);
        let mut purchase_orders: Vec<PurchaseOrder> =
            match self.context.get_state_entry(&address)? {
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

        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;

        Ok(())
    }

    pub fn get_purchase_order_version(
        &self,
        po_uid: &str,
        version_id: &str,
    ) -> Result<Option<PurchaseOrderVersion>, ApplyError> {
        let address = compute_purchase_order_address(po_uid);
        if let Some(packed) = self.context.get_state_entry(&address)? {
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

    pub fn set_purchase_order_version(
        &self,
        po_uid: &str,
        purchase_order_version: PurchaseOrderVersion,
    ) -> Result<(), ApplyError> {
        let address = compute_purchase_order_address(po_uid);
        let mut purchase_orders: Vec<PurchaseOrder> =
            match self.context.get_state_entry(&address)? {
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
        let vers_index = versions
            .iter()
            .position(|vers| vers.version_id() == purchase_order_version.version_id());
        match vers_index {
            Some(i) => {
                // If the `vers_index` is a Some value, then the version we are inserting is being
                // updated. Therefore, we need to remove the purchase order version that exists
                // in the list of versions at this index and replace it with the updated version
                versions.remove(i);
                versions.insert(i, purchase_order_version);
            }
            None => {
                // If the `vers_index` is a None value, the version does not exist in state so
                // the new version gets pushed to the end of the purchase order's `versions` list
                versions.push(purchase_order_version);
            }
        }
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
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    pub fn get_agent(&self, public_key: &str) -> Result<Option<Agent>, ApplyError> {
        let address = compute_agent_address(public_key);
        if let Some(packed) = self.context.get_state_entry(&address)? {
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

    pub fn get_organization(&self, id: &str) -> Result<Option<Organization>, ApplyError> {
        let address = compute_organization_address(id);
        if let Some(packed) = self.context.get_state_entry(&address)? {
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::collections::HashMap;

    use grid_sdk::protocol::purchase_order::state::{
        PurchaseOrder, PurchaseOrderBuilder, PurchaseOrderRevision, PurchaseOrderRevisionBuilder,
        PurchaseOrderVersion, PurchaseOrderVersionBuilder,
    };
    use sawtooth_sdk::processor::handler::{ContextError, TransactionContext};

    use crate::workflow::POWorkflow;

    const AGENT_PUB_KEY: &str = "test_agent_pub_key";

    const PO_UID: &str = "test_po_1";
    const PO_VERSION_ID_1: &str = "01";
    const PO_VERSION_ID_2: &str = "02";

    const ORG_1: &str = "test_org_1";
    const ORG_2: &str = "test_org_2";

    #[derive(Default, Debug)]
    /// A MockTransactionContext that can be used to test Purchase Order state
    struct MockTransactionContext {
        state: RefCell<HashMap<String, Vec<u8>>>,
    }

    impl TransactionContext for MockTransactionContext {
        fn get_state_entries(
            &self,
            addresses: &[String],
        ) -> Result<Vec<(String, Vec<u8>)>, ContextError> {
            let mut results = Vec::new();
            for addr in addresses {
                let data = match self.state.borrow().get(addr) {
                    Some(data) => data.clone(),
                    None => Vec::new(),
                };
                results.push((addr.to_string(), data));
            }
            Ok(results)
        }

        fn set_state_entries(&self, entries: Vec<(String, Vec<u8>)>) -> Result<(), ContextError> {
            for (addr, data) in entries {
                self.state.borrow_mut().insert(addr, data);
            }
            Ok(())
        }

        /// this is not needed for these tests
        fn delete_state_entries(&self, _addresses: &[String]) -> Result<Vec<String>, ContextError> {
            unimplemented!()
        }

        /// this is not needed for these tests
        fn add_receipt_data(&self, _data: &[u8]) -> Result<(), ContextError> {
            unimplemented!()
        }

        /// this is not needed for these tests
        fn add_event(
            &self,
            _event_type: String,
            _attributes: Vec<(String, String)>,
            _data: &[u8],
        ) -> Result<(), ContextError> {
            unimplemented!()
        }
    }

    #[test]
    /// Validate `PurchaseOrderState` returns correctly if the purchase order does not exist
    ///
    /// 1. Create the context and state for the test
    /// 2. Attempt to get a purchase order from state
    /// 3. Validate `None` is returned, as no purchase orders have been added to state
    fn test_get_po_does_not_exist() {
        let mut ctx = MockTransactionContext::default();
        let state = PurchaseOrderState::new(&mut ctx);

        let result = state.get_purchase_order("does_not_exist").unwrap();
        assert!(result.is_none());
    }

    #[test]
    /// Validate `PurchaseOrderState` returns correctly if the purchase order version does not exist
    ///
    /// 1. Create the context and state for the test
    /// 2. Create a purchase order, with one version, and set this object in state
    /// 3. Attempt to get a version that does not exist from the purchase order added to state
    /// 4. Validate `None` is returned, as the version does not exist
    fn test_get_po_version_does_not_exist() {
        let mut ctx = MockTransactionContext::default();
        let state = PurchaseOrderState::new(&mut ctx);

        let po = purchase_order_basic();
        if let Err(err) = state.set_purchase_order(PO_UID, po.clone()) {
            panic!("Unable to add Purchase Order to state: {:?}", err);
        }

        let result = state
            .get_purchase_order_version(PO_UID, "does_not_exist")
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    /// Validate `PurchaseOrderState` returns correctly if the purchase order version does exist
    ///
    /// 1. Create the context and state for the test
    /// 2. Create a purchase order, with two versions, and set this object in state
    /// 3. Attempt to get the first version from the purchase order added to state
    /// 4. Validate the expected version is returned successfully
    /// 5. Attempt to get the second version from the purchase order added to state
    /// 6. Validate the expected version is returned successfully
    /// 7. Retrieve the Purchase Order from state and validate both versions are present
    fn test_get_po_version_does_exist() {
        let mut ctx = MockTransactionContext::default();
        let state = PurchaseOrderState::new(&mut ctx);

        let po = purchase_order_multiple_versions();
        if let Err(err) = state.set_purchase_order(PO_UID, po.clone()) {
            panic!("Unable to add Purchase Order to state: {:?}", err);
        }

        let version_result = state
            .get_purchase_order_version(PO_UID, PO_VERSION_ID_1)
            .unwrap();
        assert_eq!(
            version_result,
            Some(purchase_order_version(PO_VERSION_ID_1))
        );

        let version_result = state
            .get_purchase_order_version(PO_UID, PO_VERSION_ID_2)
            .unwrap();
        assert_eq!(
            version_result,
            Some(purchase_order_version(PO_VERSION_ID_2))
        );

        let po_result = state
            .get_purchase_order(PO_UID)
            .expect("Unable to get purchase order from state")
            .unwrap();
        assert!(po_result
            .versions()
            .contains(&purchase_order_version(PO_VERSION_ID_1)));
        assert!(po_result
            .versions()
            .contains(&purchase_order_version(PO_VERSION_ID_2)));
    }

    #[test]
    /// Validate `PurchaseOrderState` returns correctly if the purchase order version does exist
    ///
    /// 1. Create the context and state for the test
    /// 2. Create a purchase order, with two versions, and set this object in state
    /// 3. Attempt to get the purchase order added in the previous step from state
    /// 4. Validate the expected purchase order is returned successfully
    fn test_get_po_does_exist() {
        let mut ctx = MockTransactionContext::default();
        let state = PurchaseOrderState::new(&mut ctx);

        let po = purchase_order_basic();
        if let Err(err) = state.set_purchase_order(PO_UID, po.clone()) {
            panic!("Unable to add Purchase Order to state: {:?}", err);
        }

        let result = state
            .get_purchase_order(PO_UID)
            .expect("Unable to get po from state");
        assert_eq!(result, Some(po));
    }

    fn purchase_order_basic() -> PurchaseOrder {
        PurchaseOrderBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_workflow_state("Issued".to_string())
            .with_created_at(1)
            .with_buyer_org_id(ORG_1.to_string())
            .with_seller_org_id(ORG_2.to_string())
            .with_versions(vec![purchase_order_version(PO_VERSION_ID_1)])
            .with_workflow_type(POWorkflow::SystemOfRecord.to_string())
            .with_is_closed(false)
            .build()
            .expect("Unable to build purchase order")
    }

    fn purchase_order_multiple_versions() -> PurchaseOrder {
        PurchaseOrderBuilder::new()
            .with_uid(PO_UID.to_string())
            .with_workflow_state("Issued".to_string())
            .with_created_at(2)
            .with_buyer_org_id(ORG_1.to_string())
            .with_seller_org_id(ORG_2.to_string())
            .with_workflow_type(POWorkflow::SystemOfRecord.to_string())
            .with_versions(vec![
                purchase_order_version(PO_VERSION_ID_1),
                purchase_order_version(PO_VERSION_ID_2),
            ])
            .with_is_closed(false)
            .build()
            .expect("Unable to build purchase order")
    }

    fn purchase_order_version(version_id: &str) -> PurchaseOrderVersion {
        PurchaseOrderVersionBuilder::new()
            .with_version_id(version_id.to_string())
            .with_workflow_state("Editable".to_string())
            .with_is_draft(true)
            .with_current_revision_id(1)
            .with_revisions(purchase_order_revision())
            .build()
            .expect("Unable to build first purchase order version")
    }

    fn purchase_order_revision() -> Vec<PurchaseOrderRevision> {
        vec![PurchaseOrderRevisionBuilder::new()
            .with_revision_id(1)
            .with_submitter(AGENT_PUB_KEY.to_string())
            .with_created_at(1)
            .with_order_xml_v3_4("xml_purchase_order".to_string())
            .build()
            .expect("Unable to build purchase order revision")]
    }
}
