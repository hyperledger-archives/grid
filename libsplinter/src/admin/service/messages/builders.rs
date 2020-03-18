// Copyright 2018-2020 Cargill Incorporated
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

use std::error::Error as StdError;

use crate::base62::generate_random_base62_string;

use super::{
    is_valid_circuit_id, is_valid_service_id, AuthorizationType, CreateCircuit, DurabilityType,
    PersistenceType, RouteType, SplinterNode, SplinterService,
};

#[derive(Default, Clone)]
pub struct CreateCircuitBuilder {
    circuit_id: Option<String>,
    roster: Option<Vec<SplinterService>>,
    members: Option<Vec<SplinterNode>>,
    authorization_type: Option<AuthorizationType>,
    persistence: Option<PersistenceType>,
    durability: Option<DurabilityType>,
    routes: Option<RouteType>,
    circuit_management_type: Option<String>,
    application_metadata: Option<Vec<u8>>,
}

impl CreateCircuitBuilder {
    pub fn new() -> Self {
        CreateCircuitBuilder::default()
    }

    pub fn circuit_id(&self) -> Option<String> {
        self.circuit_id.clone()
    }

    pub fn roster(&self) -> Option<Vec<SplinterService>> {
        self.roster.clone()
    }

    pub fn members(&self) -> Option<Vec<SplinterNode>> {
        self.members.clone()
    }

    pub fn authorization_type(&self) -> Option<AuthorizationType> {
        self.authorization_type.clone()
    }

    pub fn persistence(&self) -> Option<PersistenceType> {
        self.persistence.clone()
    }

    pub fn durability(&self) -> Option<DurabilityType> {
        self.durability.clone()
    }

    pub fn routes(&self) -> Option<RouteType> {
        self.routes.clone()
    }

    pub fn circuit_management_type(&self) -> Option<String> {
        self.circuit_management_type.clone()
    }

    pub fn application_metadata(&self) -> Option<Vec<u8>> {
        self.application_metadata.clone()
    }

    pub fn with_circuit_id(mut self, circuit_id: &str) -> CreateCircuitBuilder {
        self.circuit_id = Some(circuit_id.into());
        self
    }

    pub fn with_roster(mut self, services: &[SplinterService]) -> CreateCircuitBuilder {
        self.roster = Some(services.into());
        self
    }

    pub fn with_members(mut self, members: &[SplinterNode]) -> CreateCircuitBuilder {
        self.members = Some(members.into());
        self
    }

    pub fn with_authorization_type(
        mut self,
        authorization_type: &AuthorizationType,
    ) -> CreateCircuitBuilder {
        self.authorization_type = Some(authorization_type.clone());
        self
    }

    pub fn with_persistence(mut self, persistence: &PersistenceType) -> CreateCircuitBuilder {
        self.persistence = Some(persistence.clone());
        self
    }

    pub fn with_durability(mut self, durability: &DurabilityType) -> CreateCircuitBuilder {
        self.durability = Some(durability.clone());
        self
    }

    pub fn with_routes(mut self, route_type: &RouteType) -> CreateCircuitBuilder {
        self.routes = Some(route_type.clone());
        self
    }

    pub fn with_circuit_management_type(
        mut self,
        circuit_management_type: &str,
    ) -> CreateCircuitBuilder {
        self.circuit_management_type = Some(circuit_management_type.into());
        self
    }

    pub fn with_application_metadata(
        mut self,
        application_metadata: &[u8],
    ) -> CreateCircuitBuilder {
        self.application_metadata = Some(application_metadata.into());
        self
    }

