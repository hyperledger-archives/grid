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

use super::{
    AuthorizationType, CreateCircuit, DurabilityType, PersistenceType, RouteType, SplinterNode,
    SplinterService,
};
use std::error::Error as StdError;

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
        let circuit_id = self.circuit_id.ok_or_else(|| {
            BuilderError::MissingField(
                "Unable to build CreateCircuit message. Missing required field circuit_id"
                    .to_string(),
            )
        })?;

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
        let service_id = self.service_id.ok_or_else(|| {
            BuilderError::MissingField(
                "Unable to build SplinterService. Missing required field service_id".to_string(),
            )
        })?;

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
    MissingField(String),
}

impl StdError for BuilderError {}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            BuilderError::MissingField(ref s) => write!(f, "MissingField: {}", s),
        }
    }
}
