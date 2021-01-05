// Copyright 2019 Cargill Incorporated
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

use std::collections::HashMap;

use actix_web::{web, HttpRequest, HttpResponse};
use sawtooth_sdk::messages::batch::BatchList;
use serde::Deserialize;

use crate::rest_api::error::RestApiResponseError;
use crate::rest_api::{AcceptServiceIdParam, AppState, QueryServiceId};
use crate::submitter::{BatchStatusResponse, BatchStatuses, SubmitBatches, DEFAULT_TIME_OUT};

pub async fn submit_batches(
    req: HttpRequest,
    body: web::Bytes,
    state: web::Data<AppState>,
    query_service_id: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    let batch_list: BatchList = match protobuf::Message::parse_from_bytes(&*body) {
        Ok(batch_list) => batch_list,
        Err(err) => {
            return Err(RestApiResponseError::BadRequest(format!(
                "Protobuf message was badly formatted. {}",
                err.to_string()
            )));
        }
    };

    let response_url = req.url_for_static("batch_statuses")?;

    state
        .batch_submitter
        .submit_batches(SubmitBatches {
            batch_list,
            response_url,
            service_id: query_service_id.into_inner().service_id,
        })
        .await
        .map(|link| HttpResponse::Ok().json(link))
}

#[derive(Deserialize, Debug)]
struct Params {
    id: Vec<String>,
}

pub async fn get_batch_statuses(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<HashMap<String, String>>,
    query_service_id: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    let batch_ids = match query.get("id") {
        Some(ids) => ids.split(',').map(ToString::to_string).collect(),
        None => {
            return Err(RestApiResponseError::BadRequest(
                "Request for statuses missing id query.".to_string(),
            ));
        }
    };

    // Max wait time allowed is 95% of network's configured timeout
    let max_wait_time = (DEFAULT_TIME_OUT * 95) / 100;

    let wait = match query.get("wait") {
        Some(wait_time) => {
            if wait_time == "false" {
                None
            } else {
                match wait_time.parse::<u32>() {
                    Ok(wait_time) => {
                        if wait_time > max_wait_time {
                            Some(max_wait_time)
                        } else {
                            Some(wait_time)
                        }
                    }
                    Err(_) => {
                        return Err(RestApiResponseError::BadRequest(format!(
                            "Query wait has invalid value {}. \
                             It should set to false or a time in seconds to wait for the commit",
                            wait_time
                        )));
                    }
                }
            }
        }

        None => Some(max_wait_time),
    };

    let response_url = match req.url_for_static("batch_statuses") {
        Ok(url) => format!("{}?{}", url, req.query_string()),
        Err(err) => {
            return Err(err.into());
        }
    };

    state
        .batch_submitter
        .batch_status(BatchStatuses {
            batch_ids,
            wait,
            service_id: query_service_id.into_inner().service_id,
        })
        .await
        .map(|batch_statuses| {
            HttpResponse::Ok().json(BatchStatusResponse {
                data: batch_statuses,
                link: response_url,
            })
        })
}
