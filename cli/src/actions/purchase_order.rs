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

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use grid_sdk::{
    client::purchase_order::{
        AlternateId, PurchaseOrder, PurchaseOrderClient, PurchaseOrderRevision,
        PurchaseOrderVersion,
    },
    data_validation::purchase_order::validate_alt_id_format,
    error::ClientError,
    pike::addressing::GRID_PIKE_NAMESPACE,
    protocol::purchase_order::payload::{
        Action, CreatePurchaseOrderPayload, CreateVersionPayload, PurchaseOrderPayloadBuilder,
        UpdatePurchaseOrderPayload, UpdateVersionPayload,
    },
    protos::IntoProto,
    purchase_order::addressing::GRID_PURCHASE_ORDER_NAMESPACE,
    purchase_order::store::{ListPOFilters, ListVersionFilters},
};

use chrono::{DateTime, NaiveDateTime, Utc};
use cylinder::Signer;
use rand::{distributions::Alphanumeric, Rng};
use sawtooth_sdk::messages::batch::BatchList;
use serde::Serialize;

use crate::actions;
use crate::error::CliError;
use crate::transaction::purchase_order_batch_builder;

fn create_po_batchlist(signer: Box<dyn Signer>, action: Action) -> Result<BatchList, CliError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

    let payload = PurchaseOrderPayloadBuilder::new()
        .with_action(action)
        .with_timestamp(timestamp)
        .build()
        .map_err(|err| CliError::UserError(format!("{}", err)))?;

    Ok(purchase_order_batch_builder(signer)
        .add_transaction(
            &payload.into_proto()?,
            &[
                GRID_PURCHASE_ORDER_NAMESPACE.to_string(),
                GRID_PIKE_NAMESPACE.to_string(),
            ],
            &[GRID_PURCHASE_ORDER_NAMESPACE.to_string()],
        )?
        .create_batch_list())
}

fn post_po_batch_action(
    client: &dyn PurchaseOrderClient,
    signer: Box<dyn Signer>,
    wait: u64,
    service_id: Option<&str>,
    action: Action,
) -> Result<(), CliError> {
    client
        .post_batches(wait, &create_po_batchlist(signer, action)?, service_id)
        .map_err(CliError::from)
}

pub fn do_create_purchase_order(
    client: &dyn PurchaseOrderClient,
    signer: Box<dyn Signer>,
    wait: u64,
    create_purchase_order: CreatePurchaseOrderPayload,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    post_po_batch_action(
        client,
        signer,
        wait,
        service_id,
        Action::CreatePo(create_purchase_order),
    )
}

pub fn do_update_purchase_order(
    client: &dyn PurchaseOrderClient,
    signer: Box<dyn Signer>,
    wait: u64,
    update_purchase_order: UpdatePurchaseOrderPayload,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    post_po_batch_action(
        client,
        signer,
        wait,
        service_id,
        Action::UpdatePo(update_purchase_order),
    )
}

pub fn do_create_version(
    client: &dyn PurchaseOrderClient,
    signer: Box<dyn Signer>,
    wait: u64,
    create_version: CreateVersionPayload,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    post_po_batch_action(
        client,
        signer,
        wait,
        service_id,
        Action::CreateVersion(create_version),
    )
}

pub fn do_fetch_purchase_order(
    client: &dyn PurchaseOrderClient,
    po_uid: &str,
    service_id: Option<&str>,
) -> Result<Option<PurchaseOrder>, CliError> {
    let po = client.get_purchase_order(po_uid.to_string(), service_id)?;

    Ok(po)
}

pub fn do_update_version(
    client: &dyn PurchaseOrderClient,
    signer: Box<dyn Signer>,
    wait: u64,
    update_version: UpdateVersionPayload,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    post_po_batch_action(
        client,
        signer,
        wait,
        service_id,
        Action::UpdateVersion(update_version),
    )
}

