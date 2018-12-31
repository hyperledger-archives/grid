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

use protobuf::parse_from_reader;
use rocket::data::Data;
use rocket_contrib::json::Json;
use sawtooth_sdk::messages::batch::BatchList;

use crate::routes::error::{BatchStatusesError, BatchSubmitError};

#[derive(Debug, Serialize)]
pub struct BatchesResponse {
    link: String,
}

#[post("/batches", format = "application/octet-stream", data = "<data>")]
pub fn batches(data: Data) -> Result<Json<BatchesResponse>, BatchSubmitError> {
    let mut data_stream = data.open();

    let batch_list: BatchList = parse_from_reader(&mut data_stream)?;

    log::debug!("Submitted {:?}", &batch_list);

    let batch_ids = batch_list
        .get_batches()
        .iter()
        .map(|batch| batch.header_signature.clone())
        .collect::<Vec<_>>()
        .join(",");

    Ok(Json(BatchesResponse {
        link: uri!(batch_statuses: id = batch_ids, wait = _).to_string(),
    }))
}

#[derive(Debug, Serialize)]
pub struct BatchStatusesResponse {
    data: Vec<BatchStatus>,
    link: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status")]
pub enum BatchStatus {
    COMMITTED {
        id: String,
    },
    INVALID {
        id: String,
        invalid_transactions: Vec<InvalidTransaction>,
    },
    PENDING {
        id: String,
    },
    UNKNOWN {
        id: String,
    },
}

#[derive(Debug, Serialize)]
pub struct InvalidTransaction {
    id: String,
    message: String,
    /// This is an Base64-encoded string of bytes
    extended_data: String,
}

#[get("/batch_statuses?<id>&<wait>")]
pub fn batch_statuses(
    id: String,
    wait: Option<u32>,
) -> Result<Json<BatchStatusesResponse>, BatchStatusesError> {
    let ids = id.split(",").collect::<Vec<_>>();
    let wait_time = wait.unwrap_or(0);

    log::debug!("Checking status for batches {:?}", &ids);

    Ok(Json(BatchStatusesResponse {
        data: ids
            .iter()
            .map(|batch_id| BatchStatus::PENDING {
                id: batch_id.to_string(),
            })
            .collect(),
        link: uri!(batch_statuses: id = id, wait = wait_time).to_string(),
    }))
}
