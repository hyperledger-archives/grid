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

use super::ProductStoreOperations;

use crate::grid_db::products::{
    store::{
        diesel::{
            models::{Product as ModelProduct, ProductPropertyValue},
            schema::{product, product_property_value},
        },
        error::ProductStoreError,
        Product, PropertyValue,
    },
    MAX_COMMIT_NUM,
};
use diesel::{prelude::*, result::Error::NotFound};

pub(in crate::grid_db::products) trait FetchProductOperation {
    fn fetch_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Product>, ProductStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> FetchProductOperation for ProductStoreOperations<'a, diesel::pg::PgConnection> {
    fn fetch_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Product>, ProductStoreError> {
        let product = if let Some(product) = pg::fetch_product(&*self.conn, product_id, service_id)?
        {
            product
        } else {
            return Ok(None);
        };

        let root_values = pg::get_root_values(&*self.conn, product_id)?;

        let values = pg::get_property_values(&*self.conn, root_values)?;

        Ok(Some(Product::from((product, values))))
    }
}

#[cfg(feature = "sqlite")]
impl<'a> FetchProductOperation for ProductStoreOperations<'a, diesel::sqlite::SqliteConnection> {
    fn fetch_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Product>, ProductStoreError> {
        let product =
            if let Some(product) = sqlite::fetch_product(&*self.conn, product_id, service_id)? {
                product
            } else {
                return Ok(None);
            };

        let root_values = sqlite::get_root_values(&*self.conn, product_id)?;

        let values = sqlite::get_property_values(&*self.conn, root_values)?;

        Ok(Some(Product::from((product, values))))
    }
}

#[cfg(feature = "postgres")]
mod pg {
    use super::*;

    pub fn fetch_product(
        conn: &PgConnection,
        product_id: &str,
        service_id: Option<&str>,
    ) -> QueryResult<Option<ModelProduct>> {
        let mut query = product::table
            .into_boxed()
            .select(product::all_columns)
            .filter(
                product::product_id
                    .eq(product_id)
                    .and(product::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(product::service_id.eq(service_id));
        } else {
            query = query.filter(product::service_id.is_null());
        }

        query
            .first(conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
    }

    pub fn get_root_values(
        conn: &PgConnection,
        product_id: &str,
    ) -> QueryResult<Vec<ProductPropertyValue>> {
        product_property_value::table
            .select(product_property_value::all_columns)
            .filter(
                product_property_value::product_id
                    .eq(product_id)
                    .and(product_property_value::parent_property.is_null())
                    .and(product_property_value::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .load::<ProductPropertyValue>(conn)
    }

    pub fn get_property_values(
        conn: &PgConnection,
        root_values: Vec<ProductPropertyValue>,
    ) -> Result<Vec<PropertyValue>, ProductStoreError> {
        let mut definitions = Vec::new();

        for root_value in root_values {
            let children = product_property_value::table
                .select(product_property_value::all_columns)
                .filter(product_property_value::parent_property.eq(&root_value.parent_property))
                .load(conn)?;

            if children.is_empty() {
                definitions.push(PropertyValue::from(root_value));
            } else {
                definitions.push(PropertyValue::from((
                    root_value,
                    get_property_values(conn, children)?,
                )));
            }
        }

        Ok(definitions)
    }
}

#[cfg(feature = "sqlite")]
mod sqlite {
    use super::*;

    pub fn fetch_product(
        conn: &SqliteConnection,
        product_id: &str,
        service_id: Option<&str>,
    ) -> QueryResult<Option<ModelProduct>> {
        let mut query = product::table
            .into_boxed()
            .select(product::all_columns)
            .filter(
                product::product_id
                    .eq(product_id)
                    .and(product::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(product::service_id.eq(service_id));
        } else {
            query = query.filter(product::service_id.is_null());
        }

        query
            .first(conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
    }

    pub fn get_root_values(
        conn: &SqliteConnection,
        product_id: &str,
    ) -> QueryResult<Vec<ProductPropertyValue>> {
        product_property_value::table
            .select(product_property_value::all_columns)
            .filter(
                product_property_value::product_id
                    .eq(product_id)
                    .and(product_property_value::parent_property.is_null())
                    .and(product_property_value::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .load::<ProductPropertyValue>(conn)
    }

    pub fn get_property_values(
        conn: &SqliteConnection,
        root_values: Vec<ProductPropertyValue>,
    ) -> Result<Vec<PropertyValue>, ProductStoreError> {
        let mut definitions = Vec::new();

        for root_value in root_values {
            let children = product_property_value::table
                .select(product_property_value::all_columns)
                .filter(product_property_value::parent_property.eq(&root_value.parent_property))
                .load(conn)?;

            if children.is_empty() {
                definitions.push(PropertyValue::from(root_value));
            } else {
                definitions.push(PropertyValue::from((
                    root_value,
                    get_property_values(conn, children)?,
                )));
            }
        }

        Ok(definitions)
    }
}
