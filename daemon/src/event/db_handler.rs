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

use std::convert::TryInto;

use diesel::prelude::*;
use diesel::result::Error;
use grid_sdk::{
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
        LatLongValue, NewAgent, NewAssociatedAgent, NewBlock, NewGridPropertyDefinition,
        NewGridSchema, NewOrganization, NewProduct, NewProductPropertyValue, NewProperty,
        NewProposal, NewRecord, NewReportedValue, NewReporter,
    },
    ConnectionPool,
};

use super::{
    CommitEvent, EventError, EventHandler, StateChange, GRID_PRODUCT, GRID_SCHEMA, PIKE_AGENT,
    PIKE_ORG, TRACK_AND_TRACE_PROPERTY, TRACK_AND_TRACE_PROPOSAL, TRACK_AND_TRACE_RECORD,
};

pub struct DatabaseEventHandler {
    connection_pool: ConnectionPool,
}

impl DatabaseEventHandler {
    pub fn new(connection_pool: ConnectionPool) -> Self {
        Self { connection_pool }
    }
}

impl EventHandler for DatabaseEventHandler {
    fn handle_event(&self, event: &CommitEvent) -> Result<(), EventError> {
        debug!("Received commit event: ({}, {:?})", event.id, event.height);

        let block = create_db_block_from_commit_event(event)?;
        let db_ops = create_db_operations_from_block(event)?;

        trace!("The following operations will be performed: {:#?}", db_ops);

        let conn = self
            .connection_pool
            .get()
            .map_err(|err| EventError(format!("Unable to connect to database: {}", err)))?;

        conn.build_transaction()
            .run::<_, Error, _>(|| {
                match db::get_block_by_block_num(&conn, block.block_num) {
                    Ok(Some(ref b)) if b.block_id != block.block_id => {
                        db::resolve_fork(&conn, block.block_num)?;
                        info!(
                            "Fork detected. Replaced {} at height {}, with block {}.",
                            &b.block_id, &b.block_num, &block.block_id
                        );
                        db::insert_block(&conn, &block)?;
                    }
                    Ok(Some(_)) => {
                        info!(
                            "Block {} at height {} is duplicate no action taken",
                            &block.block_id, block.block_num
                        );
                    }
                    Ok(None) => {
                        info!("Received new block {}", block.block_id);
                        db::insert_block(&conn, &block)?;
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }

                db_ops.iter().try_for_each(|op| op.execute(&conn))
            })
            .map_err(|err| EventError(format!("Database transaction failed {}", err)))
    }
}

fn create_db_block_from_commit_event(event: &CommitEvent) -> Result<NewBlock, EventError> {
    let block_id = event.id.clone();
    let block_num = commit_event_height_to_block_num(event.height)?;
    let source = Some(event.source.clone());
    Ok(NewBlock {
        block_id,
        block_num,
        source,
    })
}

fn create_db_operations_from_block(
    event: &CommitEvent,
) -> Result<Vec<DbInsertOperation>, EventError> {
    let block_num = commit_event_height_to_block_num(event.height)?;
    event
        .state_changes
        .iter()
        .map(|state_change| state_change_to_db_operation(state_change, block_num))
        .collect::<Result<Vec<DbInsertOperation>, EventError>>()
}

fn commit_event_height_to_block_num(height: Option<u64>) -> Result<i64, EventError> {
    height
        .ok_or_else(|| EventError("event height cannot be none".into()))?
        .try_into()
        .map_err(|err| EventError(format!("failed to convert event height to i64: {}", err)))
}

fn state_change_to_db_operation(
    state_change: &StateChange,
    block_num: i64,
) -> Result<DbInsertOperation, EventError> {
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
                        start_block_num: block_num,
                        end_block_num: db::MAX_BLOCK_NUM,
                        source: None,
                    })
                    .collect::<Vec<NewAgent>>();

                Ok(DbInsertOperation::Agents(agents))
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
                        start_block_num: block_num,
                        end_block_num: db::MAX_BLOCK_NUM,
                        source: None,
                    })
                    .collect::<Vec<NewOrganization>>();

                Ok(DbInsertOperation::Organizations(orgs))
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
                            start_block_num: block_num,
                            end_block_num: db::MAX_BLOCK_NUM,
                            source: None,
                        };

                        let definitions = make_property_definitions(
                            block_num,
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

                Ok(DbInsertOperation::GridSchemas(schemas, definitions))
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
                            start_block_num: block_num,
                            end_block_num: db::MAX_BLOCK_NUM,
                            source: None,
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
                                start_block_num: block_num,
                                end_block_num: db::MAX_BLOCK_NUM,
                                source: None,
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

                Ok(DbInsertOperation::Properties(properties, reporters))
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
                            block_num,
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

                Ok(DbInsertOperation::ReportedValues(reported_values))
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
                        start_block_num: block_num,
                        end_block_num: db::MAX_BLOCK_NUM,
                        source: None,
                    })
                    .collect::<Vec<NewProposal>>();

                Ok(DbInsertOperation::Proposals(proposals))
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
                        start_block_num: block_num,
                        end_block_num: db::MAX_BLOCK_NUM,
                        source: None,
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
                            start_block_num: block_num,
                            end_block_num: db::MAX_BLOCK_NUM,
                            source: None,
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
                                    start_block_num: block_num,
                                    end_block_num: db::MAX_BLOCK_NUM,
                                    source: None,
                                })
                        })
                        .collect::<Vec<NewAssociatedAgent>>(),
                );

                Ok(DbInsertOperation::Records(records, associated_agents))
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
                            start_block_num: block_num,
                            end_block_num: db::MAX_BLOCK_NUM,
                            source: None,
                        };
                        acc.0.push(new_product);

                        let mut properties = make_product_property_values(
                            block_num,
                            product.product_id(),
                            &key,
                            product.properties(),
                        );
                        acc.1.append(&mut properties);

                        acc
                    });

                Ok(DbInsertOperation::Products(
                    product_tuple.0,
                    product_tuple.1,
                ))
            }
            _ => Err(EventError(format!(
                "could not handle state change; unknown key: {}",
                key
            ))),
        },
        StateChange::Delete { key } => {
            if &key[0..8] == GRID_PRODUCT {
                Ok(DbInsertOperation::RemoveProduct(key.to_string(), block_num))
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
            DbInsertOperation::Agents(ref agents) => db::insert_agents(conn, agents),
            DbInsertOperation::Organizations(ref orgs) => db::insert_organizations(conn, orgs),
            DbInsertOperation::GridSchemas(ref schemas, ref defs) => {
                db::insert_grid_schemas(conn, schemas)?;
                db::insert_grid_property_definitions(conn, defs)
            }
            DbInsertOperation::Properties(ref properties, ref reporters) => {
                db::insert_properties(conn, properties)?;
                db::insert_reporters(conn, reporters)
            }
            DbInsertOperation::ReportedValues(ref reported_values) => {
                db::insert_reported_values(conn, reported_values)
            }
            DbInsertOperation::Proposals(ref proposals) => db::insert_proposals(conn, proposals),
            DbInsertOperation::Records(ref records, ref associated_agents) => {
                db::insert_records(conn, records)?;
                db::insert_associated_agents(conn, associated_agents)
            }
            DbInsertOperation::Products(ref products, ref properties) => {
                db::insert_products(conn, products)?;
                db::insert_product_property_values(conn, properties)
            }
            DbInsertOperation::RemoveProduct(ref address, current_block_num) => {
                db::delete_product(conn, address, current_block_num)?;
                db::delete_product_property_values(conn, address, current_block_num)
            }
        }
    }
}

fn make_reported_values(
    start_block_num: i64,
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
        start_block_num,
        end_block_num: db::MAX_BLOCK_NUM,
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
                    start_block_num,
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
    start_block_num: i64,
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
            start_block_num,
            end_block_num: db::MAX_BLOCK_NUM,
            source: None,
        });

        if !def.struct_properties().is_empty() {
            properties.append(&mut make_property_definitions(
                start_block_num,
                schema_name,
                def.struct_properties(),
            ));
        }
    }

    properties
}

fn make_product_property_values(
    start_block_num: i64,
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
            start_block_num,
            end_block_num: db::MAX_BLOCK_NUM,
            source: None,
        });

        if !val.struct_values().is_empty() {
            properties.append(&mut make_product_property_values(
                start_block_num,
                product_id,
                product_address,
                val.struct_values(),
            ));
        }
    }

    properties
}
