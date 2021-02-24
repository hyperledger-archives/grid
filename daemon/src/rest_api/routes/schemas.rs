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

use std::convert::TryFrom;

use crate::rest_api::{
    error::RestApiResponseError, routes::DbExecutor, AcceptServiceIdParam, AppState, QueryPaging,
    QueryServiceId,
};

use actix::{Handler, Message, SyncContext};
use actix_web::{web, HttpResponse};
use grid_sdk::{
    rest_api::resources::paging::v1::Paging,
    schemas::store::{PropertyDefinition, Schema},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GridSchemaSlice {
    pub name: String,
    pub description: String,
    pub owner: String,
    pub properties: Vec<GridPropertyDefinitionSlice>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GridSchemaListSlice {
    pub data: Vec<GridSchemaSlice>,
    pub paging: Paging,
}

impl From<Schema> for GridSchemaSlice {
    fn from(schema: Schema) -> Self {
        Self {
            name: schema.name.clone(),
            description: schema.description.clone(),
            owner: schema.owner.clone(),
            properties: schema
                .properties
                .into_iter()
                .map(GridPropertyDefinitionSlice::from)
                .collect(),
            service_id: schema.service_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GridPropertyDefinitionSlice {
    pub name: String,
    pub schema_name: String,
    pub data_type: String,
    pub required: bool,
    pub description: String,
    pub number_exponent: i64,
    pub enum_options: Vec<String>,
    pub struct_properties: Vec<GridPropertyDefinitionSlice>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

impl From<PropertyDefinition> for GridPropertyDefinitionSlice {
    fn from(definition: PropertyDefinition) -> Self {
        Self {
            name: definition.name.clone(),
            schema_name: definition.schema_name.clone(),
            data_type: definition.data_type.clone(),
            required: definition.required,
            description: definition.description.clone(),
            number_exponent: definition.number_exponent,
            enum_options: definition.enum_options.clone(),
            struct_properties: definition
                .struct_properties
                .into_iter()
                .map(GridPropertyDefinitionSlice::from)
                .collect(),
            service_id: definition.service_id,
        }
    }
}

struct ListGridSchemas {
    service_id: Option<String>,
    offset: u64,
    limit: u16,
}

impl Message for ListGridSchemas {
    type Result = Result<GridSchemaListSlice, RestApiResponseError>;
}

impl Handler<ListGridSchemas> for DbExecutor {
    type Result = Result<GridSchemaListSlice, RestApiResponseError>;

    fn handle(&mut self, msg: ListGridSchemas, _: &mut SyncContext<Self>) -> Self::Result {
        let offset = i64::try_from(msg.offset).unwrap_or(i64::MAX);

        let limit = i64::try_from(msg.limit).unwrap_or(10);

        let schema_list =
            self.schema_store
                .list_schemas(msg.service_id.as_deref(), offset, limit)?;

        let data = schema_list
            .data
            .into_iter()
            .map(GridSchemaSlice::from)
            .collect();

        let paging = Paging::new("/schema", schema_list.paging, msg.service_id.as_deref());

        Ok(GridSchemaListSlice { data, paging })
    }
}

pub async fn list_grid_schemas(
    state: web::Data<AppState>,
    query_service_id: web::Query<QueryServiceId>,
    query_paging: web::Query<QueryPaging>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    let paging = query_paging.into_inner();
    state
        .database_connection
        .send(ListGridSchemas {
            service_id: query_service_id.into_inner().service_id,
            offset: paging.offset(),
            limit: paging.limit(),
        })
        .await?
        .map(|schemas| HttpResponse::Ok().json(schemas))
}

struct FetchGridSchema {
    name: String,
    service_id: Option<String>,
}

impl Message for FetchGridSchema {
    type Result = Result<GridSchemaSlice, RestApiResponseError>;
}

impl Handler<FetchGridSchema> for DbExecutor {
    type Result = Result<GridSchemaSlice, RestApiResponseError>;

    fn handle(&mut self, msg: FetchGridSchema, _: &mut SyncContext<Self>) -> Self::Result {
        match self
            .schema_store
            .fetch_schema(&msg.name, msg.service_id.as_deref())?
        {
            Some(schema) => Ok(GridSchemaSlice::from(schema)),
            None => Err(RestApiResponseError::NotFoundError(format!(
                "Could not find schema with name: {}",
                msg.name
            ))),
        }
    }
}

pub async fn fetch_grid_schema(
    state: web::Data<AppState>,
    schema_name: web::Path<String>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    state
        .database_connection
        .send(FetchGridSchema {
            name: schema_name.into_inner(),
            service_id: query.into_inner().service_id,
        })
        .await?
        .map(|schema| HttpResponse::Ok().json(schema))
}
