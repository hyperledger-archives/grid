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

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::ApplyError;
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
    }
}

use grid_sdk::protos::track_and_trace_payload::CreateTrackAndTraceAgentAction as CreateAgentAction;
use grid_sdk::protos::track_and_trace_payload::{
    AnswerProposalAction, CreateProposalAction, CreateRecordAction, CreateRecordTypeAction,
    FinalizeRecordAction, RevokeReporterAction, SCPayload, SCPayload_Action,
    UpdatePropertiesAction,
};

#[derive(Debug, Clone)]
pub enum Action {
    CreateAgent(CreateAgentAction),
    CreateRecord(CreateRecordAction),
    FinalizeRecord(FinalizeRecordAction),
    CreateRecordType(CreateRecordTypeAction),
    UpdateProperties(UpdatePropertiesAction),
    CreateProposal(CreateProposalAction),
    AnswerProposal(AnswerProposalAction),
    RevokeReporter(RevokeReporterAction),
}

pub struct SupplyChainPayload {
    action: Action,
    timestamp: u64,
}

impl SupplyChainPayload {
    pub fn new(payload: &[u8]) -> Result<Option<SupplyChainPayload>, ApplyError> {
        let payload: SCPayload = match protobuf::parse_from_bytes(payload) {
            Ok(payload) => payload,
            Err(_) => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Cannot deserialize payload",
                )));
            }
        };

        let supply_chain_action = payload.get_action();
        let action = match supply_chain_action {
            SCPayload_Action::CREATE_AGENT => {
                let create_agent = payload.get_create_agent();
                if create_agent.get_name() == "" {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Agent name cannot be an empty string",
                    )));
                }
                Action::CreateAgent(create_agent.clone())
            }
            SCPayload_Action::CREATE_RECORD => {
                let create_record = payload.get_create_record();
                if create_record.get_record_id() == "" {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Record id cannot be empty string",
                    )));
                }
                Action::CreateRecord(create_record.clone())
            }
            SCPayload_Action::FINALIZE_RECORD => {
                Action::FinalizeRecord(payload.get_finalize_record().clone())
            }
            SCPayload_Action::CREATE_RECORD_TYPE => {
                let create_record_type = payload.get_create_record_type();
                if create_record_type.get_name() == "" {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Record type name cannot be an empty string",
                    )));
                };
                let properties = create_record_type.get_properties();
                if properties.is_empty() {
                    return Err(ApplyError::InvalidTransaction(String::from(
                        "Record type must have at least one property",
                    )));
                }
                for prop in properties {
                    if prop.name == "" {
                        return Err(ApplyError::InvalidTransaction(String::from(
                            "Property name cannot be an empty string",
                        )));
                    }
                }

                Action::CreateRecordType(create_record_type.clone())
            }
            SCPayload_Action::UPDATE_PROPERTIES => {
                Action::UpdateProperties(payload.get_update_properties().clone())
            }
            SCPayload_Action::CREATE_PROPOSAL => {
                Action::CreateProposal(payload.get_create_proposal().clone())
            }
            SCPayload_Action::ANSWER_PROPOSAL => {
                Action::AnswerProposal(payload.get_answer_proposal().clone())
            }
            SCPayload_Action::REVOKE_REPORTER => {
                Action::RevokeReporter(payload.get_revoke_reporter().clone())
            }
        };
        let timestamp = match payload.get_timestamp() {
            0 => {
                return Err(ApplyError::InvalidTransaction(String::from(
                    "Timestamp is not set",
                )));
            }
            x => x,
        };

        Ok(Some(SupplyChainPayload { action, timestamp }))
    }

    pub fn get_action(&self) -> Action {
        self.action.clone()
    }

    pub fn get_timestamp(&self) -> u64 {
        self.timestamp
    }
}
