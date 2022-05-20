// Copyright 2022 Cargill Incorporated
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

//! Provides native representations of smart contract actions used to deserialize from JSON

mod batch;
mod location;
mod pike;
mod product;
mod purchase_order;
mod schema;

pub use location::LocationPayload;
pub use pike::PikePayload;
pub use product::ProductPayload;
pub use purchase_order::PurchaseOrderPayload;
pub use schema::{PropertyValue, SchemaPayload};

use cylinder::Signer;
use transact::protocol::transaction::Transaction;

use crate::rest_api::resources::error::ErrorResponse;

/// Represents all possible smart contract payloads able to be submitted
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Payload {
    Location(LocationPayload),
    Pike(PikePayload),
    Schema(SchemaPayload),
    Product(ProductPayload),
    PurchaseOrder(PurchaseOrderPayload),
}

impl Payload {
    pub fn into_inner(self) -> Box<dyn TransactionPayload> {
        match self {
            Payload::Location(payload) => payload.into_transaction_payload(),
            Payload::Pike(payload) => payload.into_transaction_payload(),
            Payload::Schema(payload) => payload.into_transaction_payload(),
            Payload::Product(payload) => payload.into_transaction_payload(),
            Payload::PurchaseOrder(payload) => payload.into_transaction_payload(),
        }
    }
}

pub trait TransactionPayload {
    fn build_transaction(&self, signer: Box<dyn Signer>) -> Result<Transaction, ErrorResponse>;
}

