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

use std::convert::TryInto;
use std::time::{SystemTime, UNIX_EPOCH};

use grid_sdk::{
    client::purchase_order::{PurchaseOrderClient, PurchaseOrderRevision},
    protocol::purchase_order::payload::{
        Action, CreateVersionPayload, PurchaseOrderPayloadBuilder,
    },
    protos::IntoProto,
    purchase_order::addressing::GRID_PURCHASE_ORDER_NAMESPACE,
};

use cylinder::Signer;

use crate::error::CliError;
use crate::transaction::purchase_order_batch_builder;

pub fn do_create_version(
    client: &dyn PurchaseOrderClient,
    signer: Box<dyn Signer>,
    wait: u64,
    create_version: CreateVersionPayload,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

    let payload = PurchaseOrderPayloadBuilder::new()
        .with_action(Action::CreateVersion(create_version))
        .with_timestamp(timestamp)
        .build()
        .map_err(|err| CliError::UserError(format!("{}", err)))?;

    let batch_list = purchase_order_batch_builder(signer)
        .add_transaction(
            &payload.into_proto()?,
            &[GRID_PURCHASE_ORDER_NAMESPACE.to_string()],
            &[GRID_PURCHASE_ORDER_NAMESPACE.to_string()],
        )?
        .create_batch_list();

    client.post_batches(wait, &batch_list, service_id)?;
    Ok(())
}

pub fn do_fetch_revisions(
    client: &dyn PurchaseOrderClient,
    po_uid: &str,
    version_id: &str,
    service_id: Option<&str>,
) -> Result<Vec<PurchaseOrderRevision>, CliError> {
    let revisions = client.list_purchase_order_revisions(
        po_uid.to_string(),
        version_id.to_string(),
        service_id,
    )?;

    Ok(revisions)
}

pub fn get_latest_revision_id(
    client: &dyn PurchaseOrderClient,
    po_uid: &str,
    version_id: &str,
    service_id: Option<&str>,
) -> Result<i64, CliError> {
    let revisions = do_fetch_revisions(client, po_uid, version_id, service_id)?;

    let max = revisions.iter().max_by_key(|r| r.revision_id);

    if let Some(max) = max {
        Ok(max
            .revision_id
            .try_into()
            .map_err(|err| CliError::UserError(format!("{}", err)))?)
    } else {
        Ok(0)
    }
}
