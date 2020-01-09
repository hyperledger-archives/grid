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

use crate::database::{
    helpers as db,
    models::{LatLongValue, Product, ProductPropertyValue},
};

use crate::rest_api::{
    error::RestApiResponseError, routes::DbExecutor, AcceptServiceIdParam, AppState, QueryServiceId,
};

use actix::{Handler, Message, SyncContext};
use actix_web::{web, HttpResponse};
use futures::Future;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductSlice {
    pub product_id: String,
    pub product_address: String,
    pub product_namespace: String,
    pub owner: String,
    pub properties: Vec<ProductPropertyValueSlice>,
}

impl ProductSlice {
    pub fn from_model(product: &Product, properties: Vec<ProductPropertyValue>) -> Self {
        Self {
            product_id: product.product_id.clone(),
            product_address: product.product_address.clone(),
            product_namespace: product.product_namespace.clone(),
            owner: product.owner.clone(),
            properties: properties
                .iter()
                .map(|prop| ProductPropertyValueSlice::from_model(prop))
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductPropertyValueSlice {
    pub name: String,
    pub data_type: String,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub struct_values: Option<Vec<String>>,
    pub lat_long_value: LatLongSlice,
}

impl ProductPropertyValueSlice {
    pub fn from_model(property_value: &ProductPropertyValue) -> Self {
        Self {
            name: property_value.property_name.clone(),
            data_type: property_value.data_type.clone(),
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

struct ListProducts {
    service_id: Option<String>,
}

impl Message for ListProducts {
    type Result = Result<Vec<ProductSlice>, RestApiResponseError>;
}

impl Handler<ListProducts> for DbExecutor {
    type Result = Result<Vec<ProductSlice>, RestApiResponseError>;

    fn handle(&mut self, msg: ListProducts, _: &mut SyncContext<Self>) -> Self::Result {
        let mut product_properties = db::list_product_property_values(
            &*self.connection_pool.get()?,
            msg.service_id.as_deref(),
        )?
        .into_iter()
        .fold(HashMap::new(), |mut acc, product_property| {
            acc.entry(product_property.product_id.to_string())
                .or_insert_with(|| vec![])
                .push(product_property);
            acc
        });

        let fetched_products =
            db::list_products(&*self.connection_pool.get()?, msg.service_id.as_deref())?
                .iter()
                .map(|product| {
                    ProductSlice::from_model(
                        product,
                        product_properties
                            .remove(&product.product_id)
                            .unwrap_or_else(|| vec![]),
                    )
                })
                .collect();
        Ok(fetched_products)
    }
}

pub fn list_products(
    state: web::Data<AppState>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Box<dyn Future<Item = HttpResponse, Error = RestApiResponseError>> {
    Box::new(
        state
            .database_connection
            .send(ListProducts {
                service_id: query.into_inner().service_id,
            })
            .from_err()
            .and_then(move |res| match res {
                Ok(products) => Ok(HttpResponse::Ok().json(products)),
                Err(err) => Err(err),
            }),
    )
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
        let product = match db::fetch_product(
            &*self.connection_pool.get()?,
            &msg.product_id,
            msg.service_id.as_deref(),
        )? {
            Some(product) => product,
            None => {
                return Err(RestApiResponseError::NotFoundError(format!(
                    "Could not find product with id: {}",
                    msg.product_id
                )));
            }
        };

        let product_properties = db::fetch_product_property_values(
            &*self.connection_pool.get()?,
            &msg.product_id,
            msg.service_id.as_deref(),
        )?;

        Ok(ProductSlice::from_model(&product, product_properties))
    }
}

pub fn fetch_product(
    state: web::Data<AppState>,
    product_id: web::Path<String>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> impl Future<Item = HttpResponse, Error = RestApiResponseError> {
    Box::new(
        state
            .database_connection
            .send(FetchProduct {
                product_id: product_id.into_inner(),
                service_id: query.into_inner().service_id,
            })
            .from_err()
            .and_then(move |res| match res {
                Ok(product) => Ok(HttpResponse::Ok().json(product)),
                Err(err) => Err(err),
            }),
    )
}
