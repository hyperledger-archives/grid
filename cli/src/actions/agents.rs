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
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use grid_sdk::{
    pike::addressing::GRID_PIKE_NAMESPACE,
    protocol::pike::payload::{Action, CreateAgentAction, PikePayloadBuilder, UpdateAgentAction},
    protos::IntoProto,
};

use cylinder::Signer;
use reqwest::Client;
use serde::Deserialize;

use crate::actions::ListSlice;
use crate::error::CliError;
use crate::http::submit_batches;
use crate::transaction::pike_batch_builder;

#[derive(Debug, Deserialize)]
pub struct AgentSlice {
    pub public_key: String,
    pub org_id: String,
    pub active: bool,
    pub roles: Vec<String>,
    pub service_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

pub fn do_create_agent(
    url: &str,
    signer: Box<dyn Signer>,
    wait: u64,
    create_agent: CreateAgentAction,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

    let payload = PikePayloadBuilder::new()
        .with_action(Action::CreateAgent(create_agent))
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

    submit_batches(url, wait, &batch_list, service_id.as_deref())
}

pub fn do_update_agent(
    url: &str,
    signer: Box<dyn Signer>,
    wait: u64,
    update_agent: UpdateAgentAction,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

    let payload = PikePayloadBuilder::new()
        .with_action(Action::UpdateAgent(update_agent))
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

    submit_batches(url, wait, &batch_list, service_id.as_deref())
}

pub fn do_list_agents(
    url: &str,
    service_id: Option<String>,
    format: &str,
    line_per_role: bool,
) -> Result<(), CliError> {
    let client = Client::new();
    let mut final_url = format!("{}/agent", url);
    if let Some(service_id) = service_id {
        final_url = format!("{}?service_id={}", final_url, service_id);
    }

    let mut agents = Vec::new();

    loop {
        let mut response = client.get(&final_url).send()?;

        if !response.status().is_success() {
            return Err(CliError::DaemonError(response.text()?));
        }

        let mut agents_list = response.json::<ListSlice<AgentSlice>>()?;

        agents.append(&mut agents_list.data);

        if let Some(next) = agents_list.paging.next {
            final_url = format!("{}{}", url, next);
        } else {
            break;
        }
    }

    display_agents_info(&agents, format, line_per_role);
    Ok(())
}

pub fn do_show_agents(
    url: &str,
    public_key: &str,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let client = Client::new();
    let mut final_url = format!("{}/agent/{}", url, public_key);
    if let Some(service_id) = service_id {
        final_url = format!("{}?service_id={}", final_url, service_id);
    }

    let mut response = client.get(&final_url).send()?;

    if !response.status().is_success() {
        return Err(CliError::DaemonError(response.text()?));
    }

    let agent = response.json::<AgentSlice>()?;

    display_agent(&agent);

    Ok(())
}

pub fn display_agent(agent: &AgentSlice) {
    println!(
        "{}",
        vec![
            ("Public Key", &agent.public_key),
            ("Org ID", &agent.org_id),
            ("Active", &agent.active.to_string()),
            ("Roles", &agent.roles.join(", ")),
            (
                "Metadata",
                &agent
                    .metadata
                    .iter()
                    .map(|(key, value)| { format!("\n\t{}: {}", key, value) })
                    .collect::<Vec<String>>()
                    .join("")
            ),
        ]
        .iter()
        .map(|tuple| { format!("{}: {}", tuple.0, tuple.1) })
        .collect::<Vec<String>>()
        .join("\n")
    );
}

pub fn display_agents_info(agents: &[AgentSlice], format: &str, line_per_role: bool) {
    let column_names = if line_per_role {
        vec!["PUBLIC_KEY", "ORG_ID", "ACTIVE", "ROLE"]
    } else {
        vec!["PUBLIC_KEY", "ORG_ID", "ACTIVE", "ROLES"]
    };

    let row_values: Vec<Vec<String>> = if line_per_role {
        agents
            .iter()
            .flat_map(|agent| {
                agent.roles.iter().map(move |role| {
                    vec![
                        agent.public_key.to_string(),
                        agent.org_id.to_string(),
                        agent.active.to_string(),
                        role.to_string(),
                    ]
                })
            })
            .collect()
    } else {
        agents
            .iter()
            .map(|agent| {
                vec![
                    agent.public_key.to_string(),
                    agent.org_id.to_string(),
                    agent.active.to_string(),
                    agent.roles.join(", "),
                ]
            })
            .collect()
    };

    if format == "csv" {
        print_csv(column_names, row_values);
    } else {
        print_human_readable(column_names, row_values);
    }
}

fn print_csv(column_names: Vec<&str>, row_values: Vec<Vec<String>>) {
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

fn print_human_readable(column_names: Vec<&str>, row_values: Vec<Vec<String>>) {
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
