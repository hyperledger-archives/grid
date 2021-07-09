/*
 * Copyright 2019 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use diesel::Connection;
use grid_sdk::commits::{store::CommitEvent as DbCommitEvent, CommitStore, DieselCommitStore};
#[cfg(any(feature = "location", feature = "product"))]
use grid_sdk::protocol::schema::state::PropertyValue;
#[cfg(any(
    feature = "pike",
    feature = "schema",
    feature = "product",
    feature = "location"
))]
use grid_sdk::protos::FromBytes;
#[cfg(feature = "location")]
use grid_sdk::{
    location::{
        addressing::GRID_LOCATION_NAMESPACE,
        store::{
            DieselLocationStore, LatLongValue as LocationLatLongValue, Location, LocationAttribute,
            LocationStore,
        },
    },
    protocol::location::state::LocationList,
};

#[cfg(feature = "pike")]
use grid_sdk::{
    pike::{
        addressing::{
            PIKE_AGENT_NAMESPACE, PIKE_NAMESPACE, PIKE_ORGANIZATION_NAMESPACE, PIKE_ROLE_NAMESPACE,
        },
        store::{
            Agent, AgentBuilder, AlternateId, DieselPikeStore, Organization, OrganizationBuilder,
            OrganizationMetadata, PikeStore, Role, RoleBuilder,
        },
    },
    protocol::pike::state::{AgentList, OrganizationList, RoleList},
};
#[cfg(feature = "product")]
use grid_sdk::{
    product::{
        addressing::GRID_PRODUCT_NAMESPACE,
        store::{
            DieselProductStore, LatLongValue as ProductLatLongValue, Product, ProductBuilder,
            ProductStore, PropertyValue as ProductPropertyValue,
            PropertyValueBuilder as ProductPropertyValueBuilder,
        },
    },
    protocol::product::state::ProductList,
};
#[cfg(feature = "track-and-trace")]
use grid_sdk::{
    protocol::schema::state::DataType,
    protocol::track_and_trace::state::{
        PropertyList, PropertyPageList, ProposalList, RecordList, ReportedValue,
    },
    track_and_trace::{
        addressing::{
            TRACK_AND_TRACE_PROPERTY_NAMESPACE, TRACK_AND_TRACE_PROPOSAL_NAMESPACE,
            TRACK_AND_TRACE_RECORD_NAMESPACE,
        },
        store::{
            AssociatedAgent, DieselTrackAndTraceStore, LatLongValue as TntLatLongValue, Property,
            Proposal, Record, ReportedValue as StoreReportedValue, Reporter, TrackAndTraceStore,
        },
    },
};
#[cfg(feature = "schema")]
use grid_sdk::{
    protocol::schema::state::{PropertyDefinition, SchemaList},
    schema::{
        addressing::GRID_SCHEMA_NAMESPACE,
        store::{
            DieselSchemaStore, PropertyDefinition as StorePropertyDefinition, Schema, SchemaStore,
        },
    },
};
#[cfg(feature = "pike")]
use std::collections::HashMap;
use std::i64;

use crate::database::ConnectionPool;

use super::{CommitEvent, EventError, EventHandler, StateChange, IGNORED_NAMESPACES};

#[cfg(any(
    feature = "pike",
    feature = "schema",
    feature = "product",
    feature = "location"
))]
pub const MAX_COMMIT_NUM: i64 = i64::MAX;

pub struct DatabaseEventHandler<C: diesel::Connection + 'static> {
    connection_pool: ConnectionPool<C>,
    commit_store: DieselCommitStore<C>,
    #[cfg(feature = "pike")]
    pike_store: DieselPikeStore<C>,
    #[cfg(feature = "location")]
    location_store: DieselLocationStore<C>,
    #[cfg(feature = "product")]
    product_store: DieselProductStore<C>,
    #[cfg(feature = "schema")]
    schema_store: DieselSchemaStore<C>,
    #[cfg(feature = "track-and-trace")]
    tnt_store: DieselTrackAndTraceStore<C>,
}

#[cfg(feature = "database-postgres")]
impl DatabaseEventHandler<diesel::pg::PgConnection> {
    pub fn from_pg_pool(connection_pool: ConnectionPool<diesel::pg::PgConnection>) -> Self {
        let commit_store = DieselCommitStore::new(connection_pool.pool.clone());
        #[cfg(feature = "pike")]
        let pike_store = DieselPikeStore::new(connection_pool.pool.clone());
        #[cfg(feature = "location")]
        let location_store = DieselLocationStore::new(connection_pool.pool.clone());
        #[cfg(feature = "product")]
        let product_store = DieselProductStore::new(connection_pool.pool.clone());
        #[cfg(feature = "schema")]
        let schema_store = DieselSchemaStore::new(connection_pool.pool.clone());
        #[cfg(feature = "track-and-trace")]
        let tnt_store = DieselTrackAndTraceStore::new(connection_pool.pool.clone());

        Self {
            connection_pool,
            commit_store,
            #[cfg(feature = "pike")]
            pike_store,
            #[cfg(feature = "location")]
            location_store,
            #[cfg(feature = "product")]
            product_store,
            #[cfg(feature = "schema")]
            schema_store,
            #[cfg(feature = "track-and-trace")]
            tnt_store,
        }
    }
}

#[cfg(feature = "database-postgres")]
impl EventHandler for DatabaseEventHandler<diesel::pg::PgConnection> {
    fn handle_event(&self, event: &CommitEvent) -> Result<(), EventError> {
        debug!("Received commit event: {}", event);

        let conn = self
            .connection_pool
            .get()
            .map_err(|err| EventError(format!("Unable to connect to database: {}", err)))?;

        conn.build_transaction().run::<_, EventError, _>(|| {
            let commit = if let Some(commit) = self
                .commit_store
                .create_db_commit_from_commit_event(&DbCommitEvent::from(event))
                .map_err(|err| EventError(format!("{}", err)))?
            {
                commit
            } else {
                return Err(EventError(
                    "Commit could not be constructed from event data".to_string(),
                ));
            };
            let db_ops = create_db_operations_from_state_changes(
                &event.state_changes,
                commit.commit_num,
                commit.service_id.as_ref(),
            )?;

            trace!("The following operations will be performed: {:#?}", db_ops);

            match self
                .commit_store
                .get_commit_by_commit_num(commit.commit_num)
            {
                Ok(Some(ref b)) if b.commit_id != commit.commit_id => {
                    self.commit_store.resolve_fork(commit.commit_num)?;
                    info!(
                        "Fork detected. Replaced {} at height {}, with commit {}.",
                        &b.commit_id, &b.commit_num, &commit.commit_id
                    );
                    self.commit_store.add_commit(commit)?;
                }
                Ok(Some(_)) => {
                    info!(
                        "Commit {} at height {} is duplicate no action taken",
                        &commit.commit_id, commit.commit_num
                    );
                }
                Ok(None) => {
                    info!("Received new commit {}", commit.commit_id);
                    self.commit_store.add_commit(commit)?;
                }
                Err(err) => {
                    return Err(EventError::from(err));
                }
            }

            for op in db_ops {
                match op {
                    #[cfg(feature = "pike")]
                    DbInsertOperation::Agents(agents) => {
                        debug!("Inserting {} agent(s)", agents.len());
                        agents
                            .into_iter()
                            .try_for_each(|agent| self.pike_store.add_agent(agent))?;
                    }
                    #[cfg(feature = "pike")]
                    DbInsertOperation::Organizations(orgs) => {
                        debug!("Inserting {} organization(s)", orgs.len());
                        orgs.into_iter()
                            .try_for_each(|org| self.pike_store.add_organization(org))?;
                    }
                    #[cfg(feature = "pike")]
                    DbInsertOperation::Roles(roles) => {
                        debug!("Inserting {} role(s)", roles.len());
                        roles
                            .into_iter()
                            .try_for_each(|role| self.pike_store.add_role(role))?;
                    }
                    #[cfg(feature = "pike")]
                    DbInsertOperation::RemoveRole(ref address, current_commit_num) => {
                        debug!("Removing role at address {}", address);
                        self.pike_store.delete_role(address, current_commit_num)?;
                    }
                    #[cfg(feature = "schema")]
                    DbInsertOperation::GridSchemas(schemas) => {
                        debug!("Inserting {} schemas", schemas.len());
                        schemas
                            .into_iter()
                            .try_for_each(|schema| self.schema_store.add_schema(schema))?;
                    }
                    #[cfg(feature = "track-and-trace")]
                    DbInsertOperation::Properties(properties, reporters) => {
                        debug!("Inserting {} properties", properties.len());
                        self.tnt_store.add_properties(properties)?;
                        debug!("Inserting {} reporters", reporters.len());
                        self.tnt_store.add_reporters(reporters)?;
                    }
                    #[cfg(feature = "track-and-trace")]
                    DbInsertOperation::ReportedValues(reported_values) => {
                        debug!("Inserting {} reported values", reported_values.len());
                        self.tnt_store.add_reported_values(reported_values)?;
                    }
                    #[cfg(feature = "track-and-trace")]
                    DbInsertOperation::Proposals(proposals) => {
                        debug!("Inserting {} proposals", proposals.len());
                        self.tnt_store.add_proposals(proposals)?;
                    }
                    #[cfg(feature = "track-and-trace")]
                    DbInsertOperation::Records(records, associated_agents) => {
                        debug!("Inserting {} records", records.len());
                        self.tnt_store.add_records(records)?;
                        debug!("Inserting {} associated agents", associated_agents.len());
                        self.tnt_store.add_associated_agents(associated_agents)?;
                    }
                    #[cfg(feature = "location")]
                    DbInsertOperation::Locations(locations) => {
                        debug!("Inserting {} locations", locations.len());
                        locations
                            .into_iter()
                            .try_for_each(|location| self.location_store.add_location(location))?;
                    }
                    #[cfg(feature = "location")]
                    DbInsertOperation::RemoveLocation(ref address, current_commit_num) => {
                        self.location_store
                            .delete_location(address, current_commit_num)?;
                    }
                    #[cfg(feature = "product")]
                    DbInsertOperation::Products(products) => {
                        debug!("Inserting {} products", products.len());
                        products
                            .into_iter()
                            .try_for_each(|product| self.product_store.add_product(product))?;
                    }
                    #[cfg(feature = "product")]
                    DbInsertOperation::RemoveProduct(ref address, current_commit_num) => {
                        self.product_store
                            .delete_product(address, current_commit_num)?;
                    }
                };
            }

            Ok(())
        })
    }

    fn cloned_box(&self) -> Box<dyn EventHandler> {
        Box::new(Self::from_pg_pool(self.connection_pool.clone()))
    }
}

#[cfg(feature = "database-sqlite")]
impl DatabaseEventHandler<diesel::sqlite::SqliteConnection> {
    pub fn from_sqlite_pool(
        connection_pool: ConnectionPool<diesel::sqlite::SqliteConnection>,
    ) -> Self {
        let commit_store = DieselCommitStore::new(connection_pool.pool.clone());
        #[cfg(feature = "pike")]
        let pike_store = DieselPikeStore::new(connection_pool.pool.clone());
        #[cfg(feature = "location")]
        let location_store = DieselLocationStore::new(connection_pool.pool.clone());
        #[cfg(feature = "product")]
        let product_store = DieselProductStore::new(connection_pool.pool.clone());
        #[cfg(feature = "schema")]
        let schema_store = DieselSchemaStore::new(connection_pool.pool.clone());
        #[cfg(feature = "track-and-trace")]
        let tnt_store = DieselTrackAndTraceStore::new(connection_pool.pool.clone());

        Self {
            connection_pool,
            commit_store,
            #[cfg(feature = "pike")]
            pike_store,
            #[cfg(feature = "location")]
            location_store,
            #[cfg(feature = "product")]
            product_store,
            #[cfg(feature = "schema")]
            schema_store,
            #[cfg(feature = "track-and-trace")]
            tnt_store,
        }
    }
}

#[cfg(feature = "database-sqlite")]
impl EventHandler for DatabaseEventHandler<diesel::sqlite::SqliteConnection> {
    fn handle_event(&self, event: &CommitEvent) -> Result<(), EventError> {
        debug!("Received commit event: {}", event);

        let conn = self
            .connection_pool
            .get()
            .map_err(|err| EventError(format!("Unable to connect to database: {}", err)))?;

        conn.transaction::<_, EventError, _>(|| {
            let commit = if let Some(commit) = self
                .commit_store
                .create_db_commit_from_commit_event(&DbCommitEvent::from(event))
                .map_err(|err| EventError(format!("{}", err)))?
            {
                commit
            } else {
                return Err(EventError(
                    "Commit could not be constructed from event data".to_string(),
                ));
            };
            let db_ops = create_db_operations_from_state_changes(
                &event.state_changes,
                commit.commit_num,
                commit.service_id.as_ref(),
            )?;

            trace!("The following operations will be performed: {:#?}", db_ops);

            match self
                .commit_store
                .get_commit_by_commit_num(commit.commit_num)
            {
                Ok(Some(ref b)) if b.commit_id != commit.commit_id => {
                    self.commit_store.resolve_fork(commit.commit_num)?;
                    info!(
                        "Fork detected. Replaced {} at height {}, with commit {}.",
                        &b.commit_id, &b.commit_num, &commit.commit_id
                    );
                    self.commit_store.add_commit(commit)?;
                }
                Ok(Some(_)) => {
                    info!(
                        "Commit {} at height {} is duplicate no action taken",
                        &commit.commit_id, commit.commit_num
                    );
                }
                Ok(None) => {
                    info!("Received new commit {}", commit.commit_id);
                    self.commit_store.add_commit(commit)?;
                }
                Err(err) => {
                    return Err(EventError::from(err));
                }
            }

            for op in db_ops {
                match op {
                    #[cfg(feature = "pike")]
                    DbInsertOperation::Agents(agents) => {
                        debug!("Inserting {} agent(s)", agents.len());
                        agents
                            .into_iter()
                            .try_for_each(|agent| self.pike_store.add_agent(agent))?;
                    }
                    #[cfg(feature = "pike")]
                    DbInsertOperation::Organizations(orgs) => {
                        debug!("Inserting {} organization(s)", orgs.len());
                        orgs.into_iter()
                            .try_for_each(|org| self.pike_store.add_organization(org))?;
                    }
                    #[cfg(feature = "pike")]
                    DbInsertOperation::Roles(roles) => {
                        debug!("Inserting {} role(s)", roles.len());
                        roles
                            .into_iter()
                            .try_for_each(|role| self.pike_store.add_role(role))?;
                    }
                    #[cfg(feature = "pike")]
                    DbInsertOperation::RemoveRole(ref address, current_commit_num) => {
                        debug!("Removing role at address {}", &address);
                        self.pike_store.delete_role(address, current_commit_num)?;
                    }

                    #[cfg(feature = "schema")]
                    DbInsertOperation::GridSchemas(schemas) => {
                        debug!("Inserting {} schemas", schemas.len());
                        schemas
                            .into_iter()
                            .try_for_each(|schema| self.schema_store.add_schema(schema))?;
                    }
                    #[cfg(feature = "track-and-trace")]
                    DbInsertOperation::Properties(properties, reporters) => {
                        debug!("Inserting {} properties", properties.len());
                        self.tnt_store.add_properties(properties)?;
                        debug!("Inserting {} reporters", reporters.len());
                        self.tnt_store.add_reporters(reporters)?;
                    }
                    #[cfg(feature = "track-and-trace")]
                    DbInsertOperation::ReportedValues(reported_values) => {
                        debug!("Inserting {} reported values", reported_values.len());
                        self.tnt_store.add_reported_values(reported_values)?;
                    }
                    #[cfg(feature = "track-and-trace")]
                    DbInsertOperation::Proposals(proposals) => {
                        debug!("Inserting {} proposals", proposals.len());
                        self.tnt_store.add_proposals(proposals)?;
                    }
                    #[cfg(feature = "track-and-trace")]
                    DbInsertOperation::Records(records, associated_agents) => {
                        debug!("Inserting {} records", records.len());
                        self.tnt_store.add_records(records)?;
                        debug!("Inserting {} associated agents", associated_agents.len());
                        self.tnt_store.add_associated_agents(associated_agents)?;
                    }
                    #[cfg(feature = "location")]
                    DbInsertOperation::Locations(locations) => {
                        debug!("Inserting {} locations", locations.len());
                        locations
                            .into_iter()
                            .try_for_each(|location| self.location_store.add_location(location))?;
                    }
                    #[cfg(feature = "location")]
                    DbInsertOperation::RemoveLocation(ref address, current_commit_num) => {
                        self.location_store
                            .delete_location(address, current_commit_num)?;
                    }
                    #[cfg(feature = "product")]
                    DbInsertOperation::Products(products) => {
                        debug!("Inserting {} products", products.len());
                        products
                            .into_iter()
                            .try_for_each(|product| self.product_store.add_product(product))?;
                    }
                    #[cfg(feature = "product")]
                    DbInsertOperation::RemoveProduct(ref address, current_commit_num) => {
                        self.product_store
                            .delete_product(address, current_commit_num)?;
                    }
                };
            }

            Ok(())
        })
    }

    fn cloned_box(&self) -> Box<dyn EventHandler> {
        Box::new(Self::from_sqlite_pool(self.connection_pool.clone()))
    }
}

fn create_db_operations_from_state_changes(
    state_changes: &[StateChange],
    commit_num: i64,
    service_id: Option<&String>,
) -> Result<Vec<DbInsertOperation>, EventError> {
    state_changes
        .iter()
        .filter_map(|state_change| {
            state_change_to_db_operation(state_change, commit_num, service_id).transpose()
        })
        .collect::<Result<Vec<DbInsertOperation>, EventError>>()
}

#[allow(unused_variables)]
fn state_change_to_db_operation(
    state_change: &StateChange,
    commit_num: i64,
    service_id: Option<&String>,
) -> Result<Option<DbInsertOperation>, EventError> {
    #[allow(clippy::match_single_binding)]
    match state_change {
        StateChange::Set { key, value } => match &key[0..8] {
            #[cfg(feature = "pike")]
            PIKE_NAMESPACE => match &key[0..10] {
                PIKE_AGENT_NAMESPACE => {
                    let agents: Vec<Agent> = AgentList::from_bytes(&value)
                        .map_err(|err| EventError(format!("Failed to parse agent list {}", err)))?
                        .agents()
                        .iter()
                        .map(|agent| {
                            let mut builder = AgentBuilder::new()
                                .with_public_key(agent.public_key().to_string())
                                .with_org_id(agent.org_id().to_string())
                                .with_active(*agent.active())
                                .with_metadata(
                                    json!(agent.metadata().iter().fold(
                                        HashMap::new(),
                                        |mut acc, md| {
                                            acc.insert(
                                                md.key().to_string(),
                                                md.value().to_string(),
                                            );
                                            acc
                                        }
                                    ))
                                    .to_string()
                                    .into_bytes(),
                                )
                                .with_roles(agent.roles().to_vec())
                                .with_start_commit_num(commit_num)
                                .with_end_commit_num(MAX_COMMIT_NUM);
                            if let Some(service_id) = service_id {
                                builder = builder.with_service_id(service_id.to_string());
                            }
                            builder.build().map_err(|err| EventError(err.to_string()))
                        })
                        .collect::<Result<Vec<Agent>, EventError>>()?;

                    Ok(Some(DbInsertOperation::Agents(agents)))
                }
                PIKE_ORGANIZATION_NAMESPACE => {
                    let orgs = OrganizationList::from_bytes(&value)
                        .map_err(|err| {
                            EventError(format!("Failed to parse organization list {}", err))
                        })?
                        .organizations()
                        .iter()
                        .map(|org| {
                            let alt_ids = org
                                .alternate_ids()
                                .iter()
                                .map(|a| AlternateId {
                                    org_id: org.org_id().to_string(),
                                    alternate_id_type: a.id_type().to_string(),
                                    alternate_id: a.id().to_string(),
                                    start_commit_num: commit_num,
                                    end_commit_num: MAX_COMMIT_NUM,
                                    service_id: service_id.cloned(),
                                })
                                .collect();
                            let metadata = org
                                .metadata()
                                .iter()
                                .map(|md| OrganizationMetadata {
                                    key: md.key().to_string(),
                                    value: md.value().to_string(),
                                    start_commit_num: commit_num,
                                    end_commit_num: MAX_COMMIT_NUM,
                                    service_id: service_id.cloned(),
                                })
                                .collect();
                            let mut builder = OrganizationBuilder::new()
                                .with_org_id(org.org_id().to_string())
                                .with_name(org.name().to_string())
                                .with_locations(org.locations().to_vec())
                                .with_alternate_ids(alt_ids)
                                .with_metadata(metadata)
                                .with_start_commit_num(commit_num)
                                .with_end_commit_num(MAX_COMMIT_NUM);
                            if let Some(id) = service_id {
                                builder = builder.with_service_id(id.to_string());
                            }
                            builder.build().map_err(|err| EventError(err.to_string()))
                        })
                        .collect::<Result<Vec<Organization>, EventError>>()?;

                    Ok(Some(DbInsertOperation::Organizations(orgs)))
                }
                PIKE_ROLE_NAMESPACE => {
                    let roles = RoleList::from_bytes(&value)
                        .map_err(|err| EventError(format!("Failed to parse role list {}", err)))?
                        .roles()
                        .iter()
                        .map(|role| {
                            let mut builder = RoleBuilder::new()
                                .with_org_id(role.org_id().to_string())
                                .with_name(role.name().to_string())
                                .with_description(role.description().to_string())
                                .with_active(*role.active())
                                .with_permissions(role.permissions().to_vec())
                                .with_allowed_organizations(role.allowed_organizations().to_vec())
                                .with_inherit_from(role.inherit_from().to_vec())
                                .with_start_commit_num(commit_num)
                                .with_end_commit_num(MAX_COMMIT_NUM);
                            if let Some(id) = service_id {
                                builder = builder.with_service_id(id.to_string());
                            }
                            builder.build().map_err(|err| EventError(err.to_string()))
                        })
                        .collect::<Result<Vec<Role>, EventError>>()?;

                    Ok(Some(DbInsertOperation::Roles(roles)))
                }
                _ => {
                    debug!("received state change for unknown address: {}", key);
                    Ok(None)
                }
            },
            #[cfg(feature = "schema")]
            GRID_SCHEMA_NAMESPACE => {
                let schemas = SchemaList::from_bytes(&value)
                    .map_err(|err| EventError(format!("Failed to parse schema list {}", err)))?
                    .schemas()
                    .iter()
                    .map(|state_schema| Schema {
                        name: state_schema.name().to_string(),
                        description: state_schema.description().to_string(),
                        owner: state_schema.owner().to_string(),
                        start_commit_num: commit_num,
                        end_commit_num: MAX_COMMIT_NUM,
                        service_id: service_id.cloned(),
                        last_updated: None,
                        properties: make_property_definitions(
                            commit_num,
                            service_id,
                            state_schema.name(),
                            state_schema.properties(),
                        ),
                    })
                    .collect();

                Ok(Some(DbInsertOperation::GridSchemas(schemas)))
            }
            #[cfg(feature = "track-and-trace")]
            TRACK_AND_TRACE_PROPERTY_NAMESPACE if &key[66..] == "0000" => {
                let properties = PropertyList::from_bytes(&value)
                    .map_err(|err| EventError(format!("Failed to parse property list {}", err)))?
                    .properties()
                    .iter()
                    .map(|prop| {
                        let property = Property {
                            id: None,
                            name: prop.name().to_string(),
                            record_id: prop.record_id().to_string(),
                            property_definition: prop.property_definition().name().to_string(),
                            current_page: *prop.current_page() as i32,
                            wrapped: *prop.wrapped(),
                            start_commit_num: commit_num,
                            end_commit_num: MAX_COMMIT_NUM,
                            service_id: service_id.cloned(),
                        };

                        let reporters = prop
                            .reporters()
                            .iter()
                            .map(|reporter| Reporter {
                                id: None,
                                property_name: prop.name().to_string(),
                                record_id: prop.record_id().to_string(),
                                public_key: reporter.public_key().to_string(),
                                authorized: *reporter.authorized(),
                                reporter_index: *reporter.index() as i32,
                                start_commit_num: commit_num,
                                end_commit_num: MAX_COMMIT_NUM,
                                service_id: service_id.cloned(),
                            })
                            .collect::<Vec<Reporter>>();

                        (property, reporters)
                    })
                    .collect::<Vec<(Property, Vec<Reporter>)>>();

                let reporters = properties
                    .clone()
                    .into_iter()
                    .flat_map(|(_, r)| r.into_iter())
                    .collect();

                let properties = properties.into_iter().map(|(s, _)| s).collect();

                Ok(Some(DbInsertOperation::Properties(properties, reporters)))
            }
            #[cfg(feature = "track-and-trace")]
            TRACK_AND_TRACE_PROPERTY_NAMESPACE => {
                let property_pages = PropertyPageList::from_bytes(&value)
                    .map_err(|err| {
                        EventError(format!("Failed to parse property page list {}", err))
                    })?
                    .property_pages()
                    .to_vec();

                let mut reported_values: Vec<StoreReportedValue> = vec![];
                for page in property_pages {
                    page.reported_values().to_vec().iter().try_fold(
                        &mut reported_values,
                        |acc, value| match make_reported_values(
                            commit_num,
                            page.record_id(),
                            value.value().name(),
                            value,
                        ) {
                            Ok(mut vals) => {
                                acc.append(&mut vals);
                                Ok(acc)
                            }
                            Err(err) => Err(err),
                        },
                    )?;
                }

                Ok(Some(DbInsertOperation::ReportedValues(reported_values)))
            }
            #[cfg(feature = "track-and-trace")]
            TRACK_AND_TRACE_PROPOSAL_NAMESPACE => {
                let proposals = ProposalList::from_bytes(&value)
                    .map_err(|err| EventError(format!("Failed to parse proposal list {}", err)))?
                    .proposals()
                    .iter()
                    .map(|proposal| Proposal {
                        id: None,
                        record_id: proposal.record_id().to_string(),
                        timestamp: *proposal.timestamp() as i64,
                        issuing_agent: proposal.issuing_agent().to_string(),
                        receiving_agent: proposal.receiving_agent().to_string(),
                        role: format!("{:?}", proposal.role()),
                        properties: proposal.properties().to_vec(),
                        status: format!("{:?}", proposal.status()),
                        terms: proposal.terms().to_string(),
                        start_commit_num: commit_num,
                        end_commit_num: MAX_COMMIT_NUM,
                        service_id: service_id.cloned(),
                    })
                    .collect::<Vec<Proposal>>();

                Ok(Some(DbInsertOperation::Proposals(proposals)))
            }
            #[cfg(feature = "track-and-trace")]
            TRACK_AND_TRACE_RECORD_NAMESPACE => {
                let record_list = RecordList::from_bytes(&value)
                    .map_err(|err| EventError(format!("Failed to parse record list {}", err)))?
                    .records()
                    .to_vec();

                let records = record_list
                    .iter()
                    .map(|record| Record {
                        id: None,
                        record_id: record.record_id().to_string(),
                        final_: *record.field_final(),
                        schema: record.schema().to_string(),
                        owners: record
                            .owners()
                            .iter()
                            .map(|x| x.agent_id().to_string())
                            .collect(),
                        custodians: record
                            .custodians()
                            .iter()
                            .map(|x| x.agent_id().to_string())
                            .collect(),
                        start_commit_num: commit_num,
                        end_commit_num: MAX_COMMIT_NUM,
                        service_id: service_id.cloned(),
                    })
                    .collect::<Vec<Record>>();

                let mut associated_agents = record_list
                    .iter()
                    .flat_map(|record| {
                        record.owners().iter().map(move |agent| AssociatedAgent {
                            id: None,
                            agent_id: agent.agent_id().to_string(),
                            record_id: record.record_id().to_string(),
                            role: "OWNER".to_string(),
                            timestamp: *agent.timestamp() as i64,
                            start_commit_num: commit_num,
                            end_commit_num: MAX_COMMIT_NUM,
                            service_id: service_id.cloned(),
                        })
                    })
                    .collect::<Vec<AssociatedAgent>>();

                associated_agents.append(
                    &mut record_list
                        .iter()
                        .flat_map(|record| {
                            record
                                .custodians()
                                .iter()
                                .map(move |agent| AssociatedAgent {
                                    id: None,
                                    agent_id: agent.agent_id().to_string(),
                                    role: "CUSTODIAN".to_string(),
                                    record_id: record.record_id().to_string(),
                                    timestamp: *agent.timestamp() as i64,
                                    start_commit_num: commit_num,
                                    end_commit_num: MAX_COMMIT_NUM,
                                    service_id: service_id.cloned(),
                                })
                        })
                        .collect::<Vec<AssociatedAgent>>(),
                );

                Ok(Some(DbInsertOperation::Records(records, associated_agents)))
            }
            #[cfg(feature = "location")]
            GRID_LOCATION_NAMESPACE => {
                let locations = LocationList::from_bytes(&value)
                    .map_err(|err| EventError(format!("Failed to parse location list {}", err)))?
                    .locations()
                    .iter()
                    .map(|location| Location {
                        location_id: location.location_id().to_string(),
                        location_address: key.to_string(),
                        location_namespace: format!("{:?}", location.namespace()),
                        owner: location.owner().to_string(),
                        attributes: make_location_attributes(
                            commit_num,
                            service_id,
                            location.location_id(),
                            &key,
                            location.properties(),
                        ),
                        start_commit_num: commit_num,
                        end_commit_num: MAX_COMMIT_NUM,
                        service_id: service_id.cloned(),
                        last_updated: None,
                    })
                    .collect();

                Ok(Some(DbInsertOperation::Locations(locations)))
            }
            #[cfg(feature = "product")]
            GRID_PRODUCT_NAMESPACE => {
                let products = ProductList::from_bytes(&value)
                    .map_err(|err| EventError(format!("Failed to parse product list {}", err)))?
                    .products()
                    .iter()
                    .map(|product| {
                        ProductBuilder::default()
                            .with_product_id(product.product_id().to_string())
                            .with_product_address(key.to_string())
                            .with_product_namespace(format!("{:?}", product.product_namespace()))
                            .with_owner(product.owner().to_string())
                            .with_start_commit_number(commit_num)
                            .with_end_commit_number(MAX_COMMIT_NUM)
                            .with_service_id(service_id.cloned())
                            .with_last_updated(None)
                            .with_properties(
                                make_product_property_values(
                                    commit_num,
                                    service_id,
                                    product.product_id(),
                                    &key,
                                    product.properties(),
                                )
                                .map_err(|err| EventError(format!("{}", err)))?,
                            )
                            .build()
                            .map_err(|err| EventError(format!("{}", err)))
                    })
                    .collect::<Result<Vec<Product>, EventError>>()?;

                Ok(Some(DbInsertOperation::Products(products)))
            }
            _ => {
                let ignore_state_change = IGNORED_NAMESPACES
                    .iter()
                    .any(|namespace| key.starts_with(namespace));
                if !ignore_state_change {
                    debug!("received state change for unknown address: {}", key);
                }
                Ok(None)
            }
        },
        StateChange::Delete { key } => match &key[0..8] {
            #[cfg(feature = "pike")]
            PIKE_NAMESPACE => match &key[0..10] {
                PIKE_ROLE_NAMESPACE => Ok(Some(DbInsertOperation::RemoveRole(
                    key.to_string(),
                    commit_num,
                ))),
                _ => Ok(None),
            },
            #[cfg(feature = "product")]
            GRID_PRODUCT_NAMESPACE => Ok(Some(DbInsertOperation::RemoveProduct(
                key.to_string(),
                commit_num,
            ))),
            #[cfg(feature = "location")]
            GRID_LOCATION_NAMESPACE => Ok(Some(DbInsertOperation::RemoveLocation(
                key.to_string(),
                commit_num,
            ))),
            _ => Err(EventError(format!(
                "could not handle state change; unexpected delete of key {}",
                key
            ))),
        },
    }
}

#[derive(Debug)]
enum DbInsertOperation {
    #[cfg(feature = "pike")]
    Agents(Vec<Agent>),
    #[cfg(feature = "pike")]
    Organizations(Vec<Organization>),
    #[cfg(feature = "pike")]
    Roles(Vec<Role>),
    #[cfg(feature = "pike")]
    RemoveRole(String, i64),
    #[cfg(feature = "schema")]
    GridSchemas(Vec<Schema>),
    #[cfg(feature = "location")]
    Locations(Vec<Location>),
    #[cfg(feature = "track-and-trace")]
    Properties(Vec<Property>, Vec<Reporter>),
    #[cfg(feature = "track-and-trace")]
    ReportedValues(Vec<StoreReportedValue>),
    #[cfg(feature = "track-and-trace")]
    Proposals(Vec<Proposal>),
    #[cfg(feature = "track-and-trace")]
    Records(Vec<Record>, Vec<AssociatedAgent>),
    #[cfg(feature = "product")]
    Products(Vec<Product>),
    #[cfg(feature = "location")]
    RemoveLocation(String, i64),
    #[cfg(feature = "product")]
    RemoveProduct(String, i64),
}

#[cfg(feature = "track-and-trace")]
fn make_reported_values(
    start_commit_num: i64,
    record_id: &str,
    property_name: &str,
    reported_value: &ReportedValue,
) -> Result<Vec<StoreReportedValue>, EventError> {
    let mut new_values = Vec::new();

    let mut new_value = StoreReportedValue {
        property_name: property_name.to_string(),
        record_id: record_id.to_string(),
        reporter_index: *reported_value.reporter_index() as i32,
        timestamp: *reported_value.timestamp() as i64,
        start_commit_num,
        end_commit_num: MAX_COMMIT_NUM,
        data_type: format!("{:?}", reported_value.value().data_type()),
        ..StoreReportedValue::default()
    };

    match reported_value.value().data_type() {
        DataType::Bytes => {
            new_value.bytes_value = Some(reported_value.value().bytes_value().to_vec())
        }
        DataType::Boolean => {
            new_value.boolean_value = Some(*reported_value.value().boolean_value())
        }
        DataType::Number => new_value.number_value = Some(*reported_value.value().number_value()),
        DataType::String => {
            new_value.string_value = Some(reported_value.value().string_value().to_string())
        }
        DataType::Enum => new_value.enum_value = Some(*reported_value.value().enum_value() as i32),
        DataType::Struct => {
            let mut child_values = Vec::new();

            for value in reported_value.value().struct_values() {
                let property_name = format!("{}_{}", reported_value.value().name(), value.name());
                let value = reported_value
                    .clone()
                    .into_builder()
                    .with_value(value.clone())
                    .build()
                    .map_err(|err| {
                        EventError(format!("Failed to build ReportedValue: {:?}", err))
                    })?;

                child_values.append(&mut make_reported_values(
                    start_commit_num,
                    record_id,
                    &property_name,
                    &value,
                )?);
            }

            new_value.struct_values = Some(child_values);
        }
        DataType::LatLong => {
            let lat_long_value = TntLatLongValue(
                *reported_value.value().lat_long_value().latitude(),
                *reported_value.value().lat_long_value().longitude(),
            );
            new_value.lat_long_value = Some(lat_long_value);
        }
    };

    new_values.push(new_value);

    Ok(new_values)
}

#[cfg(feature = "schema")]
fn make_property_definitions(
    start_commit_num: i64,
    service_id: Option<&String>,
    schema_name: &str,
    definitions: &[PropertyDefinition],
) -> Vec<StorePropertyDefinition> {
    let mut properties = Vec::new();

    for def in definitions {
        properties.push(StorePropertyDefinition {
            name: def.name().to_string(),
            schema_name: schema_name.to_string(),
            data_type: format!("{:?}", def.data_type()),
            required: *def.required(),
            description: def.description().to_string(),
            number_exponent: i64::from(*def.number_exponent()),
            enum_options: def.enum_options().to_vec(),
            struct_properties: make_property_definitions(
                start_commit_num,
                service_id,
                schema_name,
                def.struct_properties(),
            ),
            start_commit_num,
            end_commit_num: MAX_COMMIT_NUM,
            service_id: service_id.cloned(),
        });
    }

    properties
}

#[cfg(feature = "product")]
fn make_product_property_values(
    start_commit_num: i64,
    service_id: Option<&String>,
    product_id: &str,
    product_address: &str,
    values: &[PropertyValue],
) -> Result<Vec<ProductPropertyValue>, EventError> {
    let mut properties = Vec::new();

    for val in values {
        properties.push(
            ProductPropertyValueBuilder::default()
                .with_property_name(val.name().to_string())
                .with_product_id(product_id.to_string())
                .with_product_address(product_address.to_string())
                .with_data_type(format!("{:?}", val.data_type()))
                .with_bytes_value(Some(val.bytes_value().to_vec()))
                .with_boolean_value(Some(*val.boolean_value()))
                .with_number_value(Some(*val.number_value()))
                .with_string_value(Some(val.string_value().to_string()))
                .with_enum_value(Some(*val.enum_value() as i32))
                .with_struct_values(make_product_property_values(
                    start_commit_num,
                    service_id,
                    product_id,
                    product_address,
                    val.struct_values(),
                )?)
                .with_lat_long_value(Some(ProductLatLongValue {
                    latitude: *val.lat_long_value().latitude(),
                    longitude: *val.lat_long_value().longitude(),
                }))
                .with_start_commit_number(start_commit_num)
                .with_end_commit_number(MAX_COMMIT_NUM)
                .with_service_id(service_id.cloned())
                .build()
                .map_err(|err| EventError(format!("{}", err)))?,
        )
    }

    Ok(properties)
}

#[cfg(feature = "location")]
fn make_location_attributes(
    start_commit_num: i64,
    service_id: Option<&String>,
    location_id: &str,
    location_address: &str,
    attributes: &[PropertyValue],
) -> Vec<LocationAttribute> {
    let mut attrs = Vec::new();

    for attr in attributes {
        attrs.push(LocationAttribute {
            location_id: location_id.to_string(),
            location_address: location_address.to_string(),
            property_name: attr.name().to_string(),
            data_type: format!("{:?}", attr.data_type()),
            bytes_value: Some(attr.bytes_value().to_vec()),
            boolean_value: Some(*attr.boolean_value()),
            number_value: Some(*attr.number_value()),
            string_value: Some(attr.string_value().to_string()),
            enum_value: Some(*attr.enum_value() as i32),
            struct_values: Some(make_location_attributes(
                start_commit_num,
                service_id,
                location_id,
                location_address,
                attr.struct_values(),
            )),
            lat_long_value: Some(LocationLatLongValue(
                *attr.lat_long_value().latitude(),
                *attr.lat_long_value().longitude(),
            )),
            start_commit_num,
            end_commit_num: MAX_COMMIT_NUM,
            service_id: service_id.cloned(),
        });
    }

    attrs
}