    pub fn build(self) -> Result<CreateCircuit, BuilderError> {
        let circuit_id = match self.circuit_id {
            Some(circuit_id) if is_valid_circuit_id(&circuit_id) => circuit_id,
            Some(circuit_id) => {
                return Err(BuilderError::InvalidField(format!(
                    "Field circuit_id is invalid ({}): must be an 11 character string \
                     composed of two, 5 character base62 strings joined with a '-' (example: \
                     abcDE-F0123)",
                    circuit_id,
                )))
            }
            None => generate_random_base62_string(5) + "-" + &generate_random_base62_string(5),
        };

        let roster = self.roster.ok_or_else(|| {
            BuilderError::MissingField(
                "Unable to build CreateCircuit message. Missing required field roster".to_string(),
            )
        })?;

        let members = self.members.ok_or_else(|| {
            BuilderError::MissingField(
                "Unable to build CreateCircuit message. Missing required field members".to_string(),
            )
        })?;

        let authorization_type = self.authorization_type.unwrap_or_else(|| {
            debug!(
                "Building circuit create request with default authorization_type: {:?}",
                AuthorizationType::Trust
            );
            AuthorizationType::Trust
        });

        let persistence = self.persistence.unwrap_or_else(|| {
            debug!(
                "Building circuit create request with default persistence_type: {:?}",
                PersistenceType::default()
            );
            PersistenceType::default()
        });

        let durability = self.durability.unwrap_or_else(|| {
            debug!(
                "Building circuit create request with default durability: {:?}",
                DurabilityType::NoDurability
            );
            DurabilityType::NoDurability
        });

        let routes = self.routes.unwrap_or_else(|| {
            debug!(
                "Building circuit create request with default route type: {:?}",
                RouteType::default()
            );
            RouteType::default()
        });

        let circuit_management_type = self.circuit_management_type.ok_or_else(|| {
            BuilderError::MissingField(
                "Unable to build CreateCircuit message. \
                 Missing required field circuit_management_type"
                    .to_string(),
            )
        })?;

        let application_metadata = self.application_metadata.unwrap_or_default();

        let create_circuit_message = CreateCircuit {
            circuit_id,
            roster,
            members,
            authorization_type,
            persistence,
            durability,
            routes,
            circuit_management_type,
            application_metadata,
        };

        Ok(create_circuit_message)
    }
}

#[derive(Default, Clone)]
pub struct SplinterServiceBuilder {
    service_id: Option<String>,
    service_type: Option<String>,
    allowed_nodes: Option<Vec<String>>,
    arguments: Option<Vec<(String, String)>>,
}

impl SplinterServiceBuilder {
    pub fn new() -> Self {
        SplinterServiceBuilder::default()
    }

    pub fn service_id(&self) -> Option<String> {
        self.service_id.clone()
    }

    pub fn service_type(&self) -> Option<String> {
        self.service_type.clone()
    }

    pub fn allowed_nodes(&self) -> Option<Vec<String>> {
        self.allowed_nodes.clone()
    }

    pub fn arguments(&self) -> Option<Vec<(String, String)>> {
        self.arguments.clone()
    }

    pub fn with_service_id(mut self, service_id: &str) -> SplinterServiceBuilder {
        self.service_id = Some(service_id.into());
        self
    }

    pub fn with_service_type(mut self, service_type: &str) -> SplinterServiceBuilder {
        self.service_type = Some(service_type.into());
        self
    }

    pub fn with_allowed_nodes(mut self, allowed_nodes: &[String]) -> SplinterServiceBuilder {
        self.allowed_nodes = Some(allowed_nodes.into());
        self
    }

    pub fn with_arguments(mut self, arguments: &[(String, String)]) -> SplinterServiceBuilder {
        self.arguments = Some(arguments.into());
        self
    }

    pub fn build(self) -> Result<SplinterService, BuilderError> {
        let service_id = match self.service_id {
            Some(service_id) if is_valid_service_id(&service_id) => service_id,
            Some(service_id) => {
                return Err(BuilderError::InvalidField(format!(
                    "Field service_id is invalid ({}): must be a 4 character base62 string",
                    service_id,
                )))
            }
            None => generate_random_base62_string(4),
        };

        let service_type = self.service_type.ok_or_else(|| {
            BuilderError::MissingField(
                "Unable to build SplinterService. Missing required field service_type".to_string(),
            )
        })?;

        let allowed_nodes = self.allowed_nodes.ok_or_else(|| {
            BuilderError::MissingField(
                "Unable to build SplinterService. Missing required field allowed_nodes".to_string(),
            )
        })?;

        let arguments = self.arguments.unwrap_or_default();

        let service = SplinterService {
            service_id,
            service_type,
            allowed_nodes,
            arguments,
        };

        Ok(service)
    }
}

