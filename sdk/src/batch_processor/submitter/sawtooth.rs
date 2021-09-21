// Copyright 2018-2021 Cargill Incorporated
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

use sawtooth_sdk::messages::client_batch_submit::{
    ClientBatchStatusRequest, ClientBatchStatusResponse, ClientBatchStatusResponse_Status,
    ClientBatchSubmitRequest, ClientBatchSubmitResponse, ClientBatchSubmitResponse_Status,
};
use sawtooth_sdk::messages::validator::Message_MessageType;
use sawtooth_sdk::messaging::stream::MessageSender;
use sawtooth_sdk::messaging::{
    stream::{MessageConnection, MessageReceiver},
    zmq_stream::{ZmqMessageConnection, ZmqMessageSender},
};
use uuid::Uuid;

use super::{BatchStatus, BatchStatuses, BatchSubmitter, SubmitBatches};

use super::error::BatchSubmitterError;

pub const DEFAULT_TIME_OUT: u32 = 300; // Max timeout 300 seconds == 5 minutes

pub struct SawtoothConnection {
    sender: ZmqMessageSender,
    receiver: MessageReceiver,
}

impl SawtoothConnection {
    pub fn new(validator_address: &str) -> SawtoothConnection {
        let zmq_connection = ZmqMessageConnection::new(validator_address);
        let (sender, receiver) = zmq_connection.create();
        SawtoothConnection { sender, receiver }
    }

    pub fn get_sender(&self) -> ZmqMessageSender {
        self.sender.clone()
    }

    pub fn get_receiver(&self) -> &MessageReceiver {
        &self.receiver
    }
}

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
    fn submit_batches(&self, msg: SubmitBatches) -> Result<(), BatchSubmitterError> {
        let mut client_submit_request = ClientBatchSubmitRequest::new();
        client_submit_request.set_batches(protobuf::RepeatedField::from_vec(
            msg.batch_list.get_batches().to_vec(),
        ));

        let response_status: ClientBatchSubmitResponse = query_validator(
            &self.sender,
            Message_MessageType::CLIENT_BATCH_SUBMIT_REQUEST,
            &client_submit_request,
        )?;

        process_validator_response(response_status.get_status())
    }

    fn batch_status(&self, msg: BatchStatuses) -> Result<Vec<BatchStatus>, BatchSubmitterError> {
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
) -> Result<T, BatchSubmitterError> {
    let content = protobuf::Message::write_to_bytes(message).map_err(|err| {
        BatchSubmitterError::BadRequestError(format!(
            "Failed to serialize batch submit request. {}",
            err.to_string()
        ))
    })?;

    let correlation_id = Uuid::new_v4().to_string();

    let mut response_future = sender
        .send(message_type, &correlation_id, &content)
        .map_err(|err| {
            BatchSubmitterError::ConnectionError(format!(
                "Failed to send message to validator. {}",
                err.to_string()
            ))
        })?;

    protobuf::Message::parse_from_bytes(
        response_future
            .get_timeout(Duration::new(DEFAULT_TIME_OUT.into(), 0))
            .map_err(|err| BatchSubmitterError::InternalError(err.to_string()))?
            .get_content(),
    )
    .map_err(|err| {
        BatchSubmitterError::InternalError(format!(
            "Failed to parse validator response from bytes. {}",
            err.to_string()
        ))
    })
}

pub fn process_validator_response(
    status: ClientBatchSubmitResponse_Status,
) -> Result<(), BatchSubmitterError> {
    match status {
        ClientBatchSubmitResponse_Status::OK => Ok(()),
        ClientBatchSubmitResponse_Status::INVALID_BATCH => {
            Err(BatchSubmitterError::BadRequestError(
                "The submitted BatchList was rejected by the validator. It was '
            'poorly formed, or has an invalid signature."
                    .to_string(),
            ))
        }
        _ => Err(BatchSubmitterError::InternalError(format!(
            "Validator responded with error {:?}",
            status
        ))),
    }
}

pub fn process_batch_status_response(
    response: ClientBatchStatusResponse,
) -> Result<Vec<BatchStatus>, BatchSubmitterError> {
    let status = response.get_status();
    match status {
        ClientBatchStatusResponse_Status::OK => Ok(response
            .get_batch_statuses()
            .iter()
            .map(BatchStatus::from_proto)
            .collect()),
        ClientBatchStatusResponse_Status::INVALID_ID => Err(BatchSubmitterError::BadRequestError(
            "Blockchain items are identified by 128 character hex-strings. A submitted \
             batch id was invalid"
                .to_string(),
        )),
        _ => Err(BatchSubmitterError::InternalError(format!(
            "Validator responded with error {:?}",
            status
        ))),
    }
}
