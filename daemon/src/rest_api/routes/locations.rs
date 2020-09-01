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

use crate::database::{
    helpers as db,
    models::{LatLongValue, Location, LocationPropertyValue},
};

use crate::rest_api::{
    error::RestApiResponseError, routes::DbExecutor, AcceptServiceIdParam, AppState, QueryServiceId,
};

use actix::{Handler, Message, SyncContext};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationSlice {
    pub location_id: String,
    pub location_address: String,
    pub location_namespace: String,
    pub owner: String,
    pub properties: Vec<LocationPropertyValueSlice>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

impl LocationSlice {
    pub fn from_model(location: &Location, properties: Vec<LocationPropertyValue>) -> Self {
        Self {
            location_id: location.location_id.clone(),
            location_address: location.location_address.clone(),
            location_namespace: location.location_namespace.clone(),
            owner: location.owner.clone(),
            properties: properties
                .iter()
                .map(|prop| LocationPropertyValueSlice::from_model(prop))
                .collect(),
            service_id: location.service_id.clone(),
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
    pub struct_values: Option<Vec<String>>,
    pub lat_long_value: LatLongSlice,
}

impl LocationPropertyValueSlice {
    pub fn from_model(property_value: &LocationPropertyValue) -> Self {
        Self {
            name: property_value.property_name.clone(),
            data_type: property_value.data_type.clone(),
            service_id: property_value.service_id.clone(),
            bytes_value: property_value.bytes_value.clone(),
            boolean_value: property_value.boolean_value,
            number_value: property_value.number_value,
            string_value: property_value.string_value.clone(),
            enum_value: property_value.enum_value,
            struct_values: property_value.struct_values.clone(),
            lat_long_value: LatLongSlice::from_model(property_value.lat_long_value.clone()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LatLongSlice {
    pub latitude: i64,
    pub longitude: i64,
}

impl LatLongSlice {
    pub fn new(latitude: i64, longitude: i64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    pub fn from_model(lat_long_value: Option<LatLongValue>) -> LatLongSlice {
        match lat_long_value {
            Some(value) => LatLongSlice::new(value.0 as i64, value.1 as i64),
            None => LatLongSlice::new(0 as i64, 0 as i64),
        }
    }
}

struct ListLocations {
    service_id: Option<String>,
}

impl Message for ListLocations {
    type Result = Result<Vec<LocationSlice>, RestApiResponseError>;
}

#[cfg(feature = "postgres")]
impl Handler<ListLocations> for DbExecutor<diesel::pg::PgConnection> {
    type Result = Result<Vec<LocationSlice>, RestApiResponseError>;

    fn handle(&mut self, msg: ListLocations, _: &mut SyncContext<Self>) -> Self::Result {
        let mut location_properties = db::list_location_property_values(
            &*self.connection_pool.get()?,
            msg.service_id.as_deref(),
        )?
        .into_iter()
        .fold(HashMap::new(), |mut acc, location_property| {
            acc.entry(location_property.location_id.to_string())
                .or_insert_with(Vec::new)
                .push(location_property);
            acc
        });

        let fetched_locations =
            db::list_locations(&*self.connection_pool.get()?, msg.service_id.as_deref())?
                .iter()
                .map(|location| {
                    LocationSlice::from_model(
                        location,
                        location_properties
                            .remove(&location.location_id)
                            .unwrap_or_else(Vec::new),
                    )
                })
                .collect();
        Ok(fetched_locations)
    }
}

#[cfg(feature = "postgres")]
pub async fn list_locations(
    state: web::Data<AppState<diesel::pg::PgConnection>>,
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

#[cfg(feature = "postgres")]
impl Handler<FetchLocation> for DbExecutor<diesel::pg::PgConnection> {
    type Result = Result<LocationSlice, RestApiResponseError>;

    fn handle(&mut self, msg: FetchLocation, _: &mut SyncContext<Self>) -> Self::Result {
        let location = match db::fetch_location(
            &*self.connection_pool.get()?,
            &msg.location_id,
            msg.service_id.as_deref(),
        )? {
            Some(location) => location,
            None => {
                return Err(RestApiResponseError::NotFoundError(format!(
                    "Could not find location with id: {}",
                    msg.location_id
                )));
            }
        };

        let location_properties = db::fetch_location_property_values(
            &*self.connection_pool.get()?,
            &msg.location_id,
            msg.service_id.as_deref(),
        )?;

        Ok(LocationSlice::from_model(&location, location_properties))
    }
}

#[cfg(feature = "postgres")]
pub async fn fetch_location(
    state: web::Data<AppState<diesel::pg::PgConnection>>,
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
