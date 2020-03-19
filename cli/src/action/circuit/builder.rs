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

#[cfg(feature = "node-alias")]
use super::super::node::get_node_store;
#[cfg(feature = "node-alias")]
use crate::store::node::NodeStore;

use super::defaults::{get_default_value_store, MANAGEMENT_TYPE_KEY, SERVICE_TYPE_KEY};
use crate::error::CliError;
use crate::store::default_value::DefaultValueStore;

use splinter::admin::messages::{
    AuthorizationType, CreateCircuit, CreateCircuitBuilder, SplinterNode, SplinterNodeBuilder,
    SplinterServiceBuilder,
};

const PEER_SERVICES_ARG: &str = "peer_services";

pub struct CreateCircuitMessageBuilder {
    create_circuit_builder: CreateCircuitBuilder,
    services: Vec<SplinterServiceBuilder>,
    nodes: Vec<SplinterNode>,
    management_type: Option<String>,
    #[cfg(feature = "circuit-auth-type")]
    authorization_type: Option<AuthorizationType>,
    application_metadata: Vec<u8>,
    comments: Option<String>,
}

impl CreateCircuitMessageBuilder {
    pub fn new() -> CreateCircuitMessageBuilder {
        CreateCircuitMessageBuilder {
            create_circuit_builder: CreateCircuitBuilder::new(),
            services: vec![],
            nodes: vec![],
            management_type: None,
            #[cfg(feature = "circuit-auth-type")]
            authorization_type: None,
            application_metadata: vec![],
            comments: None,
        }
    }

    #[cfg(feature = "circuit-template")]
    pub fn add_services(&mut self, service_builders: &[SplinterServiceBuilder]) {
        self.services.extend(service_builders.to_owned());
    }

    #[cfg(feature = "circuit-template")]
    pub fn set_create_circuit_builder(&mut self, create_circuit_builder: &CreateCircuitBuilder) {
        self.create_circuit_builder = create_circuit_builder.clone()
    }

    #[cfg(feature = "circuit-template")]
    pub fn get_node_ids(&self) -> Vec<String> {
        self.nodes.iter().map(|node| node.node_id.clone()).collect()
    }

    pub fn apply_service_type(&mut self, service_id_match: &str, service_type: &str) {
        self.services = self
            .services
            .clone()
            .into_iter()
            .map(|service_builder| {
                let service_id = service_builder.service_id().unwrap_or_default();
                if is_match(service_id_match, &service_id) {
                    service_builder.with_service_type(service_type)
                } else {
                    service_builder
                }
            })
            .collect();
    }

    pub fn apply_service_arguments(
        &mut self,
        service_id_match: &str,
        args: &(String, String),
    ) -> Result<(), CliError> {
        self.services = self.services.clone().into_iter().try_fold(
            Vec::new(),
            |mut acc, service_builder| {
                let service_id = service_builder.service_id().unwrap_or_default();
                if is_match(service_id_match, &service_id) {
                    let mut service_args = service_builder.arguments().unwrap_or_default();
                    if args.0 == PEER_SERVICES_ARG
                        && service_args.iter().any(|arg| arg.0 == PEER_SERVICES_ARG)
                    {
                        return Err(CliError::ActionError(format!(
                            "Peer services argument is already set for service: {}",
                            service_id
                        )));
                    }
                    service_args.push(args.clone());
                    acc.push(service_builder.with_arguments(&service_args));
                } else {
                    acc.push(service_builder);
                }
                Ok(acc)
            },
        )?;
        Ok(())
    }