#[derive(Default, Clone)]
pub struct SplinterNodeBuilder {
    node_id: Option<String>,
    endpoint: Option<String>,
}

impl SplinterNodeBuilder {
    pub fn new() -> Self {
        SplinterNodeBuilder::default()
    }

    pub fn node_id(&self) -> Option<String> {
        self.node_id.clone()
    }

    pub fn endpoint(&self) -> Option<String> {
        self.endpoint.clone()
    }

    pub fn with_node_id(mut self, node_id: &str) -> SplinterNodeBuilder {
        self.node_id = Some(node_id.into());
        self
    }

    pub fn with_endpoint(mut self, endpoint: &str) -> SplinterNodeBuilder {
        self.endpoint = Some(endpoint.into());
        self
    }

    pub fn build(self) -> Result<SplinterNode, BuilderError> {
        let node_id = self.node_id.ok_or_else(|| {
            BuilderError::MissingField(
                "Unable to build SplinterNode. Missing required field node_id".to_string(),
            )
        })?;

        let endpoint = self.endpoint.ok_or_else(|| {
            BuilderError::MissingField(
                "Unable to build SplinterNode. Missing required field endpoint".to_string(),
            )
        })?;

        let node = SplinterNode { node_id, endpoint };

        Ok(node)
    }
}

#[derive(Debug)]
pub enum BuilderError {
    InvalidField(String),
    MissingField(String),
}

