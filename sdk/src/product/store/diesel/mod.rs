// Copyright 2018-2021 Cargill Incorporated
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

pub(in crate::product) mod models;
mod operations;
pub(in crate) mod schema;

use crate::error::ResourceTemporarilyUnavailableError;

use operations::{
    add_product::AddProductOperation, delete_product::DeleteProductOperation,
    get_product::GetProductOperation, list_products::ListProductsOperation,
    update_product::UpdateProductOperation, ProductStoreOperations,
};

use diesel::connection::AnsiTransactionManager;
use diesel::r2d2::{ConnectionManager, Pool};

use super::{Product, ProductList, ProductStore, ProductStoreError};

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

pub struct DieselConnectionProductStore<'a, C>
where
    C: diesel::Connection<TransactionManager = AnsiTransactionManager> + 'static,
    C::Backend: diesel::backend::UsesAnsiSavepointSyntax,
{
    connection: &'a C,
}

impl<'a, C> DieselConnectionProductStore<'a, C>
where
    C: diesel::Connection<TransactionManager = AnsiTransactionManager> + 'static,
    C::Backend: diesel::backend::UsesAnsiSavepointSyntax,
{
    pub fn new(connection: &'a C) -> Self {
        DieselConnectionProductStore { connection }
    }
}

#[cfg(feature = "postgres")]
impl<'a> ProductStore for DieselConnectionProductStore<'a, diesel::pg::PgConnection> {
    fn add_product(&self, product: Product) -> Result<(), ProductStoreError> {
        ProductStoreOperations::new(self.connection).add_product(product)
    }

    fn get_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Product>, ProductStoreError> {
        ProductStoreOperations::new(self.connection).get_product(product_id, service_id)
    }

    fn list_products(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<ProductList, ProductStoreError> {
        ProductStoreOperations::new(self.connection).list_products(service_id, offset, limit)
    }

    fn update_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError> {
        ProductStoreOperations::new(self.connection).update_product(
            product_id,
            service_id,
            current_commit_num,
        )
    }

    fn delete_product(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError> {
        ProductStoreOperations::new(self.connection).delete_product(address, current_commit_num)
    }
}

#[cfg(feature = "sqlite")]
impl<'a> ProductStore for DieselConnectionProductStore<'a, diesel::sqlite::SqliteConnection> {
    fn add_product(&self, product: Product) -> Result<(), ProductStoreError> {
        ProductStoreOperations::new(self.connection).add_product(product)
    }

    fn get_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Product>, ProductStoreError> {
        ProductStoreOperations::new(self.connection).get_product(product_id, service_id)
    }

    fn list_products(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<ProductList, ProductStoreError> {
        ProductStoreOperations::new(self.connection).list_products(service_id, offset, limit)
    }

    fn update_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError> {
        ProductStoreOperations::new(self.connection).update_product(
            product_id,
            service_id,
            current_commit_num,
        )
    }

    fn delete_product(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError> {
        ProductStoreOperations::new(self.connection).delete_product(address, current_commit_num)
    }
}