pub fn do_fetch_revisions(
    client: &dyn PurchaseOrderClient,
    po_uid: &str,
    version_id: &str,
    service_id: Option<&str>,
) -> Result<Box<dyn Iterator<Item = Result<PurchaseOrderRevision, ClientError>>>, CliError> {
    let revisions = client
        .list_purchase_order_revisions(po_uid.to_string(), version_id.to_string(), service_id)
        .map_err(CliError::from)?;

    Ok(revisions)
}

pub fn do_list_revisions(
    client: &dyn PurchaseOrderClient,
    po_uid: &str,
    version_id: &str,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let res = do_fetch_revisions(client, po_uid, version_id, service_id)?;
    let mut revisions: Box<dyn Iterator<Item = Result<PurchaseOrderRevisionCli, CliError>>> =
        Box::new(res.map(|rev_res: Result<PurchaseOrderRevision, _>| {
            let rev_cli_res: Result<PurchaseOrderRevisionCli, _> = rev_res
                .map(|rev| {
                    let rev_cli: PurchaseOrderRevisionCli = PurchaseOrderRevisionCli::from(rev);
                    rev_cli
                })
                .map_err(CliError::from);
            rev_cli_res
        }));

    print_formattable_list(&mut *revisions, None)?;
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
    let filters = ListVersionFilters {
        is_accepted: accepted_filter,
        is_draft: draft_filter,
    };

    let res: Box<dyn Iterator<Item = Result<PurchaseOrderVersion, ClientError>>> =
        client.list_purchase_order_versions(po_uid.to_string(), Some(filters), service_id)?;

    let mut versions: Box<dyn Iterator<Item = Result<PurchaseOrderVersionCli, CliError>>> =
        Box::new(res.map(|ver_res: Result<PurchaseOrderVersion, _>| {
            let ver_cli_res: Result<PurchaseOrderVersionCli, _> = ver_res
                .map(|ver| {
                    let ver_cli: PurchaseOrderVersionCli = PurchaseOrderVersionCli::from(ver);
                    ver_cli
                })
                .map_err(CliError::from);
            ver_cli_res
        }));

    print_formattable_list(&mut *versions, format)?;

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
    let revision_id = client
        .get_latest_revision_id(po_uid.to_string(), version_id.to_string(), service_id)
        .map_err(CliError::from)?;

    if let Some(revision_id) = revision_id {
        Ok(revision_id)
    } else {
        Ok(0)
    }
}

pub fn get_po_uid_from_alternate_id(
    client: &dyn PurchaseOrderClient,
    alternate_id: &str,
    service_id: Option<&str>,
) -> Result<String, CliError> {
    let po = client
        .get_purchase_order(alternate_id.to_string(), service_id)
        .map_err(CliError::from)?;

    if let Some(po) = po {
        Ok(po.purchase_order_uid)
    } else {
        Err(CliError::UserError(format!(
            "Could not find purchase order with alternate ID: {}",
            alternate_id
        )))
    }
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
    validate_alt_id_format(id)?;

    let split: Vec<&str> = id.split(':').collect();

    Ok(AlternateId::new(uid, split[0], split[1]))
}

pub fn do_show_purchase_order(
    client: &dyn PurchaseOrderClient,
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
    client: &dyn PurchaseOrderClient,
    filter: Option<ListPOFilters>,
    service_id: Option<String>,
    format: Option<&str>,
) -> Result<(), CliError> {
    let res: Box<dyn Iterator<Item = Result<PurchaseOrder, ClientError>>> =
        client.list_purchase_orders(filter, service_id.as_deref())?;
    let mut po_list: Box<dyn Iterator<Item = Result<PurchaseOrderCli, CliError>>> =
        Box::new(res.map(|po_res: Result<PurchaseOrder, _>| {
            let po_cli_res: Result<PurchaseOrderCli, _> = po_res
                .map(|po| {
                    let po_cli: PurchaseOrderCli = PurchaseOrderCli::from(po);
                    po_cli
                })
                .map_err(CliError::from);
            po_cli_res
        }));
    print_formattable_list(&mut *po_list, format)?;

    Ok(())
}

