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
use grid_sdk::{
    grid_db::{
        agents::store::Agent,
        commits::store::CommitEvent as DbCommitEvent,
        organizations::store::Organization,
        products::store::{
            LatLongValue as ProductLatLongValue, Product, PropertyValue as ProductPropertyValue,
        },
        schemas::store::{PropertyDefinition as StorePropertyDefinition, Schema},
        track_and_trace::store::{
            AssociatedAgent, LatLongValue as TntLatLongValue, Property, Proposal, Record,
            ReportedValue as StoreReportedValue, Reporter,
        },
        AgentStore, CommitStore, DieselAgentStore, DieselCommitStore, DieselOrganizationStore,
        DieselProductStore, DieselSchemaStore, DieselTrackAndTraceStore, OrganizationStore,
        ProductStore, SchemaStore, TrackAndTraceStore,
    },
    protocol::{
        pike::state::{AgentList, OrganizationList},
        product::state::ProductList,
        schema::state::{DataType, PropertyDefinition, PropertyValue, SchemaList},
        track_and_trace::state::{
            PropertyList, PropertyPageList, ProposalList, RecordList, ReportedValue,
        },
    },
    protos::FromBytes,
};
use std::collections::HashMap;
use std::i64;

use crate::database::ConnectionPool;

use super::{
    CommitEvent, EventError, EventHandler, StateChange, GRID_PRODUCT, GRID_SCHEMA,
    IGNORED_NAMESPACES, PIKE_AGENT, PIKE_ORG, TRACK_AND_TRACE_PROPERTY, TRACK_AND_TRACE_PROPOSAL,
    TRACK_AND_TRACE_RECORD,
};

pub const MAX_COMMIT_NUM: i64 = i64::MAX;

pub struct DatabaseEventHandler<C: diesel::Connection + 'static> {
    connection_pool: ConnectionPool<C>,
    agent_store: DieselAgentStore<C>,
    commit_store: DieselCommitStore<C>,
    organization_store: DieselOrganizationStore<C>,
    product_store: DieselProductStore<C>,
    schema_store: DieselSchemaStore<C>,
    tnt_store: DieselTrackAndTraceStore<C>,
}

impl DatabaseEventHandler<diesel::pg::PgConnection> {
    pub fn from_pg_pool(connection_pool: ConnectionPool<diesel::pg::PgConnection>) -> Self {
        let agent_store = DieselAgentStore::new(connection_pool.pool.clone());
        let commit_store = DieselCommitStore::new(connection_pool.pool.clone());
        let organization_store = DieselOrganizationStore::new(connection_pool.pool.clone());
        let product_store = DieselProductStore::new(connection_pool.pool.clone());
        let schema_store = DieselSchemaStore::new(connection_pool.pool.clone());
        let tnt_store = DieselTrackAndTraceStore::new(connection_pool.pool.clone());

        Self {
            agent_store,
            connection_pool,
            commit_store,
            organization_store,
            product_store,
            schema_store,
            tnt_store,
        }
    }
}

