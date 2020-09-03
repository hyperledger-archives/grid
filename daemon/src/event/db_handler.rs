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

use ::diesel::pg::PgConnection;
use ::diesel::result::QueryResult;
use grid_sdk::{
    grid_db::commits::store::diesel::DieselCommitStore,
    grid_db::commits::store::{CommitEvent as DbCommitEvent, CommitStore},
    grid_db::error::StoreError,
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
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::i64;

use crate::database::{
    helpers as db,
    models::{
        LatLongValue, NewAgent, NewAssociatedAgent, NewGridPropertyDefinition, NewGridSchema,
        NewOrganization, NewProduct, NewProductPropertyValue, NewProperty, NewProposal, NewRecord,
        NewReportedValue, NewReporter,
    },
    ConnectionPool,
};

use super::{
    CommitEvent, EventError, EventHandler, StateChange, GRID_PRODUCT, GRID_SCHEMA,
    IGNORED_NAMESPACES, PIKE_AGENT, PIKE_ORG, TRACK_AND_TRACE_PROPERTY, TRACK_AND_TRACE_PROPOSAL,
    TRACK_AND_TRACE_RECORD,
};

pub struct DatabaseEventHandler<C: diesel::Connection + 'static> {
    connection_pool: ConnectionPool<C>,
    commit_store: Box<dyn CommitStore>,
}

#[cfg(feature = "postgres")]
impl DatabaseEventHandler<diesel::pg::PgConnection> {
    pub fn new(connection_pool: ConnectionPool<diesel::pg::PgConnection>) -> Self {
        let commit_store = Box::new(DieselCommitStore::new(connection_pool.pool.clone()));
        Self {
            connection_pool,
            commit_store,
        }
    }
}

