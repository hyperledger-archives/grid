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

use crate::rest_api::{
    error::RestApiResponseError, routes::DbExecutor, AcceptServiceIdParam, AppState, QueryServiceId,
};

use actix::{Handler, Message, SyncContext};
use actix_web::{web, HttpResponse};
use grid_sdk::schemas::store::{PropertyDefinition, Schema};
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
}

impl Message for ListGridSchemas {
    type Result = Result<Vec<GridSchemaSlice>, RestApiResponseError>;
}

impl Handler<ListGridSchemas> for DbExecutor {
    type Result = Result<Vec<GridSchemaSlice>, RestApiResponseError>;

    fn handle(&mut self, msg: ListGridSchemas, _: &mut SyncContext<Self>) -> Self::Result {
        Ok(self
            .schema_store
            .list_schemas(msg.service_id.as_deref())?
            .into_iter()
            .map(GridSchemaSlice::from)
            .collect())
    }
}

pub async fn list_grid_schemas(
    state: web::Data<AppState>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    state
        .database_connection
        .send(ListGridSchemas {
            service_id: query.into_inner().service_id,
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
