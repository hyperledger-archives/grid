// Copyright 2018-2020 Cargill Incorporated
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

pub(in crate::products) mod models;
mod operations;
pub(in crate) mod schema;

use crate::error::ResourceTemporarilyUnavailableError;
use crate::products::MAX_COMMIT_NUM;

use models::{NewProduct, NewProductPropertyValue, Product as ModelProduct, ProductPropertyValue};
use operations::{
    add_product::AddProductOperation, delete_product::DeleteProductOperation,
    get_product::GetProductOperation, list_products::ListProductsOperation,
    update_product::UpdateProductOperation, ProductStoreOperations,
};

use diesel::r2d2::{ConnectionManager, Pool};

use super::{LatLongValue, Product, ProductList, ProductStore, ProductStoreError, PropertyValue};

#[derive(Clone)]
pub struct DieselProductStore<C: diesel::Connection + 'static> {
    connection_pool: Pool<ConnectionManager<C>>,
}

impl<C: diesel::Connection> DieselProductStore<C> {
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> Self {
        DieselProductStore { connection_pool }
    }
}

#[cfg(feature = "postgres")]
impl ProductStore for DieselProductStore<diesel::pg::PgConnection> {
    fn add_product(&self, product: Product) -> Result<(), ProductStoreError> {
        ProductStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            ProductStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_product(product)
    }

    fn get_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Product>, ProductStoreError> {
        ProductStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            ProductStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_product(product_id, service_id)
    }

    fn list_products(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<ProductList, ProductStoreError> {
        ProductStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            ProductStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_products(service_id, offset, limit)
    }

    fn update_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError> {
        ProductStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            ProductStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .update_product(product_id, service_id, current_commit_num)
    }

    fn delete_product(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError> {
        ProductStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            ProductStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .delete_product(address, current_commit_num)
    }
}

#[cfg(feature = "sqlite")]
impl ProductStore for DieselProductStore<diesel::sqlite::SqliteConnection> {
    fn add_product(&self, product: Product) -> Result<(), ProductStoreError> {
        ProductStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            ProductStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_product(product)
    }

    fn get_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Product>, ProductStoreError> {
        ProductStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            ProductStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_product(product_id, service_id)
    }

    fn list_products(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<ProductList, ProductStoreError> {
        ProductStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            ProductStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_products(service_id, offset, limit)
    }

    fn update_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError> {
        ProductStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            ProductStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .update_product(product_id, service_id, current_commit_num)
    }

    fn delete_product(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError> {
        ProductStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            ProductStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .delete_product(address, current_commit_num)
    }
}

impl From<Product> for (NewProduct, Vec<NewProductPropertyValue>) {
    fn from(product: Product) -> Self {
        let new_product = NewProduct {
            product_id: product.product_id.clone(),
            product_address: product.product_address.clone(),
            product_namespace: product.product_namespace.clone(),
            owner: product.owner.clone(),
            start_commit_num: product.start_commit_num,
            end_commit_num: product.end_commit_num,
            service_id: product.service_id.clone(),
        };

        (new_product, make_property_values(None, &product.properties))
    }
}

impl From<(ModelProduct, Vec<PropertyValue>)> for Product {
    fn from((model, properties): (ModelProduct, Vec<PropertyValue>)) -> Self {
        Self {
            product_id: model.product_id,
            product_address: model.product_address,
            product_namespace: model.product_namespace,
            owner: model.owner,
            start_commit_num: model.start_commit_num,
            end_commit_num: model.end_commit_num,
            service_id: model.service_id,
            properties,
        }
    }
}

fn make_property_values(
    parent_property: Option<String>,
    properties: &[PropertyValue],
) -> Vec<NewProductPropertyValue> {
    let mut model_properties: Vec<NewProductPropertyValue> = Vec::new();

    for property in properties {
        model_properties.push(NewProductPropertyValue {
            product_id: property.product_id.clone(),
            product_address: property.product_address.clone(),
            property_name: property.property_name.clone(),
            parent_property: parent_property.clone(),
            data_type: property.data_type.clone(),
            bytes_value: property.bytes_value.clone(),
            boolean_value: property.boolean_value,
            number_value: property.number_value,
            string_value: property.string_value.clone(),
            enum_value: property.enum_value,
            latitude_value: property.lat_long_value.clone().map(|l| l.latitude),
            longitude_value: property.lat_long_value.clone().map(|l| l.longitude),
            start_commit_num: property.start_commit_num,
            end_commit_num: MAX_COMMIT_NUM,
            service_id: property.service_id.clone(),
        });

        if !property.struct_values.is_empty() {
            model_properties.append(&mut make_property_values(
                Some(format!(
                    "{}:{}",
                    property.product_id, property.property_name
                )),
                &property.struct_values,
            ));
        }
    }

    model_properties
}

impl From<ProductPropertyValue> for PropertyValue {
    fn from(model: ProductPropertyValue) -> Self {
        Self {
            product_id: model.product_id,
            product_address: model.product_address,
            property_name: model.property_name,
            data_type: model.data_type,
            bytes_value: model.bytes_value,
            boolean_value: model.boolean_value,
            number_value: model.number_value,
            string_value: model.string_value,
            enum_value: model.enum_value,
            struct_values: vec![],
            lat_long_value: if model.latitude_value.is_some() && model.longitude_value.is_some() {
                Some(LatLongValue {
                    latitude: model.latitude_value.unwrap(),
                    longitude: model.longitude_value.unwrap(),
                })
            } else {
                None
            },
            start_commit_num: model.start_commit_num,
            end_commit_num: model.end_commit_num,
            service_id: model.service_id,
        }
    }
}

impl From<(ProductPropertyValue, Vec<PropertyValue>)> for PropertyValue {
    fn from((model, children): (ProductPropertyValue, Vec<PropertyValue>)) -> Self {
        Self {
            product_id: model.product_id,
            product_address: model.product_address,
            property_name: model.property_name,
            data_type: model.data_type,
            bytes_value: model.bytes_value,
            boolean_value: model.boolean_value,
            number_value: model.number_value,
            string_value: model.string_value,
            enum_value: model.enum_value,
            struct_values: children,
            lat_long_value: if model.latitude_value.is_some() && model.longitude_value.is_some() {
                Some(LatLongValue {
                    latitude: model.latitude_value.unwrap(),
                    longitude: model.longitude_value.unwrap(),
                })
            } else {
                None
            },
            start_commit_num: model.start_commit_num,
            end_commit_num: model.end_commit_num,
            service_id: model.service_id,
        }
    }
}
