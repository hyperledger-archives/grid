/*
 * Copyright 2018-2021 Cargill Incorporated
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

use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::CliError;
use crate::transaction::pike_batch_builder;
use cylinder::Signer;
use grid_sdk::{
    client::pike::{InheritFrom, PikeClient, PikeRole},
    pike::addressing::GRID_PIKE_NAMESPACE,
    protocol::pike::payload::{
        Action, CreateRoleAction, DeleteRoleAction, PikePayloadBuilder, UpdateRoleAction,
    },
    protos::IntoProto,
};
use std::cmp::max;

/**
 * Prints info for a Grid Role
 *
 * role - Role to be printed
 */
pub fn display_role(role: &PikeRole) {
    println!(
        "Organization ID: {:?}\nName: {:?}\nDescription: {:?}\nActive: {:?}\nPermissions: {:?}\n\
        Allowed Orgs: {:?}\nInherit from:",
        role.org_id,
        role.name,
        role.description,
        role.active,
        role.permissions,
        role.allowed_organizations,
    );
    display_inherit_from(&role.inherit_from);
}

/**
 * Prints general info for a list Grid Roles
 *
 * roles - Roles to be printed
 */
pub fn display_roles_info(roles: &[PikeRole]) {
    let mut width_org_id = "Organization ID".len();
    let mut width_role_name = "Role Name".len();
    let width_active = "Active".len();
    let mut width_description = "Description".len();

    roles.iter().for_each(|role| {
        width_org_id = max(width_org_id, role.org_id.len());
        width_role_name = max(width_role_name, role.name.len());
        width_description = max(width_description, role.description.len());
    });

    println!(
        "{:<width_org_id$} {:<width_role_name$} {:<width_active$} {:<width_description$}",
        "Organization ID",
        "Role Name",
        "Active",
        "Description",
        width_org_id = width_org_id,
        width_role_name = width_role_name,
        width_active = width_active,
        width_description = width_description
    );

    roles.iter().for_each(|role| {
        println!(
            "{:<width_org_id$} {:<width_role_name$} {:<width_active$} {:<width_description$}",
            role.org_id,
            role.name,
            role.active,
            role.description,
            width_org_id = width_org_id,
            width_role_name = width_role_name,
            width_active = width_active,
            width_description = width_description
        )
    });
}

/**
 * Iterate through all fields of a inherited role and print the given value
 *
 * inherited - Inherited roles to be printed
 */
pub fn display_inherit_from(inherited: &[InheritFrom]) {
    inherited.iter().for_each(|i| {
        println!("\tOrg ID: {:?}\n\tName: {:?}", i.org_id, i.role_name,);
    });
}

/**
 * Create a new role
 *
 * url - Url for the REST API
 * signer - Signer for the agent submitting the transaction
 * wait - Time in seconds to wait for commit
 * create_role - Action to create a role
 * service_id - ID of the service to delete a role from
 */
pub fn do_create_role(
    client: Box<dyn PikeClient>,
    signer: Box<dyn Signer>,
    wait: u64,
    create_role: CreateRoleAction,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

    let payload = PikePayloadBuilder::new()
        .with_action(Action::CreateRole(create_role))
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

/**
 * Update an existing role
 *
 * url - Url for the REST API
 * signer - Signer for the agent submitting the transaction
 * wait - Time in seconds to wait for commit
 * update_role - Action to update a role
 * service_id - ID of the service to delete a role from
 */
pub fn do_update_role(
    client: Box<dyn PikeClient>,
    signer: Box<dyn Signer>,
    wait: u64,
    update_role: UpdateRoleAction,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

    let payload = PikePayloadBuilder::new()
        .with_action(Action::UpdateRole(update_role))
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

/**
 * Delete an existing role
 *
 * url - Url for the REST API
 * signer - Signer for the agent submitting the transaction
 * wait - Time in seconds to wait for commit
 * delete_role - Action to delete a role
 * service_id - ID of the service to delete a role from
 */
pub fn do_delete_role(
    client: Box<dyn PikeClient>,
    signer: Box<dyn Signer>,
    wait: u64,
    delete_role: DeleteRoleAction,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

    let payload = PikePayloadBuilder::new()
        .with_action(Action::DeleteRole(delete_role))
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

/**
 * Print a single role in state
 *
 * url - Url for the REST API
 * org_id - Org ID for the role
 * name - Role name
 * service_id - ID of the service to show the role from
 */
pub fn do_show_role(
    client: Box<dyn PikeClient>,
    org_id: String,
    name: String,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let role = client.get_role(org_id, name, service_id)?;
    display_role(&role);
    Ok(())
}

/**
 * Print all roles in state for an organization
 *
 * Client - RoleClient for the REST API
 * org_id - Org ID for the roles
 * service_id - ID of the service to list roles from
 */
pub fn do_list_roles(
    client: Box<dyn PikeClient>,
    org_id: String,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let roles = client.list_roles(org_id, service_id)?;
    display_roles_info(&roles);
    Ok(())
}
