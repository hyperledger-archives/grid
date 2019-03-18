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

use crossbeam_channel::Sender;
use protobuf::{parse_from_reader, Message};
use rocket::data::Data;
use rocket::State;
use rocket_contrib::json::Json;
use transact::protos::batch::BatchList;

use libsplinter::network::sender::SendRequest;
use libsplinter::protos::n_phase::{
    NPhaseTransactionMessage, NPhaseTransactionMessage_Type, TransactionVerificationRequest,
};

use crate::routes::error::{BatchStatusesError, BatchSubmitError};
use crate::service::{create_circuit_direct_msg, ServiceConfig};
use crate::transaction::XoState;

#[derive(Debug, Serialize)]
pub struct BatchesResponse {
    link: String,
}

#[post("/batches", format = "application/octet-stream", data = "<data>")]
pub fn batches(
    service_config: State<ServiceConfig>,
    xo_state: State<XoState>,
    sender: State<Sender<SendRequest>>,
    data: Data,
) -> Result<Json<BatchesResponse>, BatchSubmitError> {
    let mut data_stream = data.open();

    let batch_list: BatchList = parse_from_reader(&mut data_stream)?;

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

    let expected_output_hash = xo_state
        .propose_change(transact::protocol::batch::Batch::from(batch.clone()))
        .map_err(|err| {
            error!("Unable to propose change: {}", &err);
            BatchSubmitError::Internal(format!("Unable to propose change: {}", err))
        })?;

    let correlation_id = uuid::Uuid::new_v4().to_string();

    let mut request = TransactionVerificationRequest::new();
    request.set_correlation_id(correlation_id.clone());
    request.set_transaction_payload(
        batch
            .write_to_bytes()
            .expect("Unable to reserialize a deserialized batch list"),
    );
    request.set_expected_output_hash(expected_output_hash.into_bytes());

    let mut nphase_msg = NPhaseTransactionMessage::new();
    nphase_msg.set_message_type(NPhaseTransactionMessage_Type::TRANSACTION_VERIFICATION_REQUEST);
    nphase_msg.set_transaction_verification_request(request);

    for verifier in service_config.verifiers() {
        sender
            .send(SendRequest::new(
                service_config.peer_id().to_owned(),
                create_circuit_direct_msg(
                    service_config.circuit().to_owned(),
                    service_config.service_id().to_owned(),
                    verifier.clone(),
                    &nphase_msg,
                    correlation_id.clone(),
                )
                .map_err(|err| {
                    error!("unable to create circuit direct message: {}", &err);
                    BatchSubmitError::Internal(err.to_string())
                })?,
            ))
            .map_err(|err| {
                error!("Unable to send verification request: {}", &err);
                BatchSubmitError::Internal(err.to_string())
            })?;
    }

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