#[cfg(feature = "postgres")]
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

        conn.build_transaction()
            .run::<_, StoreError, _>(|| {
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
                        return Err(err);
                    }
                }

                db_ops
                    .iter()
                    .try_for_each(|op| op.execute(&conn))
                    .map_err(|err| StoreError::OperationError {
                        context: "failed DB operation".to_string(),
                        source: Some(Box::new(err)),
                    })
            })
            .map_err(|err| EventError(format!("Database transaction failed {}", err)))
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
                    .map(|agent| NewAgent {
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
                        )),
                        start_commit_num: commit_num,
                        end_commit_num: db::MAX_COMMIT_NUM,
                        service_id: service_id.cloned(),
                    })
                    .collect::<Vec<NewAgent>>();

                Ok(Some(DbInsertOperation::Agents(agents)))
            }
            PIKE_ORG => {
                let orgs = OrganizationList::from_bytes(&value)
                    .map_err(|err| {
                        EventError(format!("Failed to parse organization list {}", err))
                    })?
                    .organizations()
                    .iter()
                    .map(|org| NewOrganization {
                        org_id: org.org_id().to_string(),
                        name: org.name().to_string(),
                        address: org.address().to_string(),
                        metadata: org
                            .metadata()
                            .iter()
                            .map(|md| {
                                json!({
                                    md.key(): md.value()
                                })
                            })
                            .collect::<Vec<JsonValue>>(),
                        start_commit_num: commit_num,
                        end_commit_num: db::MAX_COMMIT_NUM,
                        service_id: service_id.cloned(),
                    })
                    .collect::<Vec<NewOrganization>>();

                Ok(Some(DbInsertOperation::Organizations(orgs)))
            }
            GRID_SCHEMA => {
                let schema_defs = SchemaList::from_bytes(&value)
                    .map_err(|err| EventError(format!("Failed to parse schema list {}", err)))?
                    .schemas()
                    .iter()
                    .map(|state_schema| {
                        let schema = NewGridSchema {
                            name: state_schema.name().to_string(),
                            description: state_schema.description().to_string(),
                            owner: state_schema.owner().to_string(),
                            start_commit_num: commit_num,
                            end_commit_num: db::MAX_COMMIT_NUM,
                            service_id: service_id.cloned(),
                        };

                        let definitions = make_property_definitions(
                            commit_num,
                            service_id,
                            state_schema.name(),
                            state_schema.properties(),
                        );

                        (schema, definitions)
                    })
                    .collect::<Vec<(NewGridSchema, Vec<NewGridPropertyDefinition>)>>();

                let definitions = schema_defs
                    .clone()
                    .into_iter()
                    .flat_map(|(_, d)| d.into_iter())
                    .collect();

                let schemas = schema_defs.into_iter().map(|(s, _)| s).collect();

                Ok(Some(DbInsertOperation::GridSchemas(schemas, definitions)))
            }
            TRACK_AND_TRACE_PROPERTY if &key[66..] == "0000" => {
                let properties = PropertyList::from_bytes(&value)
                    .map_err(|err| EventError(format!("Failed to parse property list {}", err)))?
                    .properties()
                    .iter()
                    .map(|prop| {
                        let property = NewProperty {
                            name: prop.name().to_string(),
                            record_id: prop.record_id().to_string(),
                            property_definition: prop.property_definition().name().to_string(),
                            current_page: *prop.current_page() as i32,
                            wrapped: *prop.wrapped(),
                            start_commit_num: commit_num,
                            end_commit_num: db::MAX_COMMIT_NUM,
                            service_id: service_id.cloned(),
                        };

                        let reporters = prop
                            .reporters()
                            .iter()
                            .map(|reporter| NewReporter {
                                property_name: prop.name().to_string(),
                                record_id: prop.record_id().to_string(),
                                public_key: reporter.public_key().to_string(),
                                authorized: *reporter.authorized(),
                                reporter_index: *reporter.index() as i32,
                                start_commit_num: commit_num,
                                end_commit_num: db::MAX_COMMIT_NUM,
                                service_id: service_id.cloned(),
                            })
                            .collect::<Vec<NewReporter>>();

                        (property, reporters)
                    })
                    .collect::<Vec<(NewProperty, Vec<NewReporter>)>>();

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

                let mut reported_values: Vec<NewReportedValue> = vec![];
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
                    .map(|proposal| NewProposal {
                        record_id: proposal.record_id().to_string(),
                        timestamp: *proposal.timestamp() as i64,
                        issuing_agent: proposal.issuing_agent().to_string(),
                        receiving_agent: proposal.receiving_agent().to_string(),
                        role: format!("{:?}", proposal.role()),
                        properties: proposal.properties().to_vec(),
                        status: format!("{:?}", proposal.status()),
                        terms: proposal.terms().to_string(),
                        start_commit_num: commit_num,
                        end_commit_num: db::MAX_COMMIT_NUM,
                        service_id: service_id.cloned(),
                    })
                    .collect::<Vec<NewProposal>>();

                Ok(Some(DbInsertOperation::Proposals(proposals)))
            }
            TRACK_AND_TRACE_RECORD => {
                let record_list = RecordList::from_bytes(&value)
                    .map_err(|err| EventError(format!("Failed to parse record list {}", err)))?
                    .records()
                    .to_vec();

                let records = record_list
                    .iter()
                    .map(|record| NewRecord {
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
                        end_commit_num: db::MAX_COMMIT_NUM,
                        service_id: service_id.cloned(),
                    })
                    .collect::<Vec<NewRecord>>();

                let mut associated_agents = record_list
                    .iter()
                    .flat_map(|record| {
                        record.owners().iter().map(move |agent| NewAssociatedAgent {
                            agent_id: agent.agent_id().to_string(),
                            record_id: record.record_id().to_string(),
                            role: "OWNER".to_string(),
                            timestamp: *agent.timestamp() as i64,
                            start_commit_num: commit_num,
                            end_commit_num: db::MAX_COMMIT_NUM,
                            service_id: service_id.cloned(),
                        })
                    })
                    .collect::<Vec<NewAssociatedAgent>>();

                associated_agents.append(
                    &mut record_list
                        .iter()
                        .flat_map(|record| {
                            record
                                .custodians()
                                .iter()
                                .map(move |agent| NewAssociatedAgent {
                                    agent_id: agent.agent_id().to_string(),
                                    role: "CUSTODIAN".to_string(),
                                    record_id: record.record_id().to_string(),
                                    timestamp: *agent.timestamp() as i64,
                                    start_commit_num: commit_num,
                                    end_commit_num: db::MAX_COMMIT_NUM,
                                    service_id: service_id.cloned(),
                                })
                        })
                        .collect::<Vec<NewAssociatedAgent>>(),
                );

                Ok(Some(DbInsertOperation::Records(records, associated_agents)))
            }
            GRID_PRODUCT => {
                let product_tuple = ProductList::from_bytes(&value)
                    .map_err(|err| EventError(format!("Failed to parse product list {}", err)))?
                    .products()
                    .iter()
                    .fold((Vec::new(), Vec::new()), |mut acc, product| {
                        let new_product = NewProduct {
                            product_id: product.product_id().to_string(),
                            product_address: key.to_string(),
                            product_namespace: format!("{:?}", product.product_type()),
                            owner: product.owner().to_string(),
                            start_commit_num: commit_num,
                            end_commit_num: db::MAX_COMMIT_NUM,
                            service_id: service_id.cloned(),
                        };
                        acc.0.push(new_product);

                        let mut properties = make_product_property_values(
                            commit_num,
                            service_id,
                            product.product_id(),
                            &key,
                            product.properties(),
                        );
                        acc.1.append(&mut properties);

                        acc
                    });

                Ok(Some(DbInsertOperation::Products(
                    product_tuple.0,
                    product_tuple.1,
                )))
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
    Agents(Vec<NewAgent>),
    Organizations(Vec<NewOrganization>),
    GridSchemas(Vec<NewGridSchema>, Vec<NewGridPropertyDefinition>),
    Properties(Vec<NewProperty>, Vec<NewReporter>),
    ReportedValues(Vec<NewReportedValue>),
    Proposals(Vec<NewProposal>),
    Records(Vec<NewRecord>, Vec<NewAssociatedAgent>),
    Products(Vec<NewProduct>, Vec<NewProductPropertyValue>),
    RemoveProduct(String, i64),
}

