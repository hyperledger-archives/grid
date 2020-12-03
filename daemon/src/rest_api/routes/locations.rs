// Copyright 2020 Cargill Incorporated
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
use grid_sdk::locations::store::{LatLongValue, Location, LocationAttribute};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationSlice {
    pub location_id: String,
    pub location_namespace: String,
    pub owner: String,
    pub properties: Vec<LocationPropertyValueSlice>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

impl From<Location> for LocationSlice {
    fn from(location: Location) -> Self {
        Self {
            location_id: location.location_id,
            location_namespace: location.location_namespace,
            owner: location.owner,
            properties: location
                .attributes
                .into_iter()
                .map(LocationPropertyValueSlice::from)
                .collect(),
            service_id: location.service_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationPropertyValueSlice {
    pub name: String,
    pub data_type: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub struct_values: Option<Vec<LocationPropertyValueSlice>>,
    pub lat_long_value: Option<LatLongSlice>,
}

impl From<LocationAttribute> for LocationPropertyValueSlice {
    fn from(attribute: LocationAttribute) -> Self {
        Self {
            name: attribute.property_name,
            data_type: attribute.data_type,
            service_id: attribute.service_id,
            bytes_value: attribute.bytes_value,
            boolean_value: attribute.boolean_value,
            number_value: attribute.number_value,
            string_value: attribute.string_value.clone(),
            enum_value: attribute.enum_value,
            struct_values: attribute.struct_values.map(|attrs| {
                attrs
                    .into_iter()
                    .map(LocationPropertyValueSlice::from)
                    .collect()
            }),
            lat_long_value: attribute.lat_long_value.map(LatLongSlice::from),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LatLongSlice {
    pub latitude: i64,
    pub longitude: i64,
}

impl From<LatLongValue> for LatLongSlice {
    fn from(lat_long_value: LatLongValue) -> Self {
        Self {
            latitude: lat_long_value.0,
            longitude: lat_long_value.1,
        }
    }
}

struct ListLocations {
    service_id: Option<String>,
}

impl Message for ListLocations {
    type Result = Result<Vec<LocationSlice>, RestApiResponseError>;
}

impl Handler<ListLocations> for DbExecutor {
    type Result = Result<Vec<LocationSlice>, RestApiResponseError>;

    fn handle(&mut self, msg: ListLocations, _: &mut SyncContext<Self>) -> Self::Result {
        Ok(self
            .location_store
            .list_locations(msg.service_id.as_deref())?
            .into_iter()
            .map(LocationSlice::from)
            .collect())
    }
}

pub async fn list_locations(
    state: web::Data<AppState>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    state
        .database_connection
        .send(ListLocations {
            service_id: query.into_inner().service_id,
        })
        .await?
        .map(|locations| HttpResponse::Ok().json(locations))
}

struct FetchLocation {
    location_id: String,
    service_id: Option<String>,
}

impl Message for FetchLocation {
    type Result = Result<LocationSlice, RestApiResponseError>;
}

impl Handler<FetchLocation> for DbExecutor {
    type Result = Result<LocationSlice, RestApiResponseError>;

    fn handle(&mut self, msg: FetchLocation, _: &mut SyncContext<Self>) -> Self::Result {
        match self
            .location_store
            .fetch_location(&msg.location_id, msg.service_id.as_deref())?
        {
            Some(location) => Ok(LocationSlice::from(location)),
            None => Err(RestApiResponseError::NotFoundError(format!(
                "Could not find location with id: {}",
                msg.location_id
            ))),
        }
    }
}

pub async fn fetch_location(
    state: web::Data<AppState>,
    location_id: web::Path<String>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    state
        .database_connection
        .send(FetchLocation {
            location_id: location_id.into_inner(),
            service_id: query.into_inner().service_id,
        })
        .await?
        .map(|location| HttpResponse::Ok().json(location))
}