impl StdError for BuilderError {}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            BuilderError::InvalidField(ref s) => write!(f, "InvalidField: {}", s),
            BuilderError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that the `CreateCircuitBuilder` works properly and builds a correct `CreateCircuit`
    /// when all fields are set.
    #[test]
    fn circuit_builder_successful() {
        let mut builder = CreateCircuitBuilder::new();
        assert!(builder.circuit_id().is_none());
        assert!(builder.roster().is_none());
        assert!(builder.members().is_none());
        assert!(builder.authorization_type().is_none());
        assert!(builder.persistence().is_none());
        assert!(builder.durability().is_none());
        assert!(builder.routes().is_none());
        assert!(builder.circuit_management_type().is_none());
        assert!(builder.application_metadata().is_none());

        let service = SplinterServiceBuilder::new()
            .with_service_type("service_type")
            .with_allowed_nodes(&["node_id".into()])
            .build()
            .expect("failed to build service");
        let node = SplinterNodeBuilder::new()
            .with_node_id("node_id")
            .with_endpoint("endpoint")
            .build()
            .expect("failed to build node");
        builder = builder
            .with_circuit_id("0123a-bcDEF")
            .with_roster(&[service.clone()])
            .with_members(&[node.clone()])
            .with_authorization_type(&AuthorizationType::Trust)
            .with_persistence(&PersistenceType::Any)
            .with_durability(&DurabilityType::NoDurability)
            .with_routes(&RouteType::Any)
            .with_circuit_management_type("mgmt_type")
            .with_application_metadata(b"abcd");
        assert_eq!(builder.circuit_id(), Some("0123a-bcDEF".into()));
        assert_eq!(builder.roster(), Some(vec![service.clone()]));
        assert_eq!(builder.members(), Some(vec![node.clone()]));
        assert_eq!(builder.authorization_type(), Some(AuthorizationType::Trust));
        assert_eq!(builder.persistence(), Some(PersistenceType::Any));
        assert_eq!(builder.durability(), Some(DurabilityType::NoDurability));
        assert_eq!(builder.routes(), Some(RouteType::Any));
        assert_eq!(builder.circuit_management_type(), Some("mgmt_type".into()));
        assert_eq!(builder.application_metadata(), Some(b"abcd".to_vec()));

        let circuit = builder.build().expect("failed to build circuit");
        assert_eq!(&circuit.circuit_id, "0123a-bcDEF");
        assert_eq!(circuit.roster, vec![service]);
        assert_eq!(circuit.members, vec![node]);
        assert_eq!(circuit.authorization_type, AuthorizationType::Trust);
        assert_eq!(circuit.persistence, PersistenceType::Any);
        assert_eq!(circuit.durability, DurabilityType::NoDurability);
        assert_eq!(circuit.routes, RouteType::Any);
        assert_eq!(&circuit.circuit_management_type, "mgmt_type");
        assert_eq!(&circuit.application_metadata, b"abcd");
    }

    /// Verify that the `CreateCircuitBuilder` builds a correct `CreateCircuit` when `circuit_id`,
    /// `authorization_type`, `persistence`, `durability`, `routes`, and `application_metadata` are
    /// unset.
    #[test]
    fn circuit_builder_successful_with_defaults() {
        let service = SplinterServiceBuilder::new()
            .with_service_type("service_type")
            .with_allowed_nodes(&["node_id".into()])
            .build()
            .expect("failed to build service");
        let node = SplinterNodeBuilder::new()
            .with_node_id("node_id")
            .with_endpoint("endpoint")
            .build()
            .expect("failed to build node");
        let circuit = CreateCircuitBuilder::new()
            .with_roster(&[service.clone()])
            .with_members(&[node.clone()])
            .with_circuit_management_type("mgmt_type")
            .build()
            .expect("failed to build circuit");

        assert_eq!(circuit.circuit_id.len(), 11);
        assert_eq!(circuit.circuit_id.chars().nth(5), Some('-'));
        assert!(circuit.circuit_id[0..5]
            .chars()
            .all(|c| c.is_ascii_alphanumeric()));
        assert!(circuit.circuit_id[6..11]
            .chars()
            .all(|c| c.is_ascii_alphanumeric()));
        assert_eq!(circuit.roster, vec![service]);
        assert_eq!(circuit.members, vec![node]);
        assert_eq!(circuit.authorization_type, AuthorizationType::Trust);
        assert_eq!(circuit.persistence, PersistenceType::Any);
        assert_eq!(circuit.durability, DurabilityType::NoDurability);
        assert_eq!(circuit.routes, RouteType::Any);
        assert_eq!(&circuit.circuit_management_type, "mgmt_type");
        assert!(circuit.application_metadata.is_empty());
    }

    /// Verify that the `CreateCircuitBuilder` fails to build when an invalid `circuit_id` is
    /// given.
    #[test]
    fn circuit_builder_invalid_circuit_id() {
        let service = SplinterServiceBuilder::new()
            .with_service_type("service_type")
            .with_allowed_nodes(&["node_id".into()])
            .build()
            .expect("failed to build service");
        let node = SplinterNodeBuilder::new()
            .with_node_id("node_id")
            .with_endpoint("endpoint")
            .build()
            .expect("failed to build node");
        let builder = CreateCircuitBuilder::new()
            .with_roster(&[service.clone()])
            .with_members(&[node.clone()])
            .with_circuit_management_type("mgmt_type");

        // Empty string
        match builder.clone().with_circuit_id("").build() {
            Ok(circuit) => panic!("Build did not fail; got circuit: {:?}", circuit),
            Err(BuilderError::InvalidField(_)) => {}
            Err(err) => panic!("Got unexpected error: {}", err),
        }

        // Too short
        match builder.clone().with_circuit_id("0123-bcDE").build() {
            Ok(circuit) => panic!("Build did not fail; got circuit: {:?}", circuit),
            Err(BuilderError::InvalidField(_)) => {}
            Err(err) => panic!("Got unexpected error: {}", err),
        }

        // Too long
        match builder.clone().with_circuit_id("0123a-bcDEFG").build() {
            Ok(circuit) => panic!("Build did not fail; got circuit: {:?}", circuit),
            Err(BuilderError::InvalidField(_)) => {}
            Err(err) => panic!("Got unexpected error: {}", err),
        }

        // No dash
        match builder.clone().with_circuit_id("0123abcDEFG").build() {
            Ok(circuit) => panic!("Build did not fail; got circuit: {:?}", circuit),
            Err(BuilderError::InvalidField(_)) => {}
            Err(err) => panic!("Got unexpected error: {}", err),
        }

        // Invalid character
        match builder.clone().with_circuit_id("0123a-bc:EF").build() {
            Ok(circuit) => panic!("Build did not fail; got circuit: {:?}", circuit),
            Err(BuilderError::InvalidField(_)) => {}
            Err(err) => panic!("Got unexpected error: {}", err),
        }
    }

    /// Verify that the `CreateCircuitBuilder` fails to build when `roster` is not set.
    #[test]
    fn circuit_builder_unset_roster() {
        let node = SplinterNodeBuilder::new()
            .with_node_id("node_id")
            .with_endpoint("endpoint")
            .build()
            .expect("failed to build node");
        let builder = CreateCircuitBuilder::new()
            .with_members(&[node])
            .with_circuit_management_type("mgmt_type");
        match builder.build() {
            Ok(circuit) => panic!("Build did not fail; got circuit: {:?}", circuit),
            Err(BuilderError::MissingField(_)) => {}
            Err(err) => panic!("Got unexpected error: {}", err),
        }
    }

    /// Verify that the `CreateCircuitBuilder` fails to build when `members` is not set.
    #[test]
    fn circuit_builder_unset_members() {
        let service = SplinterServiceBuilder::new()
            .with_service_type("service_type")
            .with_allowed_nodes(&["node_id".into()])
            .build()
            .expect("failed to build service");
        let builder = CreateCircuitBuilder::new()
            .with_roster(&[service])
            .with_circuit_management_type("mgmt_type");
        match builder.build() {
            Ok(circuit) => panic!("Build did not fail; got circuit: {:?}", circuit),
            Err(BuilderError::MissingField(_)) => {}
            Err(err) => panic!("Got unexpected error: {}", err),
        }
    }

    /// Verify that the `CreateCircuitBuilder` fails to build when `circuit_management_type` is not
    /// set.
    #[test]
    fn circuit_builder_unset_circuit_management_type() {
        let service = SplinterServiceBuilder::new()
            .with_service_type("service_type")
            .with_allowed_nodes(&["node_id".into()])
            .build()
            .expect("failed to build service");
        let node = SplinterNodeBuilder::new()
            .with_node_id("node_id")
            .with_endpoint("endpoint")
            .build()
            .expect("failed to build node");
        let builder = CreateCircuitBuilder::new()
            .with_roster(&[service])
            .with_members(&[node]);
        match builder.build() {
            Ok(circuit) => panic!("Build did not fail; got circuit: {:?}", circuit),
            Err(BuilderError::MissingField(_)) => {}
            Err(err) => panic!("Got unexpected error: {}", err),
        }
    }

    /// Verify that the `SplinterServiceBuilder` works properly and builds a correct
    /// `SplinterService` when all fields are set.
    #[test]
    fn service_builder_successful() {
        let mut builder = SplinterServiceBuilder::new();
        assert!(builder.service_id().is_none());
        assert!(builder.service_type().is_none());
        assert!(builder.allowed_nodes().is_none());
        assert!(builder.arguments().is_none());

        builder = builder
            .with_service_id("0aZ9")
            .with_service_type("service_type")
            .with_allowed_nodes(&["node_id".into()])
            .with_arguments(&[("key".into(), "value".into())]);
        assert_eq!(builder.service_id(), Some("0aZ9".into()));
        assert_eq!(builder.service_type(), Some("service_type".into()));
        assert_eq!(builder.allowed_nodes(), Some(vec!["node_id".into()]));
        assert_eq!(
            builder.arguments(),
            Some(vec![("key".into(), "value".into())])
        );

        let service = builder.build().expect("failed to build service");
        assert_eq!(&service.service_id, "0aZ9");
        assert_eq!(&service.service_type, "service_type");
        assert_eq!(&service.allowed_nodes, &["node_id".to_string()]);
        assert_eq!(&service.arguments, &[("key".into(), "value".into())]);
    }

    /// Verify that the `SplinterServiceBuilder` builds a correct `SplinterService` when
    /// `service_id` and `arguments` are unset.
    #[test]
    fn service_builder_successful_with_defaults() {
        let service = SplinterServiceBuilder::new()
            .with_service_type("service_type")
            .with_allowed_nodes(&["node_id".into()])
            .build()
            .expect("failed to build service");

        assert_eq!(service.service_id.len(), 4);
        assert!(service
            .service_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric()));
        assert_eq!(&service.service_type, "service_type");
        assert_eq!(&service.allowed_nodes, &["node_id".to_string()]);
        assert_eq!(&service.arguments, &[]);
    }

    /// Verify that the `SplinterServiceBuilder` fails to build when an invalid `service_id` is
    /// given.
    #[test]
    fn service_builder_invalid_service_id() {
        let builder = SplinterServiceBuilder::new()
            .with_service_type("service_type")
            .with_allowed_nodes(&["node_id".into()]);

        // Empty string
        match builder.clone().with_service_id("").build() {
            Ok(service) => panic!("Build did not fail; got service: {:?}", service),
            Err(BuilderError::InvalidField(_)) => {}
            Err(err) => panic!("Got unexpected error: {}", err),
        }

        // Too short
        match builder.clone().with_service_id("abc").build() {
            Ok(service) => panic!("Build did not fail; got service: {:?}", service),
            Err(BuilderError::InvalidField(_)) => {}
            Err(err) => panic!("Got unexpected error: {}", err),
        }

        // Too long
        match builder.clone().with_service_id("toolong").build() {
            Ok(service) => panic!("Build did not fail; got service: {:?}", service),
            Err(BuilderError::InvalidField(_)) => {}
            Err(err) => panic!("Got unexpected error: {}", err),
        }

        // Invalid character
        match builder.with_service_id("ab:c").build() {
            Ok(service) => panic!("Build did not fail; got service: {:?}", service),
            Err(BuilderError::InvalidField(_)) => {}
            Err(err) => panic!("Got unexpected error: {}", err),
        }
    }

    /// Verify that the `SplinterServiceBuilder` fails to build when `service_type` is not set.
    #[test]
    fn service_builder_unset_service_type() {
        match SplinterServiceBuilder::new()
            .with_allowed_nodes(&["node_id".into()])
            .build()
        {
            Ok(service) => panic!("Build did not fail; got service: {:?}", service),
            Err(BuilderError::MissingField(_)) => {}
            Err(err) => panic!("Got unexpected error: {}", err),
        }
    }

    /// Verify that the `SplinterServiceBuilder` fails to build when `allowed_nodes` is not set.
    #[test]
    fn service_builder_unset_allowed_nodes() {
        match SplinterServiceBuilder::new()
            .with_service_type("service_type")
            .build()
        {
            Ok(service) => panic!("Build did not fail; got service: {:?}", service),
            Err(BuilderError::MissingField(_)) => {}
            Err(err) => panic!("Got unexpected error: {}", err),
        }
    }

    /// Verify that the `SplinterNodeBuilder` works properly and builds a correct `SplinterNode`.
    #[test]
    fn node_builder_success() {
        let mut builder = SplinterNodeBuilder::new();
        assert!(builder.node_id().is_none());
        assert!(builder.endpoint().is_none());

        builder = builder.with_node_id("node_id").with_endpoint("endpoint");
        assert_eq!(builder.node_id(), Some("node_id".into()));
        assert_eq!(builder.endpoint(), Some("endpoint".into()));

        let node = builder.build().expect("failed to build node");
        assert_eq!(&node.node_id, "node_id");
        assert_eq!(&node.endpoint, "endpoint");
    }

    /// Verify that the `SplinterNodeBuilder` fails to build when `node_id` is not set.
    #[test]
    fn node_builder_unset_node_id() {
        match SplinterNodeBuilder::new().with_endpoint("endpoint").build() {
            Ok(node) => panic!("Build did not fail; got node: {:?}", node),
            Err(BuilderError::MissingField(_)) => {}
            Err(err) => panic!("Got unexpected error: {}", err),
        }
    }

    /// Verify that the `SplinterNodeBuilder` fails to build when `endpoint` is not set.
    #[test]
    fn node_builder_unset_endpoint() {
        match SplinterNodeBuilder::new().with_node_id("node_id").build() {
            Ok(node) => panic!("Build did not fail; got node: {:?}", node),
            Err(BuilderError::MissingField(_)) => {}
            Err(err) => panic!("Got unexpected error: {}", err),
        }
    }
}
