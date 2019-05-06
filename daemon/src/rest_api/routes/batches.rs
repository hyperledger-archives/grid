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

use crate::rest_api::{error::RestApiResponseError, routes::SawtoothMessageSender, AppState};

use actix::{Context, Handler, Message};
use actix_web::{AsyncResponder, HttpMessage, HttpRequest, HttpResponse, Query, State};
use futures::future;
use futures::future::Future;
use sawtooth_sdk::messages::batch::{Batch, BatchList};
use sawtooth_sdk::messages::client_batch_submit::{
    ClientBatchStatus, ClientBatchStatusRequest, ClientBatchStatusResponse,
    ClientBatchStatusResponse_Status, ClientBatchSubmitRequest, ClientBatchSubmitResponse,
    ClientBatchSubmitResponse_Status,
};
use sawtooth_sdk::messages::validator::Message_MessageType;
use sawtooth_sdk::messaging::stream::MessageSender;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use url::Url;
use uuid::Uuid;

const DEFAULT_TIME_OUT: u32 = 300; // Max timeout 300 seconds == 5 minutes

struct SubmitBatches {
    batch_list: BatchList,
    response_url: Url,
}

impl Message for SubmitBatches {
    type Result = Result<BatchStatusLink, RestApiResponseError>;
}

struct BatchStatuses {
    batch_ids: Vec<String>,
    wait: Option<u32>,
}

