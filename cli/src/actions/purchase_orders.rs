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
    purchase_order::store::{ListPOFilters, ListVersionFilters},
};

use chrono::{DateTime, NaiveDateTime, Utc};
use cylinder::Signer;
use rand::{distributions::Alphanumeric, Rng};
use serde::Serialize;

use crate::error::CliError;
use crate::transaction::purchase_order_batch_builder;

pub const GRID_ORDER_SCHEMA_DIR: &str = "GRID_ORDER_SCHEMA_DIR";

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
    let revisions = do_fetch_revisions(client, po_uid, version_id, service_id)?
        .iter()
        .map(PurchaseOrderRevisionCli::from)
        .collect::<Vec<PurchaseOrderRevisionCli>>();

    print_formattable_list(PurchaseOrderRevisionCliList(revisions), None)?;
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
        print_formattable(PurchaseOrderRevisionCli::from(&revision), None)?;
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
    format: Option<&str>,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let versions = get_versions(client, po_uid, accepted_filter, draft_filter, service_id)?
        .iter()
        .map(PurchaseOrderVersionCli::from)
        .collect::<Vec<PurchaseOrderVersionCli>>();

    print_formattable_list(PurchaseOrderVersionCliList(versions), format)?;
    Ok(())
}

pub fn do_show_version(
    client: &dyn PurchaseOrderClient,
    po_uid: &str,
    version_id: &str,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let version = client.get_purchase_order_version(
        po_uid.to_string(),
        version_id.to_string(),
        service_id,
    )?;

    if let Some(version) = version {
        print_formattable(PurchaseOrderVersionCli::from(&version), None)?;
    } else {
        println!("Could not find version {} for order {}", version_id, po_uid);
    }
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

    let versions =
        client.list_purchase_order_versions(po_uid.to_string(), Some(filters), service_id)?;

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

pub fn do_show_purchase_order(
    client: Box<dyn PurchaseOrderClient>,
    purchase_order_id: String,
    service_id: Option<String>,
    format: Option<&str>,
) -> Result<(), CliError> {
    let res = client.get_purchase_order(purchase_order_id, service_id.as_deref())?;
    match res {
        Some(purchase_order) => {
            print_formattable(PurchaseOrderCli::from(&purchase_order), format)?;
        }
        None => {
            println!("Purchase Order Not Found.");
        }
    }
    Ok(())
}

pub fn do_list_purchase_orders(
    client: Box<dyn PurchaseOrderClient>,
    filter: Option<ListPOFilters>,
    service_id: Option<String>,
    format: Option<&str>,
) -> Result<(), CliError> {
    let res = client.list_purchase_orders(filter, service_id.as_deref())?;
    let po_list = res
        .iter()
        .map(PurchaseOrderCli::from)
        .collect::<Vec<PurchaseOrderCli>>();
    print_formattable_list(PurchaseOrderCliList(po_list), format)?;

    Ok(())
}
#[derive(Debug, Serialize)]
struct PurchaseOrderCli {
    buyer_org_id: String,
    seller_org_id: String,
    purchase_order_uid: String,
    workflow_status: String,
    is_closed: bool,
    accepted_version_id: Option<String>,
    versions: Vec<PurchaseOrderVersionCli>,
    created_at: i64,
}

impl From<&PurchaseOrder> for PurchaseOrderCli {
    fn from(d: &PurchaseOrder) -> Self {
        Self {
            buyer_org_id: d.buyer_org_id.to_string(),
            seller_org_id: d.seller_org_id.to_string(),
            purchase_order_uid: d.purchase_order_uid.to_string(),
            workflow_status: d.workflow_status.to_string(),
            is_closed: d.is_closed,
            accepted_version_id: d.accepted_version_id.as_ref().map(String::from),
            versions: d
                .versions
                .iter()
                .map(PurchaseOrderVersionCli::from)
                .collect(),
            created_at: d.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
struct PurchaseOrderVersionCli {
    pub version_id: String,
    pub workflow_status: String,
    pub is_draft: bool,
    pub current_revision_id: u64,
    pub revisions: Vec<PurchaseOrderRevisionCli>,
}

impl From<&PurchaseOrderVersion> for PurchaseOrderVersionCli {
    fn from(d: &PurchaseOrderVersion) -> Self {
        Self {
            version_id: d.version_id.to_string(),
            workflow_status: d.workflow_status.to_string(),
            is_draft: d.is_draft,
            current_revision_id: d.current_revision_id,
            revisions: d
                .revisions
                .iter()
                .map(PurchaseOrderRevisionCli::from)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct PurchaseOrderRevisionCli {
    pub revision_id: u64,
    pub order_xml_v3_4: String,
    pub submitter: String,
    pub created_at: i64,
}

impl From<&PurchaseOrderRevision> for PurchaseOrderRevisionCli {
    fn from(d: &PurchaseOrderRevision) -> Self {
        Self {
            revision_id: d.revision_id,
            order_xml_v3_4: d.order_xml_v3_4.to_string(),
            submitter: d.submitter.to_string(),
            created_at: d.created_at,
        }
    }
}

#[derive(Serialize)]
pub struct PurchaseOrderCliList(Vec<PurchaseOrderCli>);

#[derive(Serialize)]
pub struct PurchaseOrderVersionCliList(Vec<PurchaseOrderVersionCli>);

#[derive(Serialize)]
pub struct PurchaseOrderRevisionCliList(Vec<PurchaseOrderRevisionCli>);

impl std::fmt::Display for PurchaseOrderCliList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for purchase_order in &self.0 {
            write!(f, "\n\n{}", purchase_order)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for PurchaseOrderVersionCliList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for version in &self.0 {
            write!(f, "\n\n{}", version)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for PurchaseOrderRevisionCliList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for revision in &self.0 {
            write!(f, "\n\n{}", revision)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for PurchaseOrderCli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Purchase Order {}:", &self.purchase_order_uid)?;
        write!(f, "\n\t{:18}{}", "Buyer Org", &self.buyer_org_id)?;
        write!(f, "\n\t{:18}{}", "Seller Org", &self.seller_org_id)?;
        write!(f, "\n\t{:18}{}", "Workflow Status", &self.workflow_status)?;
        write!(
            f,
            "\n\t{:18}{}",
            "Created At",
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.created_at, 0), Utc)
                .to_rfc3339()
        )?;
        write!(f, "\n\t{:18}{}", "Closed", &self.is_closed)?;
        if let Some(accepted_version_id) = &self.accepted_version_id {
            let versions = &self.versions;
            let version: Option<&PurchaseOrderVersionCli> = versions
                .iter()
                .find(|&ver| ver.version_id == *accepted_version_id);
            if let Some(version) = version {
                write!(f, "\n\n{}", version)?;
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for PurchaseOrderVersionCli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Version {}:", &self.version_id)?;
        write!(f, "\n\t{:18}{}", "Workflow Status", &self.workflow_status)?;
        write!(f, "\n\t{:18}{}", "Is Draft", &self.is_draft)?;
        let revisions = &self.revisions;
        write!(f, "\n\t{:18}{}", "Revisions", &revisions.len())?;
        write!(
            f,
            "\n\t{:18}{}",
            "Current Revision", &self.current_revision_id
        )?;
        let current_revision_id = &self.current_revision_id;
        let current_revision: Option<&PurchaseOrderRevisionCli> = revisions
            .iter()
            .find(|&rev| &rev.revision_id == current_revision_id);
        if let Some(current_revision) = current_revision {
            write!(f, "\n\n{}", current_revision)?;
        }

        Ok(())
    }
}

impl std::fmt::Display for PurchaseOrderRevisionCli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Revision {}:", &self.revision_id)?;
        write!(
            f,
            "\n\t{:18}{}",
            "Created At",
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.created_at, 0), Utc)
                .to_rfc3339()
        )?;
        write!(f, "\n\t{:18}{}", "Submitter", &self.submitter)?;
        write!(f, "\n\n{}", &self.order_xml_v3_4)?;

        Ok(())
    }
}

trait ListDisplay {
    fn header() -> Vec<&'static str>;
    fn details(&self) -> Vec<Vec<String>>;
}

impl ListDisplay for PurchaseOrderCliList {
    fn header() -> Vec<&'static str> {
        vec!["BUYER", "SELLER", "UID", "STATUS", "ACCEPTED", "CLOSED"]
    }

    fn details(&self) -> Vec<Vec<String>> {
        self.0
            .iter()
            .map(|po| {
                vec![
                    po.buyer_org_id.to_string(),
                    po.seller_org_id.to_string(),
                    po.purchase_order_uid.to_string(),
                    po.workflow_status.to_string(),
                    match &po.accepted_version_id {
                        Some(s) => s.to_string(),
                        None => String::new(),
                    },
                    po.is_closed.to_string(),
                ]
            })
            .collect::<Vec<_>>()
    }
}

impl ListDisplay for PurchaseOrderVersionCliList {
    fn header() -> Vec<&'static str> {
        vec![
            "VERSION_ID",
            "WORKFLOW_STATUS",
            "IS_DRAFT",
            "CURRENT_REVISION",
            "REVISIONS",
        ]
    }

    fn details(&self) -> Vec<Vec<String>> {
        self.0
            .iter()
            .map(|version| {
                vec![
                    version.version_id.to_string(),
                    version.workflow_status.to_string(),
                    version.is_draft.to_string(),
                    version.current_revision_id.to_string(),
                    version.revisions.len().to_string(),
                ]
            })
            .collect::<Vec<_>>()
    }
}

impl ListDisplay for PurchaseOrderRevisionCliList {
    fn header() -> Vec<&'static str> {
        vec!["REVISION_ID", "CREATED_AT", "SUBMITTER"]
    }

    fn details(&self) -> Vec<Vec<String>> {
        self.0
            .iter()
            .map(|rev| {
                vec![
                    rev.revision_id.to_string(),
                    rev.created_at.to_string(),
                    rev.submitter.to_string(),
                ]
            })
            .collect::<Vec<_>>()
    }
}

fn print_formattable<T: std::fmt::Display + Serialize>(
    object: T,
    format: Option<&str>,
) -> Result<(), CliError> {
    match format {
        Some("json") => {
            let formatted = serde_json::to_string(&object).map_err(|err| {
                CliError::ActionError(format!("Error formatting as JSON: {}", err))
            })?;
            println!("{}", formatted);
        }
        Some("yaml") => {
            let formatted = serde_yaml::to_string(&object).map_err(|err| {
                CliError::ActionError(format!("Error formatting as YAML: {}", err))
            })?;
            println!("{}", formatted);
        }
        _ => println!("{}", object),
    }
    Ok(())
}

fn print_formattable_list<T: std::fmt::Display + Serialize + ListDisplay>(
    object: T,
    format: Option<&str>,
) -> Result<(), CliError> {
    match format {
        Some("json") => {
            print_formattable(object, format)?;
        }
        Some("yaml") => {
            print_formattable(object, format)?;
        }
        Some("csv") => {
            let details = object
                .details()
                .iter()
                .map(|detail| str_join(detail.to_vec(), ","))
                .collect::<Vec<String>>()
                .join("\n");

            println!("{}\n{}", str_join(T::header(), ","), details);
        }
        _ => {
            print_human_readable_list(T::header(), object.details());
        }
    };
    Ok(())
}

fn str_join<T: ToString>(array: Vec<T>, delimiter: &str) -> String {
    array
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(delimiter)
}

fn print_human_readable_list(column_names: Vec<&str>, row_values: Vec<Vec<String>>) {
    // Calculate max-widths for columns
    let mut widths: Vec<usize> = column_names.iter().map(|name| name.len()).collect();
    row_values.iter().for_each(|row| {
        for i in 0..widths.len() {
            widths[i] = cmp::max(widths[i], row[i].len())
        }
    });

    // print header row
    let mut header_row = "".to_owned();
    for i in 0..column_names.len() {
        header_row += &format!("{:width$} ", column_names[i], width = widths[i]);
    }
    println!("{}", header_row);

    // print each row
    for row in row_values {
        let mut print_row = "".to_owned();
        for i in 0..column_names.len() {
            print_row += &format!("{:width$} ", row[i], width = widths[i]);
        }
        println!("{}", print_row);
    }
}