impl TransactionPayload for Box<dyn TransactionPayload> {
    fn build_transaction(&self, signer: Box<dyn Signer>) -> Result<Transaction, ErrorResponse> {
        (**self).build_transaction(signer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::location::{CreateLocationActionBuilder, LocationAction, LocationNamespace};
    use super::pike::{CreateAgentActionBuilder, PikeAction, UpdateAgentActionBuilder};
    use super::Payload;

    use serde_json;
    use std::time::{SystemTime, UNIX_EPOCH};

    const LOCATION: &str = "location";
    const ORG: &str = "myorg";
    const ROLE: &str = "test_role";
    const PUBLIC_KEY: &str = "PUBLIC_KEY";

    const JSON_CREATE_AGENT_PAYLOAD: &str =
        "{ \"org_id\": \"myorg\", \"public_key\": \"PUBLIC_KEY\", \"active\": true, \
        \"target\": \"POST /agent\" }";
    const JSON_UPDATE_AGENT_PAYLOAD: &str =
        "{ \"org_id\": \"myorg\", \"public_key\": \"PUBLIC_KEY\", \"active\": true, \
        \"target\": \"PUT /agent/PUBLIC_KEY\"}";

    const JSON_CREATE_LOCATION_PAYLOAD: &str =
        "{ \"namespace\": \"GS1\", \"location_id\": \"location\", \"owner\": \"myorg\", \
        \"target\": \"POST /location\" }";

    #[test]
    /// Test the process of deserializing a `CreateAgentAction` into a `Payload` enum.
    /// The test follows this process:
    ///
    /// 1. Create a String representing a `CreateAgentAction` as JSON
    /// 2. Deserialize this string using `serde_json` into a `Payload` enum variant
    /// 3. Validate this variant is equivalent to `Payload::CreateAgent` and the inner struct
    ///    is a `CreateAgentAction` with the expected values
    fn test_deserialize_json_create_agent_payload() {
        let example_payload = create_agent_payload();

        let deserialized_payload: Payload = serde_json::from_str(JSON_CREATE_AGENT_PAYLOAD)
            .expect("Unable to parse `create agent`");

        assert_eq!(example_payload, deserialized_payload);
    }

    #[test]
    /// Test the process of deserializing a `CreateLocationAction` into a `Payload` enum.
    /// The test follows this process:
    ///
    /// 1. Create a String representing a `CreateLocationAction` as JSON
    /// 2. Deserialize this string using `serde_json` into a `Payload` enum variant
    /// 3. Validate this variant is equivalent to `Payload::CreateLocation` and the inner struct
    ///    is a `CreateLocationAction` with the expected values
    fn test_deserialize_json_create_location_payload() {
        let example_payload = create_location_payload();

        let deserialized_payload: Payload = serde_json::from_str(JSON_CREATE_LOCATION_PAYLOAD)
            .expect("Unable to parse `create location`");

        assert_eq!(example_payload, deserialized_payload);
    }

    #[test]
    /// Test the process of deserializing a list of JSON strings into a list of `Payload` enums.
    /// The test follows this process:
    ///
    /// 1. Create multiple Strings representing various contract actions as JSON
    /// 2. Deserialize this list using `serde_json` into a list of `Payload` enum variants
    /// 3. Validate these `Payload` variants match the native struct that is expected
    fn test_deserialize_json_payload_list() {
        let example_payload_list = vec![
            create_location_payload(),
            create_agent_payload(),
            update_agent_payload(),
        ];
        // Test the deserializing of a list of these JSON payloads
        let payload_list = vec![
            JSON_CREATE_LOCATION_PAYLOAD,
            JSON_CREATE_AGENT_PAYLOAD,
            JSON_UPDATE_AGENT_PAYLOAD,
        ];
        let payloads: Vec<Payload> = payload_list
            .iter()
            .map(|e| serde_json::from_str(e).expect("Unable to parse action"))
            .collect();
        let assert_payloads = example_payload_list.iter().zip(payloads.iter());
        for (example_payload, test_payload) in assert_payloads {
            assert_payload_actions(example_payload, test_payload);
        }
    }

    fn assert_payload_actions(example: &Payload, test: &Payload) {
        match (example, test) {
            (Payload::Location(ex_payload), Payload::Location(test_payload)) => {
                assert_eq!(ex_payload.action(), test_payload.action())
            }
            (Payload::Pike(ex_payload), Payload::Pike(test_payload)) => {
                assert_eq!(ex_payload.action(), test_payload.action())
            }
            (Payload::Schema(ex_payload), Payload::Schema(test_payload)) => {
                assert_eq!(ex_payload.action(), test_payload.action())
            }
            (Payload::Product(ex_payload), Payload::Product(test_payload)) => {
                assert_eq!(ex_payload.action(), test_payload.action())
            }
            (Payload::PurchaseOrder(ex_payload), Payload::PurchaseOrder(test_payload)) => {
                assert_eq!(ex_payload.action(), test_payload.action())
            }
            (_, _) => {
                panic!(
                    "Invalid `Payload` comparison, expected: {:?}, got: {:?}",
                    example, test
                )
            }
        }
    }

    fn create_agent_payload() -> Payload {
        let action = PikeAction::CreateAgent(
            CreateAgentActionBuilder::default()
                .with_org_id(ORG.to_string())
                .with_public_key(PUBLIC_KEY.to_string())
                .with_active(true)
                .build()
                .expect("Unable to build CreateAgentAction"),
        );
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .expect("Unable to get current time as seconds");
        Payload::Pike(PikePayload::new(action, timestamp))
    }

    fn update_agent_payload() -> Payload {
        let action = PikeAction::UpdateAgent(
            UpdateAgentActionBuilder::default()
                .with_org_id(ORG.to_string())
                .with_public_key(PUBLIC_KEY.to_string())
                .with_active(true)
                .with_roles(vec![])
                .with_metadata(vec![])
                .build()
                .expect("Unable to build UpdateAgentAction"),
        );
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .expect("Unable to get current time as seconds");
        Payload::Pike(PikePayload::new(action, timestamp))
    }

    fn create_location_payload() -> Payload {
        let namespace = LocationNamespace::Gs1;
        let action = LocationAction::CreateLocation(
            CreateLocationActionBuilder::default()
                .with_namespace(namespace)
                .with_location_id(LOCATION.to_string())
                .with_owner(ORG.to_string())
                .with_properties(vec![])
                .build()
                .expect("Unable to build CreateLocationAction"),
        );
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .expect("Unable to get current time as seconds");
        Payload::Location(LocationPayload::new(action, timestamp))
    }
}