impl DbInsertOperation {
    fn execute(&self, conn: &PgConnection) -> QueryResult<()> {
        match *self {
            DbInsertOperation::Agents(ref agents) => {
                debug!("Inserting {} agents", agents.len());
                db::insert_agents(conn, agents)
            }
            DbInsertOperation::Organizations(ref orgs) => {
                debug!("Inserting {} organizations", orgs.len());
                db::insert_organizations(conn, orgs)
            }
            DbInsertOperation::GridSchemas(ref schemas, ref defs) => {
                debug!("Inserting {} schemas", schemas.len());
                db::insert_grid_schemas(conn, schemas)?;
                db::insert_grid_property_definitions(conn, defs)
            }
            DbInsertOperation::Properties(ref properties, ref reporters) => {
                debug!("Inserting {} properties", properties.len());
                db::insert_properties(conn, properties)?;
                db::insert_reporters(conn, reporters)
            }
            DbInsertOperation::ReportedValues(ref reported_values) => {
                debug!("Inserting {} reported values", reported_values.len());
                db::insert_reported_values(conn, reported_values)
            }
            DbInsertOperation::Proposals(ref proposals) => {
                debug!("Inserting {} proposals", proposals.len());
                db::insert_proposals(conn, proposals)
            }
            DbInsertOperation::Records(ref records, ref associated_agents) => {
                debug!("Inserting {} records", records.len());
                db::insert_records(conn, records)?;
                db::insert_associated_agents(conn, associated_agents)
            }
            DbInsertOperation::Products(ref products, ref properties) => {
                debug!("Inserting {} products", products.len());
                db::insert_products(conn, products)?;
                db::insert_product_property_values(conn, properties)
            }
            DbInsertOperation::RemoveProduct(ref address, current_commit_num) => {
                db::delete_product(conn, address, current_commit_num)?;
                db::delete_product_property_values(conn, address, current_commit_num)
            }
        }
    }
}

