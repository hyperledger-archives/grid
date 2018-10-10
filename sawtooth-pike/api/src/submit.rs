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

use protobuf;
use uuid;

use sawtooth_sdk::messaging::stream;
use sawtooth_sdk::messaging::zmq_stream::{ZmqMessageSender};
use sawtooth_sdk::messaging::stream::MessageSender;
use sawtooth_sdk::messages::validator::Message_MessageType;
use sawtooth_sdk::messages::validator::Message;
use sawtooth_sdk::messages::batch::BatchList;
use sawtooth_sdk::messages::client_batch_submit::ClientBatchSubmitResponse_Status as submit_status;
use sawtooth_sdk::messages::client_batch_submit::ClientBatchStatusResponse_Status as batch_res_status;
use sawtooth_sdk::messages::client_batch_submit::ClientBatchStatus_Status as batch_status;
use sawtooth_sdk::messages::client_batch_submit::{
    ClientBatchSubmitRequest,
    ClientBatchSubmitResponse,
    ClientBatchStatusRequest,
    ClientBatchStatusResponse
};

pub fn submit_batches(
        sender: &mut ZmqMessageSender,
        batches: &[u8],
        timeout: u32) -> Result<Vec<BatchStatus>, TransactionError> {

    let batch_list: BatchList = protobuf::parse_from_bytes(&batches)?;

    let mut submit_req = ClientBatchSubmitRequest::new();
    submit_req.set_batches(batch_list.batches.clone());

    let submit_req_bytes = protobuf::Message::write_to_bytes(&submit_req)?;

    let response = send(
        sender,
        Message_MessageType::CLIENT_BATCH_SUBMIT_REQUEST,
        &submit_req_bytes)?;

    let res_msg: ClientBatchSubmitResponse = protobuf::parse_from_bytes(&response.content)?;

    if res_msg.status != submit_status::OK {
        return Err(TransactionError::map_submit_status(res_msg.status));
    }

    let batch_ids = batch_list.batches.into_iter()
        .map(|b| b.header_signature)
        .collect();

    check_batch_status(sender, batch_ids, timeout)
}

pub fn check_batch_status(
    sender: &mut ZmqMessageSender,
    batch_ids: Vec<String>,
    timeout: u32) -> Result<Vec<BatchStatus>, TransactionError> {

    let mut request = ClientBatchStatusRequest::new();
    request.set_batch_ids(protobuf::RepeatedField::from_vec(batch_ids));
    request.set_wait(timeout > 0);
    request.set_timeout(timeout);

    let req_bytes = protobuf::Message::write_to_bytes(&request)?;

    let response = send(
        sender,
        Message_MessageType::CLIENT_BATCH_STATUS_REQUEST,
        &req_bytes)?;

    let res_msg: ClientBatchStatusResponse = protobuf::parse_from_bytes(&response.content)?;

    if res_msg.status != batch_res_status::OK {
        return Err(TransactionError::map_batch_status(res_msg.status));
    }

    Ok(res_msg
        .batch_statuses
        .into_iter()
        .map(|b| BatchStatus::new(b.batch_id, b.status))
        .collect())

}

fn send(
    sender: &mut ZmqMessageSender,
    message_type: Message_MessageType,
    content: &[u8]) -> Result<Message, TransactionError> {

    let correlation_id = &generate_uuid()?;
    sender.send(message_type, correlation_id, content)?
        .get()
        .map_err(TransactionError::from)
}

fn generate_uuid() -> Result<String, TransactionError> {
    uuid::Uuid::new(uuid::UuidVersion::Random)
        .ok_or(
            TransactionError::UuidGenerationError(
                String::from("Failed to generate UUID")))
        .and_then(|uuid| Ok(uuid.to_string()))
}

#[derive(Deserialize, Serialize)]
pub struct BatchStatus {
    pub batch_id: String,
    pub status: String
}

impl BatchStatus {
    fn new(id: String, status: batch_status) -> BatchStatus {
        let status_str = match status {
            batch_status::STATUS_UNSET => String::from("STATUS_UNSET"),
            batch_status::COMMITTED => String::from("COMMITTED"),
            batch_status::INVALID => String::from("INVALID"),
            batch_status::PENDING => String::from("PENDING"),
            batch_status::UNKNOWN => String::from("UNKNOWN")
        };

        BatchStatus {
            batch_id: id,
            status: status_str
        }
    }
}

#[derive(Debug)]
pub enum TransactionError {
    BatchParseError(protobuf::ProtobufError),
    UuidGenerationError(String),
    RequestError(stream::SendError),
    ResponseError(stream::ReceiveError),

    UnsetError(String),
    ValidatorInternalError(String),
    InvalidBatch(String),
    NoResource(String),
    InvalidId(String),

    UnknownError(String)
}

impl TransactionError {
    fn map_submit_status(status: submit_status) -> TransactionError {
        match status {
            submit_status::STATUS_UNSET =>
                TransactionError::UnsetError(String::from("Response status was not set")),
            submit_status::INTERNAL_ERROR =>
                TransactionError::ValidatorInternalError(String::from("Internal validator error")),
            submit_status::INVALID_BATCH =>
                TransactionError::InvalidBatch(String::from("Invalid Batch")),
            _ => TransactionError::UnknownError(String::from("Unknown error"))
        }
    }

    fn map_batch_status(status: batch_res_status) -> TransactionError {
        match status {
            batch_res_status::STATUS_UNSET =>
                TransactionError::UnsetError(String::from("Response status was not set")),
            batch_res_status::INTERNAL_ERROR =>
                TransactionError::ValidatorInternalError(String::from("Internal validator error")),
            batch_res_status::NO_RESOURCE =>
                TransactionError::NoResource(String::from("Resource not found")),
            batch_res_status::INVALID_ID =>
                TransactionError::InvalidId(String::from("Batch Id supplied is invalid")),
            _ => TransactionError::UnknownError(String::from("Unknown error"))
        }
    }
}

impl From<protobuf::ProtobufError> for TransactionError {
    fn from(e: protobuf::ProtobufError) -> Self {
        TransactionError::BatchParseError(e)
    }
}

impl From<stream::SendError> for TransactionError {
    fn from(e: stream::SendError) -> Self {
        TransactionError::RequestError(e)
    }
}

impl From<stream::ReceiveError> for TransactionError {
    fn from(e: stream::ReceiveError) -> Self {
        TransactionError::ResponseError(e)
    }
}
