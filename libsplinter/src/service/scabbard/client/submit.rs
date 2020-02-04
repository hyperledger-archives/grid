// Copyright 2020 Cargill Incorporated
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

//! Contains functions which assist with batch submission to a REST API

use std::{
    fmt, str,
    time::{Duration, Instant},
};

use protobuf::Message;
use reqwest::{
    blocking::{Client, RequestBuilder, Response},
    Url,
};

use sawtooth_sdk::messages::batch::BatchList;

use crate::service::scabbard::{BatchInfo, BatchStatus, SERVICE_TYPE};

use super::Error;

pub fn submit_batches(
    base_url: &str,
    circuit_id: &str,
    service_id: &str,
    batches: BatchList,
) -> Result<String, Error> {
    let url = parse_http_url(&format!(
        "{}/{}/{}/{}/batches",
        base_url, SERVICE_TYPE, circuit_id, service_id
    ))?;

    let body = batches.write_to_bytes()?;

    debug!("Submitting batches via {}", url);
    let request = Client::new().post(url).body(body);
    let response = perform_request(request)?;

    let batch_link: Link = response.json().map_err(|err| {
        Error::new_with_source("failed to parse response as batch link", err.into())
    })?;

    Ok(batch_link.link)
}

pub fn wait_for_batches(base_url: &str, batch_link: &str, wait: u64) -> Result<(), Error> {
    let url = if batch_link.starts_with("http") || batch_link.starts_with("https") {
        parse_http_url(batch_link)
    } else {
        parse_http_url(&format!("{}{}", base_url, batch_link))
    }?;

    let time_left = Duration::from_secs(wait);

    wait_for_batches_until_timeout(url, time_left)
}

/// Recursive function that will repeatedly get the batch statuses with `url` until no batches are
/// pending or the `time_left` reaches zero.
fn wait_for_batches_until_timeout(url: Url, time_left: Duration) -> Result<(), Error> {
    let start_time = Instant::now();

    let wait_query = format!("wait={}", time_left.as_secs());
    let query_string = if let Some(existing_query) = url.query() {
        format!("{}&{}", existing_query, wait_query)
    } else {
        wait_query
    };

    let mut url_with_query = url.clone();
    url_with_query.set_query(Some(&query_string));

    debug!("Checking batches via {}", url);
    let request = Client::new().get(url.clone());
    let response = perform_request(request)?;

    let batch_infos: Vec<BatchInfo> = response.json().map_err(|err| {
        Error::new_with_source("failed to parse response as batch statuses", err.into())
    })?;

    let any_pending_batches = batch_infos.iter().any(|info| {
        match info.status {
            // `Valid` is still technically pending until it's `Committed`
            BatchStatus::Pending | BatchStatus::Valid(_) => true,
            _ => false,
        }
    });

    if any_pending_batches {
        match time_left.checked_sub(start_time.elapsed()) {
            Some(time_still_left) => wait_for_batches_until_timeout(url, time_still_left),
            None => Err(Error::new(&format!(
                "one or more batches are still pending: {:?}",
                batch_infos
            ))),
        }
    } else {
        let any_invalid_batches = batch_infos.iter().any(|info| {
            if let BatchStatus::Invalid(_) = info.status {
                true
            } else {
                false
            }
        });

        if any_invalid_batches {
            Err(Error::new(&format!(
                "one or more batches were invalid: {:?}",
                batch_infos
            )))
        } else {
            Ok(())
        }
    }
}

fn parse_http_url(url: &str) -> Result<Url, Error> {
    let url = Url::parse(url).map_err(|err| Error::new_with_source("invalid URL", err.into()))?;
    if url.scheme() != "http" {
        Err(Error::new(&format!(
            "unsupported scheme ({}) in URL: {}",
            url.scheme(),
            url
        )))
    } else {
        Ok(url)
    }
}

fn perform_request(request: RequestBuilder) -> Result<Response, Error> {
    request
        .send()
        .map_err(|err| Error::new_with_source("request failed", err.into()))?
        .error_for_status()
        .map_err(|err| Error::new_with_source("received error status code", err.into()))
}

#[derive(Deserialize, Debug)]
struct Link {
    link: String,
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{\"link\": {}}}", self.link)
    }
}