    pub fn apply_peer_services(&mut self, service_id_globs: &[&str]) -> Result<(), CliError> {
        let peers = self
            .services
            .iter()
            .filter_map(|service_builder| {
                let service_id = service_builder.service_id().unwrap_or_default();
                if service_id_globs
                    .iter()
                    .any(|glob| is_match(glob, &service_id))
                {
                    Some(service_id)
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();

        self.services = self.services.clone().into_iter().try_fold(
            Vec::new(),
            |mut acc, service_builder| {
                let service_id = service_builder.service_id().unwrap_or_default();
                let index = peers.iter().enumerate().find_map(|(index, peer_id)| {
                    if peer_id == &service_id {
                        Some(index)
                    } else {
                        None
                    }
                });

                if let Some(index) = index {
                    let mut service_peers = peers.clone();
                    service_peers.remove(index);
                    let mut service_args = service_builder.arguments().unwrap_or_default();
                    if service_args.iter().any(|arg| arg.0 == PEER_SERVICES_ARG) {
                        return Err(CliError::ActionError(format!(
                            "Peer services argument for service {} is already set.",
                            service_id
                        )));
                    }
                    service_args.push((PEER_SERVICES_ARG.into(), format!("{:?}", service_peers)));
                    acc.push(service_builder.with_arguments(&service_args));
                } else {
                    acc.push(service_builder);
                }
                Ok(acc)
            },
        )?;
        Ok(())
    }

    #[cfg(feature = "node-alias")]
    pub fn add_node_by_alias(&mut self, alias: &str) -> Result<(), CliError> {
        let node_store = get_node_store();
        let (node_id, endpoint) = match node_store.get_node(alias)? {
            Some(node) => (node.node_id(), node.endpoint()),
            None => {
                return Err(CliError::ActionError(format!(
                    "No endpoint provided and an alias for node {} has not been set",
                    alias
                )))
            }
        };

        let node = make_splinter_node(&node_id, &endpoint)?;
        self.nodes.push(node);
        Ok(())
    }

    pub fn add_node(&mut self, node_id: &str, node_endpoint: &str) -> Result<(), CliError> {
        let node = make_splinter_node(node_id, node_endpoint)?;
        self.nodes.push(node);
        Ok(())
    }

    pub fn set_management_type(&mut self, management_type: &str) {
        self.management_type = Some(management_type.into());
    }

    #[cfg(feature = "circuit-auth-type")]
    pub fn set_authorization_type(&mut self, authorization_type: &str) -> Result<(), CliError> {
        let auth_type = match authorization_type {
            "trust" => AuthorizationType::Trust,
            _ => {
                return Err(CliError::ActionError(format!(
                    "Invalid authorization type {}",
                    authorization_type
                )))
            }
        };

        self.authorization_type = Some(auth_type);
        Ok(())
    }

    pub fn set_application_metadata(&mut self, application_metadata: &[u8]) {
        self.application_metadata = application_metadata.into();
    }

    pub fn set_comments(&mut self, comments: &str) {
        self.comments = Some(comments.into());
    }

    pub fn build(self) -> Result<CreateCircuit, CliError> {
        let default_store = get_default_value_store();

        // if management type is not set check for default value
        let management_type = match self.management_type {
            Some(management_type) => management_type,
            None => match self.create_circuit_builder.circuit_management_type() {
                Some(management_type) => management_type,
                None => match default_store.get_default_value(MANAGEMENT_TYPE_KEY)? {
                    Some(management_type) => management_type.value(),
                    None => {
                        return Err(CliError::ActionError(
                            "Management type not provided and no default value set".to_string(),
                        ))
                    }
                },
            },
        };

        let services =
            self.services
                .into_iter()
                .try_fold(Vec::new(), |mut services, mut builder| {
                    // if service type is not set, check for default value
                    if builder.service_type().is_none() {
                        builder = match default_store.get_default_value(SERVICE_TYPE_KEY)? {
                            Some(service_type) => builder.with_service_type(&service_type.value()),
                            None => {
                                return Err(CliError::ActionError(
                                    "Service has no service type and no default value is set"
                                        .to_string(),
                                ))
                            }
                        }
                    }

                    let service = builder.build().map_err(|err| {
                        CliError::ActionError(format!("Failed to build service: {}", err))
                    })?;
                    services.push(service);
                    Ok(services)
                })?;

        let mut create_circuit_builder = self
            .create_circuit_builder
            .with_members(&self.nodes)
            .with_roster(&services)
            .with_circuit_management_type(&management_type)
            .with_comments(&self.comments.unwrap_or_default());

        if !self.application_metadata.is_empty() {
            create_circuit_builder =
                create_circuit_builder.with_application_metadata(&self.application_metadata);
        }

        #[cfg(not(feature = "circuit-auth-type"))]
        let create_circuit_builder =
            create_circuit_builder.with_authorization_type(&AuthorizationType::Trust);

        #[cfg(feature = "circuit-auth-type")]
        let create_circuit_builder = match self.authorization_type {
            Some(authorization_type) => {
                create_circuit_builder.with_authorization_type(&authorization_type)
            }
            None => create_circuit_builder,
        };

        let create_circuit = create_circuit_builder.build().map_err(|err| {
            CliError::ActionError(format!("Failed to build CreateCircuit message: {}", err))
        })?;
        Ok(create_circuit)
    }

    pub fn add_service(&mut self, service_id: &str, allowed_nodes: &[String]) {
        let service_builder = SplinterServiceBuilder::new()
            .with_service_id(service_id)
            .with_allowed_nodes(allowed_nodes);
        self.services.push(service_builder);
    }
}

fn is_match(service_id_match: &str, service_id: &str) -> bool {
    service_id_match.split('*').fold(true, |is_match, part| {
        if part.len() != service_id_match.len() {
            is_match && service_id.contains(part)
        } else {
            service_id == part
        }
    })
}

fn make_splinter_node(node_id: &str, endpoint: &str) -> Result<SplinterNode, CliError> {
    let node = SplinterNodeBuilder::new()
        .with_node_id(&node_id)
        .with_endpoint(&endpoint)
        .build()
        .map_err(|err| CliError::ActionError(format!("Failed to build SplinterNode: {}", err)))?;
    Ok(node)
}
