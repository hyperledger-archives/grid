// Copyright 2018 Cargill Incorporated
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

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use iron::prelude::*;
use iron::status;
use protobuf::parse_from_reader;
use router::url_for;
use transact::protos::batch::{Batch, BatchList};

use super::error::{BatchStatusesError, BatchSubmitError};
use super::{query_param, Json, State};

#[derive(Debug, Serialize)]
pub struct BatchesResponse {
    link: String,
}

/// The handler function for the `/batches` endpoint
pub fn batches(req: &mut Request) -> IronResult<Response> {
    let pending_batches = req
        .extensions
        .get::<State<Arc<Mutex<VecDeque<Batch>>>>>()
        .expect("Expected pending batches, but none was set on the request");

    let batch_list: BatchList = parse_from_reader(&mut req.body).map_err(BatchSubmitError::from)?;

    log::debug!("Submitted {:?}", &batch_list);

    let batch_ids = batch_list
        .get_batches()
        .iter()
        .map(|batch| batch.header_signature.clone())
        .collect::<Vec<_>>()
        .join(",");

    let batch =
        batch_list.batches.get(0).cloned().ok_or_else(|| {
            BatchSubmitError::InvalidBatchListFormat("No batches provided".into())
        })?;

    pending_batches
        .lock()
        .expect("pending batches lock poisoned")
        .push_back(batch);

    let mut params = HashMap::new();
    params.insert("id".into(), batch_ids);

    let link = url_for(&req, "batch_statuses", params).to_string();

    Ok(Response::with((
        status::Accepted,
        Json(BatchesResponse { link }),
    )))
}

#[derive(Debug, Serialize)]
pub struct BatchStatusesResponse {
    data: Vec<BatchStatus>,
    link: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status")]
pub enum BatchStatus {
    PENDING { id: String },
}

/// The handler function for the `/batch_statuses` endpoint
pub fn batch_statuses(req: &mut Request) -> IronResult<Response> {
    let id: String = query_param(req, "id")
        .unwrap()
        .ok_or_else(|| BatchStatusesError::MissingParameter("id".into()))?;
    let wait: Option<u32> = query_param(req, "wait").map_err(|err| {
        BatchStatusesError::InvalidParameter(format!("wait must be an integer: {}", err))
    })?;
    let ids = id.split(',').collect::<Vec<_>>();
    let wait_time = wait.unwrap_or(0);

    log::debug!("Checking status for batches {:?}", &ids);
    let mut params = HashMap::new();
    params.insert("id".into(), id.clone());
    params.insert("wait".into(), wait_time.to_string());

    let link = url_for(&req, "batch_statuses", params).to_string();

    Ok(Response::with((
        status::Ok,
        Json(BatchStatusesResponse {
            data: ids
                .iter()
                .map(|batch_id| BatchStatus::PENDING {
                    id: (*batch_id).to_string(),
                })
                .collect(),
            link,
        }),
    )))
}
