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

use crate::CliError;
use protobuf::Message;
use reqwest::Client;
use sawtooth_sdk::messages::batch::BatchList;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Instant;

pub fn submit_batches(
    url: &str,
    mut wait: u64,
    batch_list: &BatchList,
    service_id: Option<&str>,
) -> Result<(), CliError> {
    let bytes = batch_list.write_to_bytes()?;

    let client = Client::new();

    let mut final_url = format!("{}/batches", url);
    if let Some(service_id) = service_id {
        final_url = format!("{}?service_id={}", final_url, service_id);
    }
    debug!("url {}", final_url);
    let mut response = client
        .post(&final_url)
        .header("GridProtocolVersion", "1")
        .body(bytes)
        .send()?;

    if !response.status().is_success() {
        return Err(CliError::DaemonError(response.text()?));
    }

    let batch_link = response.json::<BatchStatusLink>()?;

    let params: Vec<&str> = batch_link.link.split('?').collect();

    let id_param: Vec<&str> = params[1].split('=').collect();

    let id = id_param[1];

    info!("Submitted batch: {}", id);

    while wait > 0 {
        let time = Instant::now();

        let url = if let Some(service_id) = service_id {
            format!(
                "{}&wait={}&service_id={}",
                batch_link.link, wait, service_id
            )
        } else {
            format!("{}&wait={}", batch_link.link, wait)
        };

        let mut response = client.get(&url).send()?;

        if !response.status().is_success() {
            return Err(CliError::DaemonError(response.text()?));
        }

        let batch_status = response.json::<BatchStatusResponse>()?;

        for t in &batch_status.data {
            if t.status == "Invalid" {
                for i in &t.invalid_transactions {
                    error!(
                        "Error: {}",
                        i.get("message")
                            .unwrap_or(&"Batch contained invalid transactions".to_string())
                    );
                }
            }
        }

        if batch_status.data.iter().all(|d| d.status == "Valid") {
            info!("Batch and transaction structure was valid. Batch queued.");
        }

        if batch_status.data.iter().all(|x| x.status != "PENDING") {
            break;
        }

        wait -= time.elapsed().as_secs()
    }

    Ok(())
}

// Server Responses

#[derive(Deserialize, Debug)]
pub struct BatchStatusLink {
    pub link: String,
}

#[derive(Deserialize, Debug)]
struct BatchStatusResponse {
    pub data: Vec<BatchStatus>,
    pub link: String,
}

#[derive(Deserialize, Debug)]
struct BatchStatus {
    pub id: String,
    pub invalid_transactions: Vec<HashMap<String, String>>,
    pub status: String,
}
