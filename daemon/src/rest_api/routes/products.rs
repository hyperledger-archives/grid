/*
 * Copyright (c) 2019 Target Brands, Inc.
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

use crate::rest_api::{
    error::RestApiResponseError, routes::DbExecutor, AcceptServiceIdParam, AppState, QueryServiceId,
};

use actix::{Handler, Message, SyncContext};
use actix_web::{web, HttpResponse};
use grid_sdk::products::store::{LatLongValue, Product, PropertyValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductSlice {
    pub product_id: String,
    pub product_address: String,
    pub product_namespace: String,
    pub owner: String,
    pub properties: Vec<ProductPropertyValueSlice>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

impl From<Product> for ProductSlice {
    fn from(product: Product) -> Self {
        Self {
            product_id: product.product_id.clone(),
            product_address: product.product_address.clone(),
            product_namespace: product.product_namespace.clone(),
            owner: product.owner.clone(),
            properties: product
                .properties
                .into_iter()
                .map(ProductPropertyValueSlice::from)
                .collect(),
            service_id: product.service_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductPropertyValueSlice {
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
    pub struct_values: Vec<ProductPropertyValueSlice>,
    pub lat_long_value: Option<LatLongSlice>,
}

impl From<PropertyValue> for ProductPropertyValueSlice {
    fn from(property_value: PropertyValue) -> Self {
        Self {
            name: property_value.property_name.clone(),
            data_type: property_value.data_type.clone(),
            service_id: property_value.service_id.clone(),
            bytes_value: property_value.bytes_value.clone(),
            boolean_value: property_value.boolean_value,
            number_value: property_value.number_value,
            string_value: property_value.string_value.clone(),
            enum_value: property_value.enum_value,
            struct_values: property_value
                .struct_values
                .into_iter()
                .map(ProductPropertyValueSlice::from)
                .collect(),
            lat_long_value: property_value.lat_long_value.map(LatLongSlice::from),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LatLongSlice {
    pub latitude: i64,
    pub longitude: i64,
}

impl From<LatLongValue> for LatLongSlice {
    fn from(value: LatLongValue) -> Self {
        LatLongSlice {
            latitude: value.latitude,
            longitude: value.longitude,
        }
    }
}

struct ListProducts {
    service_id: Option<String>,
}

impl Message for ListProducts {
    type Result = Result<Vec<ProductSlice>, RestApiResponseError>;
}

impl Handler<ListProducts> for DbExecutor {
    type Result = Result<Vec<ProductSlice>, RestApiResponseError>;

    fn handle(&mut self, msg: ListProducts, _: &mut SyncContext<Self>) -> Self::Result {
        Ok(self
            .product_store
            .list_products(msg.service_id.as_deref())?
            .into_iter()
            .map(ProductSlice::from)
            .collect())
    }
}

pub async fn list_products(
    state: web::Data<AppState>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    state
        .database_connection
        .send(ListProducts {
            service_id: query.into_inner().service_id,
        })
        .await?
        .map(|products| HttpResponse::Ok().json(products))
}

struct FetchProduct {
    product_id: String,
    service_id: Option<String>,
}

impl Message for FetchProduct {
    type Result = Result<ProductSlice, RestApiResponseError>;
}

impl Handler<FetchProduct> for DbExecutor {
    type Result = Result<ProductSlice, RestApiResponseError>;

    fn handle(&mut self, msg: FetchProduct, _: &mut SyncContext<Self>) -> Self::Result {
        match self
            .product_store
            .fetch_product(&msg.product_id, msg.service_id.as_deref())?
        {
            Some(product) => Ok(ProductSlice::from(product)),
            None => Err(RestApiResponseError::NotFoundError(format!(
                "Could not find product with id: {}",
                msg.product_id
            ))),
        }
    }
}

pub async fn fetch_product(
    state: web::Data<AppState>,
    product_id: web::Path<String>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    state
        .database_connection
        .send(FetchProduct {
            product_id: product_id.into_inner(),
            service_id: query.into_inner().service_id,
        })
        .await?
        .map(|product| HttpResponse::Ok().json(product))
}
