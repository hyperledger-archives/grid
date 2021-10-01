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
use grid_sdk::client::pike::{PikeClient, PikeOrganization};
use grid_sdk::{
    pike::addressing::GRID_PIKE_NAMESPACE,
    protocol::pike::payload::{
        Action, CreateOrganizationAction, PikePayloadBuilder, UpdateOrganizationAction,
    },
    protos::IntoProto,
};

use crate::error::CliError;
use crate::transaction::pike_batch_builder;

pub fn do_create_organization(
    client: Box<dyn PikeClient>,
    signer: Box<dyn Signer>,
    wait: u64,
    create_org: CreateOrganizationAction,
    service_id: Option<&str>,
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
            &[GRID_PIKE_NAMESPACE.to_string()],
            &[GRID_PIKE_NAMESPACE.to_string()],
        )?
        .create_batch_list();

    client.post_batches(wait, &batch_list, service_id)?;
    Ok(())
}

pub fn do_update_organization(
    client: Box<dyn PikeClient>,
    signer: Box<dyn Signer>,
    wait: u64,
    update_org: UpdateOrganizationAction,
    service_id: Option<&str>,
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
            &[GRID_PIKE_NAMESPACE.to_string()],
            &[GRID_PIKE_NAMESPACE.to_string()],
        )?
        .create_batch_list();

    client.post_batches(wait, &batch_list, service_id)?;
    Ok(())
}

pub fn do_list_organizations(
    client: Box<dyn PikeClient>,
    service_id: Option<&str>,
    format: &str,
    display_alternate_ids: bool,
) -> Result<(), CliError> {
    let orgs = client.list_organizations(service_id)?;

    list_organizations(orgs, format, display_alternate_ids);
    Ok(())
}

fn list_organizations(orgs: Vec<PikeOrganization>, format: &str, display_alternate_ids: bool) {
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
    client: Box<dyn PikeClient>,
    service_id: Option<&str>,
    org_id: &str,
) -> Result<(), CliError> {
    let org = client.get_organization(org_id.into(), service_id)?;

    print_organization(org);

    Ok(())
}

fn print_organization(org: PikeOrganization) {
    let mut display_string = format!("Organization ID: {}\nName: {}\n", &org.org_id, &org.name);
    if let Some(service_id) = &org.service_id {
        display_string += &format!("Service ID: {}\n", service_id);
    }
    display_string += "Locations:";
    let locations = if org.locations.is_empty() {
        " -\n".to_string()
    } else {
        org.locations
            .iter()
            .map(|locale| format!("\n\t{}", locale))
            .collect::<Vec<String>>()
            .join(",")
    };
    display_string += &locations;

    display_string += "Alternate IDs:";
    let ids = if org.alternate_ids.is_empty() {
        " -\n".to_string()
    } else {
        org.alternate_ids
            .iter()
            .map(|alt_id| format!("\n\t{}: {}", alt_id.id_type, alt_id.id))
            .collect::<Vec<String>>()
            .join(",")
    };
    display_string += &ids;

    display_string += "Metadata:";
    let metadata = if org.metadata.is_empty() {
        " -\n".to_string()
    } else {
        org.metadata
            .iter()
            .map(|data| format!("\n\t{}: {}", data.key, data.value))
            .collect::<Vec<String>>()
            .join(",")
    };
    display_string += &metadata;

    println!("{}", display_string)
}
