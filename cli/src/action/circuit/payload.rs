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

use openssl::hash::{hash, MessageDigest};
use protobuf::Message;
use sawtooth_sdk::signing::secp256k1;
use splinter::admin::messages::CreateCircuit;
use splinter::protos::admin::{
    CircuitCreateRequest, CircuitManagementPayload, CircuitManagementPayload_Action as Action,
    CircuitManagementPayload_Header as Header, CircuitProposalVote, CircuitProposalVote_Vote,
};
use splinter::signing::{sawtooth, Signer};

use crate::error::CliError;

use super::{CircuitVote, Vote};

/// A circuit action that has a type and can be converted into a protobuf-serializable struct.
pub trait CircuitAction<M: Message> {
    fn into_proto(self) -> Result<M, CliError>;

    fn action_type(&self) -> Action;
}

/// Applies a circuit payload action to the given CircuitManagementPayload.
pub trait ApplyToEnvelope {
    fn apply(self, circuit_management_payload: &mut CircuitManagementPayload);
}

/// Makes a signed, circuit management payload to be submitted to the Splinter REST API.
pub fn make_signed_payload<M, A>(
    requester_node: &str,
    private_key: &str,
    action: A,
) -> Result<Vec<u8>, CliError>
where
    M: Message + ApplyToEnvelope,
    A: CircuitAction<M>,
{
    let action_type = action.action_type();
    let action_proto = action.into_proto()?;
    let serialized_action = action_proto.write_to_bytes().map_err(|err| {
        CliError::ActionError(format!("Unable to serialize action to bytes: {}", err))
    })?;

    let hashed_bytes = hash(MessageDigest::sha512(), &serialized_action)?;

    let signing_context = secp256k1::Secp256k1Context::new();
    let private_key = secp256k1::Secp256k1PrivateKey::from_hex(private_key)
        .map_err(|_| CliError::ActionError("Invalid private key provided".into()))?;

    let signer = sawtooth::SawtoothSecp256k1RefSigner::new(&signing_context, private_key).map_err(
        |err| CliError::ActionError(format!("Unable to create signer from private key: {}", err)),
    )?;

    let public_key = signer.public_key().to_vec();

    let mut header = Header::new();
    header.set_action(action_type);
    header.set_payload_sha512(hashed_bytes.to_vec());
    header.set_requester(public_key);
    header.set_requester_node_id(requester_node.into());
    let header_bytes = header.write_to_bytes().map_err(|err| {
        CliError::ActionError(format!("Unable to serialize header to bytes: {}", err))
    })?;

    let header_signature = signer
        .sign(&header_bytes)
        .map_err(|err| CliError::ActionError(format!("Unable to sign payload header: {}", err)))?;

    let mut circuit_management_payload = CircuitManagementPayload::new();
    circuit_management_payload.set_header(header_bytes);
    circuit_management_payload.set_signature(header_signature);
    action_proto.apply(&mut circuit_management_payload);
    let payload_bytes = circuit_management_payload.write_to_bytes().map_err(|err| {
        CliError::ActionError(format!("Unable to serialize payload to bytes: {}", err))
    })?;
    Ok(payload_bytes)
}

// Conversions for explicit actions and their associated types.

impl CircuitAction<CircuitCreateRequest> for CreateCircuit {
    fn action_type(&self) -> Action {
        Action::CIRCUIT_CREATE_REQUEST
    }

    fn into_proto(self) -> Result<CircuitCreateRequest, CliError> {
        CreateCircuit::into_proto(self).map_err(|err| {
            CliError::ActionError(format!("Unable to convert proposal to protobuf: {}", err))
        })
    }
}

impl ApplyToEnvelope for CircuitCreateRequest {
    fn apply(self, circuit_management_payload: &mut CircuitManagementPayload) {
        circuit_management_payload.set_circuit_create_request(self);
    }
}

impl CircuitAction<CircuitProposalVote> for CircuitVote {
    fn action_type(&self) -> Action {
        Action::CIRCUIT_PROPOSAL_VOTE
    }

    fn into_proto(self) -> Result<CircuitProposalVote, CliError> {
        let mut vote = CircuitProposalVote::new();
        vote.set_vote(match self.vote {
            Vote::Accept => CircuitProposalVote_Vote::ACCEPT,
            Vote::Reject => CircuitProposalVote_Vote::REJECT,
        });
        vote.set_circuit_id(self.circuit_id);
        vote.set_circuit_hash(self.circuit_hash);

        Ok(vote)
    }
}

impl ApplyToEnvelope for CircuitProposalVote {
    fn apply(self, circuit_management_payload: &mut CircuitManagementPayload) {
        circuit_management_payload.set_circuit_proposal_vote(self);
    }
}
