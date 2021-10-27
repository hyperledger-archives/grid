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

use std::cmp;
use std::convert::TryInto;
use std::time::{SystemTime, UNIX_EPOCH};

use grid_sdk::{
    client::purchase_order::{
        AlternateId, PurchaseOrder, PurchaseOrderClient, PurchaseOrderRevision,
        PurchaseOrderVersion,
    },
    protocol::purchase_order::payload::{
        Action, CreatePurchaseOrderPayload, CreateVersionPayload, PurchaseOrderPayloadBuilder,
        UpdatePurchaseOrderPayload,
    },
    protos::IntoProto,
    purchase_order::addressing::GRID_PURCHASE_ORDER_NAMESPACE,
    purchase_order::store::ListVersionFilters,
};

use cylinder::Signer;
use rand::{distributions::Alphanumeric, Rng};

use crate::error::CliError;
use crate::transaction::purchase_order_batch_builder;

pub fn do_create_purchase_order(
    client: Box<dyn PurchaseOrderClient>,
    signer: Box<dyn Signer>,
    wait: u64,
    create_purchase_order: CreatePurchaseOrderPayload,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

    let payload = PurchaseOrderPayloadBuilder::new()
        .with_action(Action::CreatePo(create_purchase_order))
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

pub fn do_update_purchase_order(
    client: Box<dyn PurchaseOrderClient>,
    signer: Box<dyn Signer>,
    wait: u64,
    update_purchase_order: UpdatePurchaseOrderPayload,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

    let payload = PurchaseOrderPayloadBuilder::new()
        .with_action(Action::UpdatePo(update_purchase_order))
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

pub fn do_fetch_purchase_order(
    client: &dyn PurchaseOrderClient,
    po_uid: &str,
    service_id: Option<&str>,
) -> Result<Option<PurchaseOrder>, CliError> {
    let po = client.get_purchase_order(po_uid.to_string(), service_id)?;

    Ok(po)
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

pub fn do_fetch_alternate_ids(
    client: &dyn PurchaseOrderClient,
    po_uid: &str,
    service_id: Option<&str>,
) -> Result<Vec<AlternateId>, CliError> {
    let alternate_ids = client.list_alternate_ids(po_uid.to_string(), service_id)?;

    Ok(alternate_ids)
}

pub fn do_list_revisions(
    client: &dyn PurchaseOrderClient,
    po_uid: &str,
    version_id: &str,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let revisions = do_fetch_revisions(client, po_uid, version_id, service_id)?;

    display_revisions(revisions);
    Ok(())
}

pub fn do_show_revision(
    client: &dyn PurchaseOrderClient,
    po_uid: &str,
    version_id: &str,
    revision_num: u64,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let revision = client.get_purchase_order_revision(
        po_uid.to_string(),
        version_id.to_string(),
        revision_num,
        service_id,
    )?;

    if let Some(revision) = revision {
        display_revision(revision);
    } else {
        println!(
            "Could not find revision {}, for version {} for order {}",
            revision_num, version_id, po_uid
        );
    }
    Ok(())
}

pub fn do_list_versions(
    client: &dyn PurchaseOrderClient,
    po_uid: &str,
    accepted_filter: Option<bool>,
    draft_filter: Option<bool>,
    _format: &str,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let revision = client.get_purchase_order_revision(po_uid.to_string(), version_id.to_string(), revision_num, service_id)?;

    if let Some(revision) = revision {
        display_revision(revision);
    } else {
        println!("Could not find revision {}, for version {} for order {}", revision_num, version_id, po_uid);
    }
    Ok(())
}

pub fn do_list_versions(
    client: &dyn PurchaseOrderClient,
    po_uid: &str,
    accepted_filter: Option<bool>,
    draft_filter: Option<bool>,
    _format: &str,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let versions = get_versions(client, po_uid, accepted_filter, draft_filter, service_id)?;

    display_versions(versions);
    Ok(())
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

fn get_versions(
    client: &dyn PurchaseOrderClient,
    po_uid: &str,
    accepted_filter: Option<bool>,
    draft_filter: Option<bool>,
    service_id: Option<&str>,
) -> Result<Vec<PurchaseOrderVersion>, CliError> {
    let filters = ListVersionFilters {
        is_accepted: accepted_filter,
        is_draft: draft_filter,
    };

    let versions = client.list_purchase_order_versions(po_uid.to_string(), filters, service_id)?;

    Ok(versions)
}

pub fn generate_purchase_order_uid() -> String {
    format!(
        "PO-{}-{}",
        generate_random_base62_string(5),
        generate_random_base62_string(4),
    )
}

fn generate_random_base62_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(len)
        .collect()
}

pub fn make_alternate_id_from_str(uid: &str, id: &str) -> Result<AlternateId, CliError> {
    let split: Vec<&str> = id.split(':').collect();
    if split.len() != 2 {
        return Err(CliError::UserError(format!(
            "Could not parse alternate ID: {}",
            id
        )));
    }

    Ok(AlternateId::new(uid, split[0], split[1]))
}

fn display_revisions(revisions: Vec<PurchaseOrderRevision>) {
    let mut rows = vec![];
    revisions.iter().for_each(|rev| {
        let values = vec![
            rev.revision_id.to_string(),
            rev.created_at.to_string(),
            rev.submitter.to_string(),
        ];

        rows.push(values);
    });

    let column_names = vec!["REVISION_ID", "CREATED_AT", "SUBMITTER"];

    // Calculate max-widths for columns
    let mut widths: Vec<usize> = column_names.iter().map(|name| name.len()).collect();
    rows.iter().for_each(|row| {
        for i in 0..widths.len() {
            widths[i] = cmp::max(widths[i], row[i].to_string().len())
        }
    });

    // print header row
    let mut header_row = "".to_owned();
    for i in 0..column_names.len() {
        header_row += &format!("{:width$} ", column_names[i], width = widths[i]);
    }
    println!("{}", header_row);

    // print each row
    for row in rows {
        let mut print_row = "".to_owned();
        for i in 0..column_names.len() {
            print_row += &format!("{:width$} ", row[i], width = widths[i]);
        }
        println!("{}", print_row);
    }
}

fn display_revision(revision: PurchaseOrderRevision) {
    let column_names = vec!["REVISION_ID", "CREATED_AT", "SUBMITTER"];
    let table_values = vec![
        revision.revision_id.to_string(),
        revision.created_at.to_string(),
        revision.submitter,
    ];

    // Calculate max-widths for columns
    let mut widths: Vec<usize> = column_names.iter().map(|name| name.len()).collect();
    for i in 0..widths.len() {
        widths[i] = cmp::max(widths[i], table_values[i].to_string().len())
    }

    // print header row
    let mut header_row = "".to_owned();
    for i in 0..column_names.len() {
        header_row += &format!("{:width$} ", column_names[i], width = widths[i]);
    }
    println!("{}", header_row);

    // print revision
    let mut print_row = "".to_owned();
    for i in 0..column_names.len() {
        print_row += &format!("{:width$} ", table_values[i], width = widths[i]);
    }
    println!("{}", print_row);

    // print XML
    println!("ORDER XML");
    println!("{}", revision.order_xml_v3_4);
}

fn display_versions(versions: Vec<PurchaseOrderVersion>) {
    let mut rows = vec![];
    versions.iter().for_each(|version| {
        let values = vec![
            version.version_id.to_string(),
            version.workflow_status.to_string(),
            version.is_draft.to_string(),
            version.current_revision_id.to_string(),
            version.revisions.len().to_string(),
        ];

        rows.push(values);
    });

    let column_names = vec![
        "VERSION_ID",
        "WORKFLOW_STATUS",
        "IS_DRAFT",
        "CURRENT_REVISION",
        "REVISIONS",
    ];

    // Calculate max-widths for columns
    let mut widths: Vec<usize> = column_names.iter().map(|name| name.len()).collect();
    rows.iter().for_each(|row| {
        for i in 0..widths.len() {
            widths[i] = cmp::max(widths[i], row[i].to_string().len())
        }
    });

    // print header row
    let mut header_row = "".to_owned();
    for i in 0..column_names.len() {
        header_row += &format!("{:width$} ", column_names[i], width = widths[i]);
    }
    println!("{}", header_row);

    // print each row
    for row in rows {
        let mut print_row = "".to_owned();
        for i in 0..column_names.len() {
            print_row += &format!("{:width$} ", row[i], width = widths[i]);
        }
        println!("{}", print_row);
    }
}
