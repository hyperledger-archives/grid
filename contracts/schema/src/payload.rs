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

use grid_sdk::protocol::schema::payload::{
    Action, SchemaCreateAction, SchemaPayload, SchemaUpdateAction,
};

pub fn validate_payload(payload: &SchemaPayload) -> Result<(), ApplyError> {
    match payload.action() {
        Action::SchemaCreate(payload) => validate_schema_create_action(payload),
        Action::SchemaUpdate(payload) => validate_schema_update_action(payload),
    }
}

fn validate_schema_create_action(create_action: &SchemaCreateAction) -> Result<(), ApplyError> {
    if create_action.schema_name().is_empty() {
        return Err(ApplyError::InvalidTransaction(String::from(
            "Schema name must be set",
        )));
    }

    if create_action.properties().is_empty() {
        return Err(ApplyError::InvalidTransaction(String::from(
            "Properties must not be empty",
        )));
    }
    Ok(())
}

fn validate_schema_update_action(update_action: &SchemaUpdateAction) -> Result<(), ApplyError> {
    if update_action.schema_name().is_empty() {
        return Err(ApplyError::InvalidTransaction(String::from(
            "Schema name must be set",
        )));
    }

    if update_action.properties().is_empty() {
        return Err(ApplyError::InvalidTransaction(String::from(
            "Properties must not be empty",
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use grid_sdk::protocol::schema::payload::{
        Action, SchemaCreateBuilder, SchemaPayloadBuilder, SchemaUpdateBuilder,
    };
    use grid_sdk::protocol::schema::state::{DataType, PropertyDefinitionBuilder};
    use grid_sdk::protos;
    use grid_sdk::protos::IntoNative;

    #[test]
    // Test a payload with a schema create action is properly validated. This test needs to use
    // the proto directly originally to be able to mimic the scenarios possbile from creating
    // a SchemaPayload from bytes. This is because the SchemaPayloadBuilder protects from building
    // a certain invalid payloads.
    fn test_validate_schema_create_action() {
        let mut payload_proto = protos::schema_payload::SchemaPayload::new();
        assert!(
            payload_proto.clone().into_native().is_err(),
            "Cannot convert SchemaPayload_Action with type unset."
        );

        // create payload with no create SchemaCreateAction
        payload_proto.set_action(protos::schema_payload::SchemaPayload_Action::SCHEMA_CREATE);
        let payload = payload_proto.clone().into_native().unwrap();
        assert!(
            validate_payload(&payload).is_err(),
            "Empty SchemaCreateAction should not be valid"
        );

        let mut action = protos::schema_payload::SchemaCreateAction::new();

        // create payload with empty create SchemaCreateAction
        payload_proto.set_schema_create(action.clone());
        let payload = payload_proto.clone().into_native().unwrap();
        assert!(
            validate_payload(&payload).is_err(),
            "Schema name must be set"
        );

        // create payload with SchemaCreateAction without any properites,
        action.set_schema_name("test_schema".into());
        payload_proto.set_schema_create(action.clone());
        let payload = payload_proto.clone().into_native().unwrap();
        assert!(
            validate_payload(&payload).is_err(),
            "Properties must not be empty"
        );

        // create payload with full payload
        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaCreateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_owner("test_org".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        let builder = SchemaPayloadBuilder::new();
        let payload = builder
            .with_action(Action::SchemaCreate(action))
            .build()
            .unwrap();

        assert!(
            validate_payload(&payload).is_ok(),
            "Payload should be valid"
        );
    }

    #[test]
    // Test a payload with a schema update action is properly validated. This test needs to use
    // the proto directly originally to be able to mimic the scenarios possbile from creating
    // a SchemaPayload from bytes. This is because the SchemaPayloadBuilder protects from building
    // a certain invalid payloads.
    fn test_validate_schema_update_action() {
        let mut payload_proto = protos::schema_payload::SchemaPayload::new();
        assert!(
            payload_proto.clone().into_native().is_err(),
            "Cannot convert SchemaPayload_Action with type unset."
        );

        // create payload with no create SchemaCreateAction
        payload_proto.set_action(protos::schema_payload::SchemaPayload_Action::SCHEMA_UPDATE);
        let payload = payload_proto.clone().into_native().unwrap();
        assert!(
            validate_payload(&payload).is_err(),
            "Empty SchemaUpdateAction should not be valid"
        );

        let mut action = protos::schema_payload::SchemaUpdateAction::new();

        // create payload with empty create SchemaCreateAction
        payload_proto.set_schema_update(action.clone());
        let payload = payload_proto.clone().into_native().unwrap();
        assert!(
            validate_payload(&payload).is_err(),
            "Schema name must be set"
        );

        // create payload with SchemaCreateAction without any properites,
        action.set_schema_name("test_schema".into());
        payload_proto.set_schema_update(action.clone());
        let payload = payload_proto.clone().into_native().unwrap();
        assert!(
            validate_payload(&payload).is_err(),
            "Properties must not be empty"
        );

        // create payload with full payload
        let builder = PropertyDefinitionBuilder::new();
        let property_definition = builder
            .with_name("TEST".to_string())
            .with_data_type(DataType::String)
            .with_description("Optional".to_string())
            .build()
            .unwrap();

        let builder = SchemaUpdateBuilder::new();
        let action = builder
            .with_schema_name("TestSchema".to_string())
            .with_owner("test_org".to_string())
            .with_properties(vec![property_definition.clone()])
            .build()
            .unwrap();

        let builder = SchemaPayloadBuilder::new();
        let payload = builder
            .with_action(Action::SchemaUpdate(action))
            .build()
            .unwrap();

        assert!(
            validate_payload(&payload).is_ok(),
            "Payload should be valid"
        );
    }
}
