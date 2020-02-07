/*
 * Copyright 2019 Cargill Incorporated
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

use crate::error::CliError;
use crate::http::submit_batches;
use crate::transaction::{pike_batch_builder, PIKE_NAMESPACE};
use grid_sdk::{
    protocol::pike::payload::{Action, CreateAgentAction, PikePayloadBuilder, UpdateAgentAction},
    protos::IntoProto,
};

pub fn do_create_agent(
    url: &str,
    key: Option<String>,
    wait: u64,
    create_agent: CreateAgentAction,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let payload = PikePayloadBuilder::new()
        .with_action(Action::CreateAgent)
        .with_create_agent(create_agent)
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

pub fn do_update_agent(
    url: &str,
    key: Option<String>,
    wait: u64,
    update_agent: UpdateAgentAction,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let payload = PikePayloadBuilder::new()
        .with_action(Action::UpdateAgent)
        .with_update_agent(update_agent)
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
