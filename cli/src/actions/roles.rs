/*
 * Copyright 2018-2020 Cargill Incorporated
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

use crate::actions::Paging;
use crate::error::CliError;
use crate::http::submit_batches;
use crate::transaction::pike_batch_builder;
use grid_sdk::{
    pike::addressing::PIKE_NAMESPACE,
    protocol::pike::payload::{
        Action, CreateRoleAction, DeleteRoleAction, PikePayloadBuilder, UpdateRoleAction,
    },
    protos::IntoProto,
};
use reqwest::Client;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GridRole {
    pub org_id: String,
    pub name: String,
    pub description: String,
    pub active: bool,
    pub permissions: Vec<String>,
    pub inherit_from: Vec<GridInheritFrom>,
    pub allowed_organizations: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GridRoleList {
    pub data: Vec<GridRole>,
    pub paging: Paging,
}

#[derive(Debug, Deserialize)]
pub struct GridInheritFrom {
    pub role_name: String,
    pub org_id: String,
}

/**
 * Prints info for a Grid Role
 *
 * role - Role to be printed
 */
pub fn display_role(role: &GridRole) {
    println!(
        "Organization ID: {:?}\nName: {:?}\nDescription: {:?}\nActive: {:?}\nPermissions: {:?}\nAllowed Orgs: {:?}\nInherit from:",
        role.org_id, role.name, role.description, role.active, role.permissions, role.allowed_organizations,
    );
    display_inherit_from(&role.inherit_from);
}

/**
 * Iterate through all fields of a inherited role and print the given value
 *
 * inherited - Inherited roles to be printed
 */
pub fn display_inherit_from(inherited: &[GridInheritFrom]) {
    inherited.iter().for_each(|i| {
        println!("\tOrg ID: {:?}\n\tName: {:?}", i.org_id, i.role_name,);
    });
}

pub fn do_create_role(
    url: &str,
    key: Option<String>,
    wait: u64,
    create_role: CreateRoleAction,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let payload = PikePayloadBuilder::new()
        .with_action(Action::CreateRole)
        .with_create_role(create_role)
        .build()
        .map_err(|err| CliError::UserError(format!("{}", err)))?;

    let batch_list = pike_batch_builder(key)
        .add_transaction(
            &payload.into_proto()?,
            &[PIKE_NAMESPACE.to_string()],
            &[PIKE_NAMESPACE.to_string()],
        )?
        .create_batch_list();

    submit_batches(url, wait, &batch_list, service_id.as_deref())
}

pub fn do_update_role(
    url: &str,
    key: Option<String>,
    wait: u64,
    update_role: UpdateRoleAction,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let payload = PikePayloadBuilder::new()
        .with_action(Action::UpdateRole)
        .with_update_role(update_role)
        .build()
        .map_err(|err| CliError::UserError(format!("{}", err)))?;

    let batch_list = pike_batch_builder(key)
        .add_transaction(
            &payload.into_proto()?,
            &[PIKE_NAMESPACE.to_string()],
            &[PIKE_NAMESPACE.to_string()],
        )?
        .create_batch_list();

    submit_batches(url, wait, &batch_list, service_id.as_deref())
}

pub fn do_delete_role(
    url: &str,
    key: Option<String>,
    wait: u64,
    delete_role: DeleteRoleAction,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let payload = PikePayloadBuilder::new()
        .with_action(Action::DeleteRole)
        .with_delete_role(delete_role)
        .build()
        .map_err(|err| CliError::UserError(format!("{}", err)))?;

    let batch_list = pike_batch_builder(key)
        .add_transaction(
            &payload.into_proto()?,
            &[PIKE_NAMESPACE.to_string()],
            &[PIKE_NAMESPACE.to_string()],
        )?
        .create_batch_list();

    submit_batches(url, wait, &batch_list, service_id.as_deref())
}

/**
 * Print a single role in state
 *
 * url - Url for the REST API
 * org_id - org ID for the role
 * name - role name
 */
pub fn do_show_role(
    url: &str,
    org_id: &str,
    name: &str,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let client = Client::new();
    let mut final_url = format!("{}/role/{}/{}", url, org_id, name);
    if let Some(service_id) = service_id {
        final_url = format!("{}?service_id={}", final_url, service_id);
    }

    let mut response = client.get(&final_url).send()?;

    if !response.status().is_success() {
        return Err(CliError::DaemonError(response.text()?));
    }

    let role = response.json::<GridRole>()?;
    display_role(&role);
    Ok(())
}
