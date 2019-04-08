// Copyright 2019 Bitwise IO, Inc.
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

use crate::rest_api::{error::RestApiResponseError, AppState};

use actix::{Actor, Context, Handler, Message};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, State};
use futures::future;
use futures::future::Future;
use sawtooth_sdk::messages::batch::{Batch, BatchList};
use sawtooth_sdk::messages::client_batch_submit::{
    ClientBatchSubmitRequest, ClientBatchSubmitResponse, ClientBatchSubmitResponse_Status,
    ClientBatchStatus, ClientBatchStatusRequest, ClientBatchStatusResponse,
};
use sawtooth_sdk::messages::validator::Message_MessageType;
use sawtooth_sdk::messaging::stream::MessageSender;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;
use url::Url;
use uuid::Uuid;

const DEFAULT_TIME_OUT: u64 = 30;

pub struct SawtoothMessageSender {
    sender: Box<dyn MessageSender>,
}

impl Actor for SawtoothMessageSender {
    type Context = Context<Self>;
}

impl SawtoothMessageSender {
    pub fn new(sender: Box<dyn MessageSender>) -> SawtoothMessageSender {
        SawtoothMessageSender { sender }
    }
}

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

#[derive(Serialize)]
struct BatchStatus {
    id: String,
    invalid_transactions: Vec<HashMap<String, String>>,
    status: String,
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

#[derive(Serialize)]
struct BatchStatusResponse {
    data: Vec<BatchStatus>,
    link: String,
}

#[derive(Serialize)]
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
        let content = protobuf::Message::write_to_bytes(&client_submit_request).map_err(|err| {
            RestApiResponseError::RequestHandlerError(format!(
                "Failed to serialize batch submit request. {}",
                err.to_string()
            ))
        })?;
        let correlation_id = Uuid::new_v4().to_string();
        let mut response_future = self
            .sender
            .send(
                Message_MessageType::CLIENT_BATCH_SUBMIT_REQUEST,
                &correlation_id,
                &content,
            )
            .map_err(|err| {
                RestApiResponseError::SawtoothConnectionError(format!(
                    "Failed to send message to validator. {}",
                    err.to_string()
                ))
            })?;
        let response_status: ClientBatchSubmitResponse = protobuf::parse_from_bytes(
            response_future
                .get_timeout(Duration::new(DEFAULT_TIME_OUT, 0))
                .map_err(|err| RestApiResponseError::RequestHandlerError(err.to_string()))?
                .get_content(),
        )
        .map_err(|err| {
            RestApiResponseError::RequestHandlerError(format!(
                "Failed to parse validator response from bytes. {}",
                err.to_string()
            ))
        })?;

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

pub fn get_batch_statuses(
    (_req, _state): (HttpRequest<AppState>, State<AppState>),
) -> Box<Future<Item = HttpResponse, Error = RestApiResponseError>> {
    unimplemented!()
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
