/*
 * Copyright 2019-2021 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use std::cmp;
use std::time::{SystemTime, UNIX_EPOCH};

use cylinder::Signer;
use grid_sdk::{
    pike::addressing::PIKE_NAMESPACE,
    protocol::pike::payload::{
        Action, CreateOrganizationAction, PikePayloadBuilder, UpdateOrganizationAction,
    },
    protos::IntoProto,
};
use reqwest::Client;
use serde::Deserialize;

use crate::actions::Paging;
use crate::error::CliError;
use crate::http::submit_batches;
use crate::transaction::pike_batch_builder;

#[derive(Debug, Deserialize)]
pub struct AlternateIdSlice {
    pub id_type: String,
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct OrganizationMetadataSlice {
    pub key: String,
    pub value: String,
    pub service_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OrganizationSlice {
    pub org_id: String,
    pub name: String,
    pub locations: Vec<String>,
    pub alternate_ids: Vec<AlternateIdSlice>,
    pub metadata: Vec<OrganizationMetadataSlice>,
    pub service_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OrganizationListSlice {
    pub data: Vec<OrganizationSlice>,
    pub paging: Paging,
}

pub fn do_create_organization(
    url: &str,
    signer: Box<dyn Signer>,
    wait: u64,
    create_org: CreateOrganizationAction,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

    let payload = PikePayloadBuilder::new()
        .with_action(Action::CreateOrganization(create_org))
        .with_timestamp(timestamp)
        .build()
        .map_err(|err| CliError::UserError(format!("{}", err)))?;

    let batch_list = pike_batch_builder(signer)
        .add_transaction(
            &payload.into_proto()?,
            &[PIKE_NAMESPACE.to_string()],
            &[PIKE_NAMESPACE.to_string()],
        )?
        .create_batch_list();

    submit_batches(url, wait, &batch_list, service_id.as_deref())
}

pub fn do_update_organization(
    url: &str,
    signer: Box<dyn Signer>,
    wait: u64,
    update_org: UpdateOrganizationAction,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

    let payload = PikePayloadBuilder::new()
        .with_action(Action::UpdateOrganization(update_org))
        .with_timestamp(timestamp)
        .build()
        .map_err(|err| CliError::UserError(format!("{}", err)))?;

    let batch_list = pike_batch_builder(signer)
        .add_transaction(
            &payload.into_proto()?,
            &[PIKE_NAMESPACE.to_string()],
            &[PIKE_NAMESPACE.to_string()],
        )?
        .create_batch_list();

    submit_batches(url, wait, &batch_list, service_id.as_deref())
}

pub fn do_list_organizations(
    url: &str,
    service_id: Option<String>,
    format: &str,
    display_alternate_ids: bool,
) -> Result<(), CliError> {
    let client = Client::new();
    let mut final_url = format!("{}/organization", url);
    if let Some(service_id) = service_id {
        final_url = format!("{}?service_id={}", final_url, service_id);
    }

    let mut orgs = Vec::new();

    loop {
        let mut response = client.get(&final_url).send()?;

        if !response.status().is_success() {
            return Err(CliError::DaemonError(response.text()?));
        }

        let mut orgs_list = response.json::<OrganizationListSlice>()?;
        orgs.append(&mut orgs_list.data);

        if let Some(next) = orgs_list.paging.next {
            final_url = format!("{}{}", url, next);
        } else {
            break;
        }
    }

    list_organizations(orgs, format, display_alternate_ids);
    Ok(())
}

fn list_organizations(orgs: Vec<OrganizationSlice>, format: &str, display_alternate_ids: bool) {
    let mut headers = vec![
        "ORG_ID".to_string(),
        "NAME".to_string(),
        "LOCATIONS".to_string(),
    ];
    if display_alternate_ids {
        headers.push("ALTERNATE_IDS".to_string());
    }
    let mut rows = vec![];
    orgs.iter().for_each(|org| {
        let mut values = vec![
            org.org_id.to_string(),
            org.name.to_string(),
            org.locations.join(", "),
        ];
        if display_alternate_ids {
            values.push(
                org.alternate_ids
                    .iter()
                    .map(|id| format!("{}:{}", id.id_type, id.id))
                    .collect::<Vec<String>>()
                    .join(", "),
            );
        }
        rows.push(values);
    });
    if format == "csv" {
        print_csv(headers, rows);
    } else {
        print_human_readable(headers, rows);
    }
}

fn print_csv(column_names: Vec<String>, row_values: Vec<Vec<String>>) {
    // print header row
    let mut header_row = "".to_owned();
    for column in &column_names {
        header_row += &format!("\"{}\",", column);
    }
    header_row.pop();
    println!("{}", header_row);

    // print each row
    for row in row_values {
        let mut print_row = "".to_owned();
        for cell in row.iter().take(column_names.len()) {
            print_row += &format!("\"{}\",", cell);
        }
        print_row.pop();
        println!("{}", print_row);
    }
}

fn print_human_readable(column_names: Vec<String>, row_values: Vec<Vec<String>>) {
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

pub fn do_show_organization(
    url: &str,
    service_id: Option<String>,
    org_id: &str,
) -> Result<(), CliError> {
    let client = Client::new();
    let mut final_url = format!("{}/organization/{}", url, org_id);
    if let Some(service_id) = service_id {
        final_url = format!("{}?service_id={}", final_url, service_id);
    }

    let mut response = client.get(&final_url).send()?;

    if !response.status().is_success() {
        return Err(CliError::DaemonError(response.text()?));
    }

    let org = response.json::<OrganizationSlice>()?;

    println!("{}", org);

    Ok(())
}

impl std::fmt::Display for OrganizationSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut display_string =
            format!("Organization ID: {}\nName: {}\n", &self.org_id, &self.name);
        if let Some(service_id) = &self.service_id {
            display_string += &format!("Service ID: {}\n", service_id);
        }
        display_string += "Locations:";
        let locations = if self.locations.is_empty() {
            " -\n".to_string()
        } else {
            self.locations
                .iter()
                .map(|locale| format!("\n\t{}", locale))
                .collect::<Vec<String>>()
                .join(",")
        };
        display_string += &locations;

        display_string += "Alternate IDs:";
        let ids = if self.alternate_ids.is_empty() {
            " -\n".to_string()
        } else {
            self.alternate_ids
                .iter()
                .map(|alt_id| format!("\n\t{}: {}", alt_id.id_type, alt_id.id))
                .collect::<Vec<String>>()
                .join(",")
        };
        display_string += &ids;

        display_string += "Metadata:";
        let metadata = if self.metadata.is_empty() {
            " -\n".to_string()
        } else {
            self.metadata
                .iter()
                .map(|data| format!("\n\t{}: {}", data.key, data.value))
                .collect::<Vec<String>>()
                .join(",")
        };
        display_string += &metadata;

        write!(f, "{}", display_string)
    }
}
