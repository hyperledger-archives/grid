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

use std::time::Duration;

use sawtooth_sdk::messages::batch::Batch;
use sawtooth_sdk::messages::client_batch_submit::{
    ClientBatchStatusRequest, ClientBatchStatusResponse, ClientBatchStatusResponse_Status,
    ClientBatchSubmitRequest, ClientBatchSubmitResponse, ClientBatchSubmitResponse_Status,
};
use sawtooth_sdk::messages::validator::Message_MessageType;
use sawtooth_sdk::messaging::stream::MessageSender;
use sawtooth_sdk::messaging::zmq_stream::ZmqMessageSender;
use uuid::Uuid;

use crate::rest_api::error::RestApiResponseError;
use crate::submitter::{
    BatchStatus, BatchStatusLink, BatchStatuses, BatchSubmitter, SubmitBatches, DEFAULT_TIME_OUT,
};

#[derive(Clone)]
pub struct SawtoothBatchSubmitter {
    sender: ZmqMessageSender,
}

impl SawtoothBatchSubmitter {
    pub fn new(sender: ZmqMessageSender) -> Self {
        Self { sender }
    }
}

impl BatchSubmitter for SawtoothBatchSubmitter {
    fn submit_batches(&self, msg: SubmitBatches) -> Result<BatchStatusLink, RestApiResponseError> {
        let mut client_submit_request = ClientBatchSubmitRequest::new();
        client_submit_request.set_batches(protobuf::RepeatedField::from_vec(
            msg.batch_list.get_batches().to_vec(),
        ));

        let response_status: ClientBatchSubmitResponse = query_validator(
            &self.sender,
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

                let mut response_url = msg.response_url;
                response_url.set_query(Some(&format!("id={}", batch_query)));

                Ok(BatchStatusLink {
                    link: response_url.to_string(),
                })
            }
            Err(err) => Err(err),
        }
    }

    fn batch_status(&self, msg: BatchStatuses) -> Result<Vec<BatchStatus>, RestApiResponseError> {
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
            &self.sender,
            Message_MessageType::CLIENT_BATCH_STATUS_REQUEST,
            &batch_status_request,
        )?;

        process_batch_status_response(response_status)
    }

    fn clone_box(&self) -> Box<dyn BatchSubmitter> {
        Box::new(self.clone())
    }
}

pub fn query_validator<T: protobuf::Message, C: protobuf::Message, MS: MessageSender>(
    sender: &MS,
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

pub fn process_validator_response(
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

pub fn process_batch_status_response(
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
