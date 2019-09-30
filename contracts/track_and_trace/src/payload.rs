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

use grid_sdk::protocol::track_and_trace::payload::{
    Action, CreateRecordAction, TrackAndTracePayload,
};

pub fn validate_payload(payload: &TrackAndTracePayload) -> Result<(), ApplyError> {
    validate_timestamp(*payload.timestamp())?;
    match payload.action() {
        Action::CreateRecord(action_payload) => validate_record_create_action(action_payload),
        _ => Ok(()),
    }
}

fn validate_record_create_action(
    create_record_action: &CreateRecordAction,
) -> Result<(), ApplyError> {
    if create_record_action.record_id() == "" {
        return Err(ApplyError::InvalidTransaction(String::from(
            "Record id cannot be empty string",
        )));
    }

    if create_record_action.schema() == "" {
        return Err(ApplyError::InvalidTransaction(String::from(
            "Schema name cannot be empty string",
        )));
    }
    Ok(())
}

fn validate_timestamp(timestamp: u64) -> Result<(), ApplyError> {
    match timestamp {
        0 => Err(ApplyError::InvalidTransaction(String::from(
            "Timestamp is not set",
        ))),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use grid_sdk::protos::track_and_trace_payload::{
        CreateRecordAction as CreateRecordActionProto,
        TrackAndTracePayload as TrackAndTracePayloadProto,
        TrackAndTracePayload_Action as ActionProto,
    };
    use grid_sdk::protos::IntoNative;

    #[test]
    /// Test that an error is returned if the payload is missing the timestamp. This test needs
    /// to use the proto directly originally to be able to mimic the scenarios possbile from
    /// creating a CreateRecordAction from bytes. This is because the CreateRecordActionBuilder
    /// protects from building certain invalid payloads.
    fn test_validate_payload_timestamp_missing() {
        let mut payload_proto = TrackAndTracePayloadProto::new();

        payload_proto.set_action(ActionProto::CREATE_RECORD);
        let payload = payload_proto.clone().into_native().unwrap();
        match validate_payload(&payload) {
            Ok(_) => panic!("Payload missing timestamp, should return error"),
            Err(err) => assert!(err.to_string().contains("Timestamp is not set")),
        }
    }

    #[test]
    /// Test that an error is returned if the payload is missing the action. This test needs
    /// to use the proto directly originally to be able to mimic the scenarios possbile from
    /// creating a CreateRecordAction from bytes. This is because the CreateRecordActionBuilder
    /// protects from building certain invalid payloads.
    fn test_validate_payload_action_missing() {
        let mut payload_proto = TrackAndTracePayloadProto::new();

        payload_proto.set_action(ActionProto::CREATE_RECORD);
        payload_proto.set_timestamp(2);
        let payload = payload_proto.clone().into_native().unwrap();
        assert!(
            validate_payload(&payload).is_err(),
            "Empty CreateRecordAction should not be valid"
        );
    }

    #[test]
    /// Test that an error is returned if the payload with CreateRecordAction is missing the
    /// record_id. This test needs to use the proto directly originally to be able to mimic the
    /// scenarios possbile from creating a CreateRecordAction from bytes. This is because the
    /// CreateRecordActionBuilder protects from building certain invalid payloads.
    fn test_validate_payload_record_id_missing() {
        let mut payload_proto = TrackAndTracePayloadProto::new();

        payload_proto.set_action(ActionProto::CREATE_RECORD);
        payload_proto.set_timestamp(2);
        let action = CreateRecordActionProto::new();
        payload_proto.set_create_record(action.clone());
        let payload = payload_proto.clone().into_native().unwrap();
        match validate_payload(&payload) {
            Ok(_) => panic!("Payload missing record_id, should return error"),
            Err(err) => {
                println!("err {:?}", err);
                assert!(err.to_string().contains("Record id cannot be empty string"));
            }
        }
    }

    #[test]
    /// Test that an error is returned if the payload with CreateRecordAction is missing the
    /// schema. This test needs to use the proto directly originally to be able to mimic the
    /// scenarios possbile from creating a CreateRecordAction from bytes. This is because the
    /// CreateRecordActionBuilder protects from building certain invalid payloads.
    fn test_validate_payload_schema_missing() {
        let mut payload_proto = TrackAndTracePayloadProto::new();

        payload_proto.set_action(ActionProto::CREATE_RECORD);
        payload_proto.set_timestamp(2);
        let mut action = CreateRecordActionProto::new();
        action.set_record_id("my_record".to_string());
        payload_proto.set_create_record(action.clone());
        let payload = payload_proto.clone().into_native().unwrap();
        match validate_payload(&payload) {
            Ok(_) => panic!("Payload missing schema, should return error"),
            Err(err) => {
                println!("err {:?}", err);
                assert!(err
                    .to_string()
                    .contains("Schema name cannot be empty string"));
            }
        }
    }

    #[test]
    /// Test that an ok is returned if the payload with CreateRecordAction is valid. This test
    /// needs to use the proto directly originally to be able to mimic the scenarios possbile
    /// from creating a CreateRecordAction from bytes. This is because the
    /// CreateRecordActionBuilder protects from building certain invalid payloads.
    fn test_validate_payload_valid() {
        let mut payload_proto = TrackAndTracePayloadProto::new();

        payload_proto.set_action(ActionProto::CREATE_RECORD);
        payload_proto.set_timestamp(2);
        let mut action = CreateRecordActionProto::new();
        action.set_record_id("my_record".to_string());
        action.set_schema("my_schema".to_string());
        payload_proto.set_create_record(action.clone());
        let payload = payload_proto.clone().into_native().unwrap();
        assert!(
            validate_payload(&payload).is_ok(),
            "Payload should be valid"
        );
    }
}