impl Message for BatchStatuses {
    type Result = Result<Vec<BatchStatus>, RestApiResponseError>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchStatus {
    pub id: String,
    pub invalid_transactions: Vec<HashMap<String, String>>,
    pub status: String,
}

impl BatchStatus {
    pub fn from_proto(proto: &ClientBatchStatus) -> BatchStatus {
        BatchStatus {
            id: proto.get_batch_id().to_string(),
            invalid_transactions: proto
                .get_invalid_transactions()
                .iter()
                .map(|txn| {
                    let mut invalid_transaction_info = HashMap::new();
                    invalid_transaction_info
                        .insert("id".to_string(), txn.get_transaction_id().to_string());
                    invalid_transaction_info
                        .insert("message".to_string(), txn.get_message().to_string());
                    invalid_transaction_info.insert(
                        "extended_data".to_string(),
                        base64::encode(txn.get_extended_data()),
                    );
                    invalid_transaction_info
                })
                .collect(),
            status: format!("{:?}", proto.get_status()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchStatusResponse {
    pub data: Vec<BatchStatus>,
    pub link: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BatchStatusLink {
    pub link: String,
}

impl Handler<SubmitBatches> for SawtoothMessageSender {
    type Result = Result<BatchStatusLink, RestApiResponseError>;

    fn handle(&mut self, msg: SubmitBatches, _: &mut Context<Self>) -> Self::Result {
        let mut client_submit_request = ClientBatchSubmitRequest::new();
        client_submit_request.set_batches(protobuf::RepeatedField::from_vec(
            msg.batch_list.get_batches().to_vec(),
        ));

        let response_status: ClientBatchSubmitResponse = query_validator(
            &*self.sender,
            Message_MessageType::CLIENT_BATCH_SUBMIT_REQUEST,
            &client_submit_request,
        )?;

        match process_validator_response(response_status.get_status()) {
            Ok(_) => {
                let batch_query = msg
                    .batch_list
                    .get_batches()
                    .iter()
                    .map(Batch::get_header_signature)
                    .collect::<Vec<_>>()
                    .join(",");

                let mut response_url = msg.response_url.clone();
                response_url.set_query(Some(&format!("id={}", batch_query)));

                Ok(BatchStatusLink {
                    link: response_url.to_string(),
                })
            }
            Err(err) => Err(err),
        }
    }
}

impl Handler<BatchStatuses> for SawtoothMessageSender {
    type Result = Result<Vec<BatchStatus>, RestApiResponseError>;

    fn handle(&mut self, msg: BatchStatuses, _: &mut Context<Self>) -> Self::Result {
        let mut batch_status_request = ClientBatchStatusRequest::new();
        batch_status_request.set_batch_ids(protobuf::RepeatedField::from_vec(msg.batch_ids));
        match msg.wait {
            Some(wait_time) => {
                batch_status_request.set_wait(true);
                batch_status_request.set_timeout(wait_time);
            }
            None => {
                batch_status_request.set_wait(false);
            }
        }

        let response_status: ClientBatchStatusResponse = query_validator(
            &*self.sender,
            Message_MessageType::CLIENT_BATCH_STATUS_REQUEST,
            &batch_status_request,
        )?;

        process_batch_status_response(response_status)
    }
}

pub fn submit_batches(
    (req, state): (HttpRequest<AppState>, State<AppState>),
) -> impl Future<Item = HttpResponse, Error = RestApiResponseError> {
    req.body().from_err().and_then(
        move |body| -> Box<Future<Item = HttpResponse, Error = RestApiResponseError>> {
            let batch_list: BatchList = match protobuf::parse_from_bytes(&*body) {
                Ok(batch_list) => batch_list,
                Err(err) => {
                    return Box::new(future::err(RestApiResponseError::BadRequest(format!(
                        "Protobuf message was badly formatted. {}",
                        err.to_string()
                    ))));
                }
            };
            let response_url = match req.url_for_static("batch_statuses") {
                Ok(url) => url,
                Err(err) => return Box::new(future::err(err.into())),
            };

            let res = state
                .sawtooth_connection
                .send(SubmitBatches {
                    batch_list,
                    response_url,
                })
                .from_err()
                .and_then(|res| match res {
                    Ok(link) => Ok(HttpResponse::Ok().json(link)),
                    Err(err) => Err(err),
                });
            Box::new(res)
        },
    )
}

#[derive(Deserialize, Debug)]
struct Params {
    id: Vec<String>,
}

pub fn get_batch_statuses(
    (state, query, req): (
        State<AppState>,
        Query<HashMap<String, String>>,
        HttpRequest<AppState>,
    ),
) -> Box<Future<Item = HttpResponse, Error = RestApiResponseError>> {
    let batch_ids = match query.get("id") {
        Some(ids) => ids.split(',').map(ToString::to_string).collect(),
        None => {
            return future::err(RestApiResponseError::BadRequest(
                "Request for statuses missing id query.".to_string(),
            ))
            .responder();
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
                        return future::err(RestApiResponseError::BadRequest(format!(
                            "Query wait has invalid value {}. \
                             It should set to false or a a time in seconds to wait for the commit",
                            wait_time
                        )))
                        .responder();
                    }
                }
            }
        }

        None => Some(max_wait_time),
    };

    let response_url = match req.url_for_static("batch_statuses") {
        Ok(url) => format!("{}?{}", url, req.query_string()),
        Err(err) => return Box::new(future::err(err.into())),
    };

    state
        .sawtooth_connection
        .send(BatchStatuses { batch_ids, wait })
        .from_err()
        .and_then(|res| match res {
            Ok(batch_statuses) => Ok(HttpResponse::Ok().json(BatchStatusResponse {
                data: batch_statuses,
                link: response_url,
            })),
            Err(err) => Err(err),
        })
        .responder()
}

fn query_validator<T: protobuf::Message, C: protobuf::Message>(
    sender: &dyn MessageSender,
    message_type: Message_MessageType,
    message: &C,
) -> Result<T, RestApiResponseError> {
    let content = protobuf::Message::write_to_bytes(message).map_err(|err| {
        RestApiResponseError::RequestHandlerError(format!(
            "Failed to serialize batch submit request. {}",
            err.to_string()
        ))
    })?;

    let correlation_id = Uuid::new_v4().to_string();

    let mut response_future = sender
        .send(message_type, &correlation_id, &content)
        .map_err(|err| {
            RestApiResponseError::SawtoothConnectionError(format!(
                "Failed to send message to validator. {}",
                err.to_string()
            ))
        })?;

    protobuf::parse_from_bytes(
        response_future
            .get_timeout(Duration::new(DEFAULT_TIME_OUT.into(), 0))
            .map_err(|err| RestApiResponseError::RequestHandlerError(err.to_string()))?
            .get_content(),
    )
    .map_err(|err| {
        RestApiResponseError::RequestHandlerError(format!(
            "Failed to parse validator response from bytes. {}",
            err.to_string()
        ))
    })
}

fn process_validator_response(
    status: ClientBatchSubmitResponse_Status,
) -> Result<(), RestApiResponseError> {
    match status {
        ClientBatchSubmitResponse_Status::OK => Ok(()),
        ClientBatchSubmitResponse_Status::INVALID_BATCH => Err(RestApiResponseError::BadRequest(
            "The submitted BatchList was rejected by the validator. It was '
            'poorly formed, or has an invalid signature."
                .to_string(),
        )),
        _ => Err(RestApiResponseError::SawtoothValidatorResponseError(
            format!("Validator responded with error {:?}", status),
        )),
    }
}

fn process_batch_status_response(
    response: ClientBatchStatusResponse,
) -> Result<Vec<BatchStatus>, RestApiResponseError> {
    let status = response.get_status();
    match status {
        ClientBatchStatusResponse_Status::OK => Ok(response
            .get_batch_statuses()
            .iter()
            .map(BatchStatus::from_proto)
            .collect()),
        ClientBatchStatusResponse_Status::INVALID_ID => Err(RestApiResponseError::BadRequest(
            "Blockchain items are identified by 128 character hex-strings. A submitted \
             batch id was invalid"
                .to_string(),
        )),
        _ => Err(RestApiResponseError::SawtoothValidatorResponseError(
            format!("Validator responded with error {:?}", status),
        )),
    }
}