pub fn do_check_alternate_ids_are_unique(
    client: &dyn PurchaseOrderClient,
    alternate_ids: Vec<String>,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let filter = ListPOFilters {
        is_open: None,
        has_accepted_version: None,
        buyer_org_id: None,
        seller_org_id: None,
        alternate_ids: Some(alternate_ids.join(",")),
    };

    let res = client
        .list_purchase_orders(Some(filter), service_id)?
        .collect::<Result<Vec<PurchaseOrder>, _>>()?;

    if !res.is_empty() {
        let res_ids: Vec<AlternateId> = res
            .iter()
            .flat_map(|id| id.alternate_ids.to_vec())
            .collect();

        let duplicates: Vec<String> = alternate_ids
            .iter()
            .filter(|prop| {
                res_ids.iter().any(|existing| {
                    format!("{}:{}", existing.alternate_id_type, existing.alternate_id)
                        == prop.to_string()
                })
            })
            .map(String::from)
            .collect();

        return Err(CliError::UserError(format!(
            "Alternate IDs {:?} are already in use",
            duplicates
        )));
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct PurchaseOrderCli {
    buyer_org_id: String,
    seller_org_id: String,
    purchase_order_uid: String,
    workflow_id: String,
    workflow_state: String,
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
            workflow_id: d.workflow_id.to_string(),
            workflow_state: d.workflow_state.to_string(),
            is_closed: d.is_closed,
            accepted_version_id: match d.accepted_version_id.as_ref() {
                Some(a) => Some(a.to_string()),
                None => Some("none".to_string()),
            },
            versions: d
                .versions
                .iter()
                .map(PurchaseOrderVersionCli::from)
                .collect(),
            created_at: d.created_at,
        }
    }
}

