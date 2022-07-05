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

use grid_sdk::{
    client::pike::{PikeAgent, PikeClient},
    pike::addressing::GRID_PIKE_NAMESPACE,
    protocol::pike::payload::{Action, CreateAgentAction, PikePayloadBuilder, UpdateAgentAction},
    protos::IntoProto,
};

use cylinder::Signer;

use crate::error::CliError;
use crate::transaction::pike_batch_builder;

pub fn do_create_agent(
    client: Box<dyn PikeClient>,
    signer: Box<dyn Signer>,
    wait: u64,
    create_agent: CreateAgentAction,
    service_id: Option<&str>,
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

    client.post_batches(wait, &batch_list, service_id)?;
    Ok(())
}

pub fn do_update_agent(
    client: Box<dyn PikeClient>,
    signer: Box<dyn Signer>,
    wait: u64,
    update_agent: UpdateAgentAction,
    service_id: Option<&str>,
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

    client.post_batches(wait, &batch_list, service_id)?;
    Ok(())
}

pub fn do_list_agents(
    client: Box<dyn PikeClient>,
    service_id: Option<&str>,
    format: &str,
    line_per_role: bool,
) -> Result<(), CliError> {
    let agents = client.list_agents(service_id)?;

    display_agents_info(&agents, format, line_per_role);
    Ok(())
}

pub fn do_show_agents(
    client: Box<dyn PikeClient>,
    public_key: &str,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let agent = client.get_agent(public_key.into(), service_id)?;

    display_agent(&agent);
    Ok(())
}

pub fn display_agent(agent: &PikeAgent) {
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

pub fn display_agents_info(agents: &[PikeAgent], format: &str, line_per_role: bool) {
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
        let csv_cell = format!("\"{}\",", column);
        header_row.push_str(&csv_cell);
    }
    header_row.pop();
    println!("{}", header_row);

    // print each row
    for row in row_values {
        let mut print_row = "".to_owned();
        for cell in row.iter().take(column_names.len()) {
            let csv_cell = format!("\"{}\",", cell);
            print_row.push_str(&csv_cell);
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
        header_row.push_str(column_names[i]);
        header_row.push_str(&" ".repeat(widths[i]));
    }
    println!("{}", header_row);

    // print each row
    for row in row_values {
        let mut print_row = "".to_owned();
        for i in 0..column_names.len() {
            print_row.push_str(&row[i]);
            print_row.push_str(&" ".repeat(widths[i]));
        }
        println!("{}", print_row);
    }
}