impl EventHandler for DatabaseEventHandler<diesel::pg::PgConnection> {
    fn handle_event(&self, event: &CommitEvent) -> Result<(), EventError> {
        debug!("Received commit event: {}", event);

        let conn = self
            .connection_pool
            .get()
            .map_err(|err| EventError(format!("Unable to connect to database: {}", err)))?;

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

        conn.build_transaction().run::<_, EventError, _>(|| {
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
                    DbInsertOperation::Agents(agents) => {
                        debug!("Inserting {} agents", agents.len());
                        agents
                            .into_iter()
                            .try_for_each(|agent| self.agent_store.add_agent(agent))?;
                    }
                    DbInsertOperation::Organizations(orgs) => {
                        debug!("Inserting {} organizations", orgs.len());
                        self.organization_store.add_organizations(orgs)?;
                    }

                    DbInsertOperation::GridSchemas(schemas) => {
                        debug!("Inserting {} schemas", schemas.len());
                        schemas
                            .into_iter()
                            .try_for_each(|schema| self.schema_store.add_schema(schema))?;
                    }
                    DbInsertOperation::Properties(properties, reporters) => {
                        debug!("Inserting {} properties", properties.len());
                        self.tnt_store.add_properties(properties)?;
                        debug!("Inserting {} reporters", reporters.len());
                        self.tnt_store.add_reporters(reporters)?;
                    }
                    DbInsertOperation::ReportedValues(reported_values) => {
                        debug!("Inserting {} reported values", reported_values.len());
                        self.tnt_store.add_reported_values(reported_values)?;
                    }
                    DbInsertOperation::Proposals(proposals) => {
                        debug!("Inserting {} proposals", proposals.len());
                        self.tnt_store.add_proposals(proposals)?;
                    }
                    DbInsertOperation::Records(records, associated_agents) => {
                        debug!("Inserting {} records", records.len());
                        self.tnt_store.add_records(records)?;
                        debug!("Inserting {} associated agents", associated_agents.len());
                        self.tnt_store.add_associated_agents(associated_agents)?;
                    }
                    DbInsertOperation::Products(products) => {
                        debug!("Inserting {} products", products.len());
                        products
                            .into_iter()
                            .try_for_each(|product| self.product_store.add_product(product))?;
                    }
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

impl DatabaseEventHandler<diesel::sqlite::SqliteConnection> {
    pub fn from_sqlite_pool(
        connection_pool: ConnectionPool<diesel::sqlite::SqliteConnection>,
    ) -> Self {
        let agent_store = DieselAgentStore::new(connection_pool.pool.clone());
        let commit_store = DieselCommitStore::new(connection_pool.pool.clone());
        let organization_store = DieselOrganizationStore::new(connection_pool.pool.clone());
        let product_store = DieselProductStore::new(connection_pool.pool.clone());
        let schema_store = DieselSchemaStore::new(connection_pool.pool.clone());
        let tnt_store = DieselTrackAndTraceStore::new(connection_pool.pool.clone());

        Self {
            agent_store,
            connection_pool,
            commit_store,
            organization_store,
            product_store,
            schema_store,
            tnt_store,
        }
    }
}

impl EventHandler for DatabaseEventHandler<diesel::sqlite::SqliteConnection> {
    fn handle_event(&self, event: &CommitEvent) -> Result<(), EventError> {
        debug!("Received commit event: {}", event);

        let conn = self
            .connection_pool
            .get()
            .map_err(|err| EventError(format!("Unable to connect to database: {}", err)))?;

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

        conn.transaction::<_, EventError, _>(|| {
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
                    DbInsertOperation::Agents(agents) => {
                        debug!("Inserting {} agents", agents.len());
                        agents
                            .into_iter()
                            .try_for_each(|agent| self.agent_store.add_agent(agent))?;
                    }
                    DbInsertOperation::Organizations(orgs) => {
                        debug!("Inserting {} organizations", orgs.len());
                        self.organization_store.add_organizations(orgs)?;
                    }

                    DbInsertOperation::GridSchemas(schemas) => {
                        debug!("Inserting {} schemas", schemas.len());
                        schemas
                            .into_iter()
                            .try_for_each(|schema| self.schema_store.add_schema(schema))?;
                    }
                    DbInsertOperation::Properties(properties, reporters) => {
                        debug!("Inserting {} properties", properties.len());
                        self.tnt_store.add_properties(properties)?;
                        debug!("Inserting {} reporters", reporters.len());
                        self.tnt_store.add_reporters(reporters)?;
                    }
                    DbInsertOperation::ReportedValues(reported_values) => {
                        debug!("Inserting {} reported values", reported_values.len());
                        self.tnt_store.add_reported_values(reported_values)?;
                    }
                    DbInsertOperation::Proposals(proposals) => {
                        debug!("Inserting {} proposals", proposals.len());
                        self.tnt_store.add_proposals(proposals)?;
                    }
                    DbInsertOperation::Records(records, associated_agents) => {
                        debug!("Inserting {} records", records.len());
                        self.tnt_store.add_records(records)?;
                        debug!("Inserting {} associated agents", associated_agents.len());
                        self.tnt_store.add_associated_agents(associated_agents)?;
                    }
                    DbInsertOperation::Products(products) => {
                        debug!("Inserting {} products", products.len());
                        products
                            .into_iter()
                            .try_for_each(|product| self.product_store.add_product(product))?;
                    }
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

fn state_change_to_db_operation(
    state_change: &StateChange,
    commit_num: i64,
    service_id: Option<&String>,
) -> Result<Option<DbInsertOperation>, EventError> {
    match state_change {
        StateChange::Set { key, value } => match &key[0..8] {
            PIKE_AGENT => {
                let agents = AgentList::from_bytes(&value)
                    .map_err(|err| EventError(format!("Failed to parse agent list {}", err)))?
                    .agents()
                    .iter()
                    .map(|agent| Agent {
                        public_key: agent.public_key().to_string(),
                        org_id: agent.org_id().to_string(),
                        active: *agent.active(),
                        roles: agent.roles().to_vec(),
                        metadata: json!(agent.metadata().iter().fold(
                            HashMap::new(),
                            |mut acc, md| {
                                acc.insert(md.key().to_string(), md.value().to_string());
                                acc
                            }
                        ))
                        .to_string()
                        .into_bytes(),
                        start_commit_num: commit_num,
                        end_commit_num: MAX_COMMIT_NUM,
                        service_id: service_id.cloned(),
                    })
                    .collect::<Vec<Agent>>();

                Ok(Some(DbInsertOperation::Agents(agents)))
            }
            PIKE_ORG => {
                let orgs = OrganizationList::from_bytes(&value)
                    .map_err(|err| {
                        EventError(format!("Failed to parse organization list {}", err))
                    })?
                    .organizations()
                    .iter()
                    .map(|org| Organization {
                        org_id: org.org_id().to_string(),
                        name: org.name().to_string(),
                        address: org.address().to_string(),
                        metadata: json!(org.metadata().iter().fold(
                            HashMap::new(),
                            |mut acc, md| {
                                acc.insert(md.key().to_string(), md.value().to_string());
                                acc
                            }
                        ))
                        .to_string()
                        .into_bytes(),
                        start_commit_num: commit_num,
                        end_commit_num: MAX_COMMIT_NUM,
                        service_id: service_id.cloned(),
                    })
                    .collect::<Vec<Organization>>();

                Ok(Some(DbInsertOperation::Organizations(orgs)))
            }
            GRID_SCHEMA => {
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
            TRACK_AND_TRACE_PROPERTY if &key[66..] == "0000" => {
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
            TRACK_AND_TRACE_PROPERTY => {
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
            TRACK_AND_TRACE_PROPOSAL => {
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
            TRACK_AND_TRACE_RECORD => {
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
            GRID_PRODUCT => {
                let products = ProductList::from_bytes(&value)
                    .map_err(|err| EventError(format!("Failed to parse product list {}", err)))?
                    .products()
                    .iter()
                    .map(|product| Product {
                        product_id: product.product_id().to_string(),
                        product_address: key.to_string(),
                        product_namespace: format!("{:?}", product.product_namespace()),
                        owner: product.owner().to_string(),
                        start_commit_num: commit_num,
                        end_commit_num: MAX_COMMIT_NUM,
                        service_id: service_id.cloned(),
                        properties: make_product_property_values(
                            commit_num,
                            service_id,
                            product.product_id(),
                            &key,
                            product.properties(),
                        ),
                    })
                    .collect();

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
        StateChange::Delete { key } => {
            if &key[0..8] == GRID_PRODUCT {
                Ok(Some(DbInsertOperation::RemoveProduct(
                    key.to_string(),
                    commit_num,
                )))
            } else {
                Err(EventError(format!(
                    "could not handle state change; unexpected delete of key {}",
                    key
                )))
            }
        }
    }
}

#[derive(Debug)]
enum DbInsertOperation {
    Agents(Vec<Agent>),
    Organizations(Vec<Organization>),
    GridSchemas(Vec<Schema>),
    Properties(Vec<Property>, Vec<Reporter>),
    ReportedValues(Vec<StoreReportedValue>),
    Proposals(Vec<Proposal>),
    Records(Vec<Record>, Vec<AssociatedAgent>),
    Products(Vec<Product>),
    RemoveProduct(String, i64),
}

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

fn make_product_property_values(
    start_commit_num: i64,
    service_id: Option<&String>,
    product_id: &str,
    product_address: &str,
    values: &[PropertyValue],
) -> Vec<ProductPropertyValue> {
    let mut properties = Vec::new();

    for val in values {
        properties.push(ProductPropertyValue {
            property_name: val.name().to_string(),
            product_id: product_id.to_string(),
            product_address: product_address.to_string(),
            data_type: format!("{:?}", val.data_type()),
            bytes_value: Some(val.bytes_value().to_vec()),
            boolean_value: Some(*val.boolean_value()),
            number_value: Some(*val.number_value()),
            string_value: Some(val.string_value().to_string()),
            enum_value: Some(*val.enum_value() as i32),
            struct_values: make_product_property_values(
                start_commit_num,
                service_id,
                product_id,
                product_address,
                val.struct_values(),
            ),
            lat_long_value: Some(ProductLatLongValue {
                latitude: *val.lat_long_value().latitude(),
                longitude: *val.lat_long_value().longitude(),
            }),
            start_commit_num,
            end_commit_num: MAX_COMMIT_NUM,
            service_id: service_id.cloned(),
        });
    }

    properties
}