fn make_reported_values(
    start_commit_num: i64,
    record_id: &str,
    property_name: &str,
    reported_value: &ReportedValue,
) -> Result<Vec<NewReportedValue>, EventError> {
    let mut new_values = Vec::new();

    let mut new_value = NewReportedValue {
        property_name: property_name.to_string(),
        record_id: record_id.to_string(),
        reporter_index: *reported_value.reporter_index() as i32,
        timestamp: *reported_value.timestamp() as i64,
        start_commit_num,
        end_commit_num: db::MAX_COMMIT_NUM,
        data_type: format!("{:?}", reported_value.value().data_type()),
        ..NewReportedValue::default()
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
            new_value.struct_values = Some(
                reported_value
                    .value()
                    .struct_values()
                    .iter()
                    .map(|x| x.name().to_string())
                    .collect(),
            )
        }
        DataType::LatLong => {
            let lat_long_value = LatLongValue(
                *reported_value.value().lat_long_value().latitude(),
                *reported_value.value().lat_long_value().longitude(),
            );
            new_value.lat_long_value = Some(lat_long_value);
        }
    };

    new_values.push(new_value);

    reported_value
        .value()
        .struct_values()
        .iter()
        .try_fold(&mut new_values, |acc, val| {
            let property_name = format!("{}_{}", reported_value.value().name(), val.name());
            match reported_value
                .clone()
                .into_builder()
                .with_value(val.clone())
                .build()
                .map_err(|err| EventError(format!("Failed to build ReportedValue: {:?}", err)))
            {
                Ok(temp_val) => match make_reported_values(
                    start_commit_num,
                    record_id,
                    &property_name,
                    &temp_val,
                ) {
                    Ok(mut vals) => {
                        acc.append(&mut vals);
                        Ok(acc)
                    }
                    Err(err) => Err(err),
                },
                Err(err) => Err(err),
            }
        })?;

    Ok(new_values)
}

fn make_property_definitions(
    start_commit_num: i64,
    service_id: Option<&String>,
    schema_name: &str,
    definitions: &[PropertyDefinition],
) -> Vec<NewGridPropertyDefinition> {
    let mut properties = Vec::new();

    for def in definitions {
        properties.push(NewGridPropertyDefinition {
            name: def.name().to_string(),
            schema_name: schema_name.to_string(),
            data_type: format!("{:?}", def.data_type()),
            required: *def.required(),
            description: def.description().to_string(),
            number_exponent: i64::from(*def.number_exponent()),
            enum_options: def.enum_options().to_vec(),
            struct_properties: def
                .struct_properties()
                .iter()
                .map(|x| x.name().to_string())
                .collect(),
            start_commit_num,
            end_commit_num: db::MAX_COMMIT_NUM,
            service_id: service_id.cloned(),
        });

        if !def.struct_properties().is_empty() {
            properties.append(&mut make_property_definitions(
                start_commit_num,
                service_id,
                schema_name,
                def.struct_properties(),
            ));
        }
    }

    properties
}

fn make_product_property_values(
    start_commit_num: i64,
    service_id: Option<&String>,
    product_id: &str,
    product_address: &str,
    values: &[PropertyValue],
) -> Vec<NewProductPropertyValue> {
    let mut properties = Vec::new();

    for val in values {
        properties.push(NewProductPropertyValue {
            property_name: val.name().to_string(),
            product_id: product_id.to_string(),
            product_address: product_address.to_string(),
            data_type: format!("{:?}", val.data_type()),
            bytes_value: Some(val.bytes_value().to_vec()),
            boolean_value: Some(*val.boolean_value()),
            number_value: Some(*val.number_value()),
            string_value: Some(val.string_value().to_string()),
            enum_value: Some(*val.enum_value() as i32),
            struct_values: Some(
                val.struct_values()
                    .iter()
                    .map(|x| x.name().to_string())
                    .collect(),
            ),
            lat_long_value: Some(LatLongValue(
                *val.lat_long_value().latitude(),
                *val.lat_long_value().longitude(),
            )),
            start_commit_num,
            end_commit_num: db::MAX_COMMIT_NUM,
            service_id: service_id.cloned(),
        });

        if !val.struct_values().is_empty() {
            properties.append(&mut make_product_property_values(
                start_commit_num,
                service_id,
                product_id,
                product_address,
                val.struct_values(),
            ));
        }
    }

    properties
}