impl From<PurchaseOrder> for PurchaseOrderCli {
    fn from(d: PurchaseOrder) -> Self {
        Self {
            buyer_org_id: d.buyer_org_id.to_string(),
            seller_org_id: d.seller_org_id.to_string(),
            purchase_order_uid: d.purchase_order_uid.to_string(),
            workflow_id: d.workflow_id.to_string(),
            workflow_state: d.workflow_state.to_string(),
            is_closed: d.is_closed,
            accepted_version_id: match d.accepted_version_id.as_ref() {
                Some(a) => Some(a.to_string()),
                None => Some("none".to_string()),
            },
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
    pub workflow_state: String,
    pub is_draft: bool,
    pub current_revision_id: u64,
    pub revisions: Vec<PurchaseOrderRevisionCli>,
}

impl From<&PurchaseOrderVersion> for PurchaseOrderVersionCli {
    fn from(d: &PurchaseOrderVersion) -> Self {
        Self {
            version_id: d.version_id.to_string(),
            workflow_state: d.workflow_state.to_string(),
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

impl From<PurchaseOrderVersion> for PurchaseOrderVersionCli {
    fn from(d: PurchaseOrderVersion) -> Self {
        Self {
            version_id: d.version_id.to_string(),
            workflow_state: d.workflow_state.to_string(),
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

impl From<PurchaseOrderRevision> for PurchaseOrderRevisionCli {
    fn from(d: PurchaseOrderRevision) -> Self {
        Self {
            revision_id: d.revision_id,
            order_xml_v3_4: d.order_xml_v3_4.to_string(),
            submitter: d.submitter.to_string(),
            created_at: d.created_at,
        }
    }
}

impl std::fmt::Display for PurchaseOrderCli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Purchase Order {}:", &self.purchase_order_uid)?;
        write!(f, "\n\t{:18}{}", "Buyer Org", &self.buyer_org_id)?;
        write!(f, "\n\t{:18}{}", "Seller Org", &self.seller_org_id)?;
        write!(f, "\n\t{:18}{}", "Workflow Name", &self.workflow_id)?;
        write!(f, "\n\t{:18}{}", "Workflow State", &self.workflow_state)?;
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
            match version {
                Some(v) => write!(f, "\n\n{}", v)?,
                None => write!(f, "\n\nNo accepted version")?,
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for PurchaseOrderVersionCli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Version {}:", &self.version_id)?;
        write!(f, "\n\t{:18}{}", "Workflow State", &self.workflow_state)?;
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

trait TableDisplay {
    fn header() -> Vec<&'static str>;
    fn details(&self) -> Vec<String>;
    fn widths() -> Vec<usize>;
}

impl TableDisplay for PurchaseOrderCli {
    fn header() -> Vec<&'static str> {
        vec![
            "BUYER", "SELLER", "UID", "WORKFLOW", "STATE", "ACCEPTED", "CLOSED",
        ]
    }

    fn details(&self) -> Vec<String> {
        vec![
            self.buyer_org_id.to_string(),
            self.seller_org_id.to_string(),
            self.purchase_order_uid.to_string(),
            self.workflow_id.to_string(),
            self.workflow_state.to_string(),
            match &self.accepted_version_id {
                Some(s) => s.to_string(),
                None => String::new(),
            },
            self.is_closed.to_string(),
        ]
    }

    fn widths() -> Vec<usize> {
        vec![12, 12, 13, 36, 18, 8, 6]
    }
}

impl TableDisplay for PurchaseOrderVersionCli {
    fn header() -> Vec<&'static str> {
        vec![
            "VERSION_ID",
            "WORKFLOW_STATE",
            "IS_DRAFT",
            "CURRENT_REVISION",
            "REVISIONS",
        ]
    }

    fn details(&self) -> Vec<String> {
        vec![
            self.version_id.to_string(),
            self.workflow_state.to_string(),
            self.is_draft.to_string(),
            self.current_revision_id.to_string(),
            self.revisions.len().to_string(),
        ]
    }

    fn widths() -> Vec<usize> {
        vec![10, 16, 8, 16, 9]
    }
}

impl TableDisplay for PurchaseOrderRevisionCli {
    fn header() -> Vec<&'static str> {
        vec!["REVISION_ID", "CREATED_AT", "SUBMITTER"]
    }

    fn details(&self) -> Vec<String> {
        vec![
            self.revision_id.to_string(),
            self.created_at.to_string(),
            self.submitter.to_string(),
        ]
    }

    fn widths() -> Vec<usize> {
        vec![11, 16, 9]
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

fn print_formattable_list<T: TableDisplay + std::fmt::Display + Serialize>(
    data: &mut dyn Iterator<Item = Result<T, CliError>>,
    format: Option<&str>,
) -> Result<(), CliError> {
    match format {
        Some("json") => {
            for row in &mut *data {
                match row {
                    Ok(object) => {
                        print_formattable(object, format)?;
                    }
                    Err(err) => {
                        println!("{}", err);
                        return Err(err);
                    }
                }
            }
        }
        Some("yaml") => {
            for row in &mut *data {
                match row {
                    Ok(object) => {
                        print_formattable(object, format)?;
                    }
                    Err(err) => {
                        println!("{}", err);
                        return Err(err);
                    }
                }
            }
        }
        Some("csv") => {
            println!("{}", str_join(T::header(), ","));
            for row in &mut *data {
                match row {
                    Ok(object) => {
                        println!("{}", str_join(object.details(), ","))
                    }
                    Err(err) => {
                        println!("{}", err);
                        return Err(err);
                    }
                }
            }
        }
        _ => {
            print_table(data)?;
        }
    }

    Ok(())
}

fn print_table<T: TableDisplay>(
    data: &mut dyn Iterator<Item = Result<T, CliError>>,
) -> Result<(), CliError> {
    // print header row
    let mut header_row = "".to_owned();
    for i in 0..T::header().len() {
        header_row += &format!("{:width$} ", T::header()[i], width = T::widths()[i]);
    }
    println!("{}", header_row);

    // print each row
    for row in &mut *data {
        match row {
            Ok(res) => {
                let mut print_row = "".to_owned();
                for i in 0..T::header().len() {
                    print_row += &format!("{:width$} ", res.details()[i], width = T::widths()[i]);
                }
                println!("{}", print_row);
            }
            Err(err) => {
                println!("{}", err);
                return Err(err);
            }
        }
    }

    Ok(())
}

fn str_join<T: ToString>(array: Vec<T>, delimiter: &str) -> String {
    array
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(delimiter)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use cylinder::{secp256k1::Secp256k1Context, Context};

    use grid_sdk::protocol::purchase_order::payload::CreatePurchaseOrderPayloadBuilder;

    #[test]
    fn test_create_po_batchlist() {
        let ctx = Secp256k1Context::new();
        let key = ctx.new_random_private_key();
        let signer = ctx.new_signer(key);

        let payload = CreatePurchaseOrderPayloadBuilder::new()
            .with_uid("test_uid".to_string())
            .with_buyer_org_id("buyer".to_string())
            .with_seller_org_id("seller".to_string())
            .with_workflow_state("created".to_string())
            .with_created_at(100)
            .with_workflow_id("workflow".to_string())
            .build()
            .expect("Could not build create po payload");

        let batch_list = create_po_batchlist(signer, Action::CreatePo(payload))
            .expect("post batch action failed");

        assert_eq!(batch_list.batches.len(), 1);
    }

    /// Tests the purchase order revisions are correctly displayed in the CLI
    /// in the format:
    /// Revision 4:
    ///     Created At       <datetime string>
    ///     Submitter        0200ef9ab9243baee...
    /// <Revision XML file>
    #[test]
    fn test_display_revision() {
        let display = "\
Revision 1:
	Created At        1970-05-23T21:21:18+00:00
	Submitter         0200ef9ab9243baee54f61a64d66aeb1d33bb063f16dfaa72e61886b9870c2b7ee

<tag></tag>";
        let revision = PurchaseOrderRevisionCli {
            revision_id: 1,
            order_xml_v3_4: "<tag></tag>".to_string(),
            submitter: "0200ef9ab9243baee54f61a64d66aeb1d33bb063f16dfaa72e61886b9870c2b7ee"
                .to_string(),
            created_at: 12345678,
        };

        assert_eq!(format!("{}", revision), display);
    }

    /// Tests the purchase order versions are correctly displayed in the CLI
    /// in the format:
    /// Version (1):
    ///     workflow_state  Editable
    ///     draft            false
    ///     Revisions        4
    ///     Current Revision 4
    ///
    /// Revision 4:
    ///     Created At       <datetime string>
    ///     Submitter        0200ef9ab9243baee...
    /// <Revision XML file>
    #[test]
    fn test_display_version() {
        let display = "\
Version 1:
	Workflow State    proposed
	Is Draft          true
	Revisions         1
	Current Revision  1

Revision 1:
	Created At        1970-05-23T21:21:18+00:00
	Submitter         0200ef9ab9243baee54f61a64d66aeb1d33bb063f16dfaa72e61886b9870c2b7ee

<tag></tag>";

        let revision = PurchaseOrderRevisionCli {
            revision_id: 1,
            order_xml_v3_4: "<tag></tag>".to_string(),
            submitter: "0200ef9ab9243baee54f61a64d66aeb1d33bb063f16dfaa72e61886b9870c2b7ee"
                .to_string(),
            created_at: 12345678,
        };

        let version = PurchaseOrderVersionCli {
            version_id: "1".to_string(),
            workflow_state: "proposed".to_string(),
            is_draft: true,
            current_revision_id: 1,
            revisions: vec![revision],
        };

        assert_eq!(format!("{}", version), display);
    }

    /// Tests the purchase orders are correctly displayed in the CLI in the
    /// format:
    /// Purchase Order PO-00000-0000:
    ///     Buyer Org        crgl (Cargill Incorporated)
    ///     Seller Org       crgl2 (Cargill 2)
    ///     Workflow state  Confirmed
    ///     Created At       <datetime string>
    ///     Closed           false
    ///
    /// Accepted Version (1):
    ///     workflow_state  Editable
    ///     draft            false
    ///     Revisions        4
    ///     Current Revision 4
    ///
    /// Revision 4:
    ///     Created At       <datetime string>
    ///     Submitter        0200ef9ab9243baee...
    /// <Revision XML file>
    #[test]
    fn test_display_po() {
        let display = "\
Purchase Order PO-00000-0000:
	Buyer Org         test
	Seller Org        test2
	Workflow Name     default
	Workflow State    created
	Created At        1970-05-23T21:21:17+00:00
	Closed            false

Version 1:
	Workflow State    proposed
	Is Draft          true
	Revisions         1
	Current Revision  1

Revision 1:
	Created At        1970-05-23T21:21:18+00:00
	Submitter         0200ef9ab9243baee54f61a64d66aeb1d33bb063f16dfaa72e61886b9870c2b7ee

<tag></tag>";
        let revision = PurchaseOrderRevisionCli {
            revision_id: 1,
            order_xml_v3_4: "<tag></tag>".to_string(),
            submitter: "0200ef9ab9243baee54f61a64d66aeb1d33bb063f16dfaa72e61886b9870c2b7ee"
                .to_string(),
            created_at: 12345678,
        };

        let version = PurchaseOrderVersionCli {
            version_id: "1".to_string(),
            workflow_state: "proposed".to_string(),
            is_draft: true,
            current_revision_id: 1,
            revisions: vec![revision],
        };

        let po = PurchaseOrderCli {
            buyer_org_id: "test".to_string(),
            seller_org_id: "test2".to_string(),
            purchase_order_uid: "PO-00000-0000".to_string(),
            workflow_id: "default".to_string(),
            workflow_state: "created".to_string(),
            is_closed: false,
            accepted_version_id: Some("1".to_string()),
            versions: vec![version],
            created_at: 12345677,
        };

        assert_eq!(format!("{}", po), display);
    }

    /// Tests that alternate IDs are parsed correctly
    #[test]
    fn test_alt_id_parsing() {
        let valid_id = "test:test_id";
        let invalid_ids = vec![
            "test_id",
            "test:test:test_id",
            "::test_id",
            ":test_id",
            "::",
            ":",
            ":test:test_id",
        ];
        let _valid_output = AlternateId::new("uid", "test", "test_id");

        assert!(matches!(
            make_alternate_id_from_str(&"uid", &valid_id).unwrap(),
            _valid_output
        ));
        for inv in invalid_ids {
            assert!(make_alternate_id_from_str(&"uid", &inv).is_err());
        }
    }
}

pub fn get_current_revision_for_version(
    client: &dyn PurchaseOrderClient,
    po_uid: &str,
    version: &PurchaseOrderVersion,
    service_id: Option<&str>,
) -> Result<PurchaseOrderRevision, CliError> {
    let revision = client.get_purchase_order_revision(
        po_uid.to_string(),
        version.version_id.to_string(),
        version.current_revision_id,
        service_id,
    )?;

    if let Some(revision) = revision {
        Ok(revision)
    } else {
        Err(CliError::UserError(format!(
            "Could not fetch revision {} for version {} of purchase order {}",
            version.current_revision_id, version.version_id, po_uid
        )))
    }
}

pub fn get_purchase_order_version(
    client: &dyn PurchaseOrderClient,
    po_uid: &str,
    version_id: &str,
    service_id: Option<&str>,
) -> Result<PurchaseOrderVersion, CliError> {
    let version = client.get_purchase_order_version(
        po_uid.to_string(),
        version_id.to_string(),
        service_id,
    )?;

    if let Some(version) = version {
        Ok(version)
    } else {
        Err(CliError::UserError(format!(
            "Could not fetch version {} for purchase order {}",
            version_id, po_uid
        )))
    }
}

/// Get the purchase order schema directory
pub fn get_order_schema_dir() -> PathBuf {
    actions::get_grid_xsd_dir().join("po/gs1/ecom")
}

/// Get the purchase order schema directory as a string
pub fn get_order_schema_dir_string() -> Result<String, CliError> {
    get_order_schema_dir()
        .into_os_string()
        .into_string()
        .map_err(|_| CliError::UserError("could not parse schema dir".to_string()))
}
