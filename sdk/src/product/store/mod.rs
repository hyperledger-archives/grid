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

#[cfg(feature = "diesel")]
pub(in crate) mod diesel;
pub mod error;

use crate::paging::Paging;

#[cfg(feature = "diesel")]
pub use self::diesel::{DieselConnectionProductStore, DieselProductStore};
pub use error::{ProductBuilderError, ProductStoreError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    product_id: String,
    product_address: String,
    product_namespace: String,
    owner: String,
    start_commit_num: i64,
    end_commit_num: i64,
    service_id: Option<String>,
    last_updated: Option<i64>,
    properties: Vec<PropertyValue>,
}

impl Product {
    /// Returns the product_id for the product
    pub fn product_id(&self) -> &str {
        &self.product_id
    }

    /// Returns the product_address for the product
    pub fn product_address(&self) -> &str {
        &self.product_address
    }

    /// Returns the product_namespace for the product
    pub fn product_namespace(&self) -> &str {
        &self.product_namespace
    }

    /// Returns the owner for the product
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Returns the start_commit_num for the product
    pub fn start_commit_num(&self) -> &i64 {
        &self.start_commit_num
    }

    /// Returns the end_commit_num for the product
    pub fn end_commit_num(&self) -> &i64 {
        &self.end_commit_num
    }

    /// Returns the service_id for the product
    pub fn service_id(&self) -> Option<&str> {
        self.service_id.as_deref()
    }

    /// Returns the last updated timestamp for the product
    pub fn last_updated(&self) -> Option<&i64> {
        self.last_updated.as_ref()
    }

    /// Returns the properties for the product
    pub fn properties(&self) -> Vec<PropertyValue> {
        self.properties.to_vec()
    }
}

/// Builder used to create a Product
#[derive(Default, Clone)]
pub struct ProductBuilder {
    product_id: String,
    product_address: String,
    product_namespace: String,
    owner: String,
    start_commit_num: i64,
    end_commit_num: i64,
    service_id: Option<String>,
    last_updated: Option<i64>,
    properties: Vec<PropertyValue>,
}

impl ProductBuilder {
    /// Sets the product ID for this product
    pub fn with_product_id(mut self, product_id: String) -> Self {
        self.product_id = product_id;
        self
    }

    /// Sets the product address for this product
    pub fn with_product_address(mut self, product_address: String) -> Self {
        self.product_address = product_address;
        self
    }

    /// Sets the product namespace for this product
    pub fn with_product_namespace(mut self, product_namespace: String) -> Self {
        self.product_namespace = product_namespace;
        self
    }

    /// Sets the owner of the product
    pub fn with_owner(mut self, owner: String) -> Self {
        self.owner = owner;
        self
    }

    /// Sets the start commit number for this product
    pub fn with_start_commit_number(mut self, start_commit_num: i64) -> Self {
        self.start_commit_num = start_commit_num;
        self
    }

    /// Sets the end commit number for this product
    pub fn with_end_commit_number(mut self, end_commit_num: i64) -> Self {
        self.end_commit_num = end_commit_num;
        self
    }

    /// Sets the service ID for this product
    pub fn with_service_id(mut self, service_id: Option<String>) -> Self {
        self.service_id = service_id;
        self
    }

    /// Sets the last updated timestamp for this product
    pub fn with_last_updated(mut self, last_updated: Option<i64>) -> Self {
        self.last_updated = last_updated;
        self
    }

    /// Sets the properties for this product
    pub fn with_properties(mut self, properties: Vec<PropertyValue>) -> Self {
        self.properties = properties;
        self
    }

    pub fn build(self) -> Result<Product, ProductBuilderError> {
        let ProductBuilder {
            product_id,
            product_address,
            product_namespace,
            owner,
            start_commit_num,
            end_commit_num,
            service_id,
            last_updated,
            properties,
        } = self;

        if product_id.is_empty() {
            return Err(ProductBuilderError::MissingRequiredField(
                "Missing product_id".to_string(),
            ));
        };

        if product_address.is_empty() {
            return Err(ProductBuilderError::MissingRequiredField(
                "Missing product_address".to_string(),
            ));
        };

        if product_namespace.is_empty() {
            return Err(ProductBuilderError::MissingRequiredField(
                "Missing product_namespace".to_string(),
            ));
        };

        if owner.is_empty() {
            return Err(ProductBuilderError::MissingRequiredField(
                "Missing owner".to_string(),
            ));
        };

        if start_commit_num >= end_commit_num {
            return Err(ProductBuilderError::MissingRequiredField(
                "start_commit_number must be less than end_commit_num".to_string(),
            ));
        };

        if end_commit_num <= start_commit_num {
            return Err(ProductBuilderError::MissingRequiredField(
                "end_commit_number must be greater than start_commit_num".to_string(),
            ));
        };

        Ok(Product {
            product_id,
            product_address,
            product_namespace,
            owner,
            start_commit_num,
            end_commit_num,
            service_id,
            last_updated,
            properties,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyValue {
    product_id: String,
    product_address: String,
    property_name: String,
    data_type: String,
    bytes_value: Option<Vec<u8>>,
    boolean_value: Option<bool>,
    number_value: Option<i64>,
    string_value: Option<String>,
    enum_value: Option<i32>,
    struct_values: Vec<PropertyValue>,
    lat_long_value: Option<LatLongValue>,
    start_commit_num: i64,
    end_commit_num: i64,
    service_id: Option<String>,
}

impl PropertyValue {
    /// Returns the product_id for the property value
    pub fn product_id(&self) -> &str {
        &self.product_id
    }

    /// Returns the product_address for the property value
    pub fn product_address(&self) -> &str {
        &self.product_address
    }

    /// Returns the property_name for the property value
    pub fn property_name(&self) -> &str {
        &self.property_name
    }

    /// Returns the data_type for the property value
    pub fn data_type(&self) -> &str {
        &self.data_type
    }

    /// Returns the bytes_value for the property value
    pub fn bytes_value(&self) -> Option<Vec<u8>> {
        self.bytes_value.clone()
    }

    /// Returns the boolean_value for the property value
    pub fn boolean_value(&self) -> Option<bool> {
        self.boolean_value
    }

    /// Returns the number_value for the property value
    pub fn number_value(&self) -> Option<i64> {
        self.number_value
    }

    /// Returns the string_value for the property value
    pub fn string_value(&self) -> Option<&str> {
        self.string_value.as_deref()
    }

    /// Returns the enum_value for the property value
    pub fn enum_value(&self) -> Option<i32> {
        self.enum_value
    }

    /// Returns the struct_values for the property value
    pub fn struct_values(&self) -> Vec<PropertyValue> {
        self.struct_values.clone()
    }

    /// Returns the lat_long_value for the property value
    pub fn lat_long_value(&self) -> Option<LatLongValue> {
        self.lat_long_value.as_ref().cloned()
    }

    /// Returns the start_commit_num for the property value
    pub fn start_commit_num(&self) -> &i64 {
        &self.start_commit_num
    }

    /// Returns the end_commit_num for the property value
    pub fn end_commit_num(&self) -> &i64 {
        &self.end_commit_num
    }

    /// Returns the service_id for the property value
    pub fn service_id(&self) -> Option<&str> {
        self.service_id.as_deref()
    }
}

/// Builder used to create a PropertyValue
#[derive(Default, Clone)]
pub struct PropertyValueBuilder {
    product_id: String,
    product_address: String,
    property_name: String,
    data_type: String,
    bytes_value: Option<Vec<u8>>,
    boolean_value: Option<bool>,
    number_value: Option<i64>,
    string_value: Option<String>,
    enum_value: Option<i32>,
    struct_values: Vec<PropertyValue>,
    lat_long_value: Option<LatLongValue>,
    start_commit_num: i64,
    end_commit_num: i64,
    service_id: Option<String>,
}

impl PropertyValueBuilder {
    /// Sets the product ID for this property value
    pub fn with_product_id(mut self, product_id: String) -> Self {
        self.product_id = product_id;
        self
    }

    /// Sets the product address for this property value
    pub fn with_product_address(mut self, product_address: String) -> Self {
        self.product_address = product_address;
        self
    }

    /// Sets the property name for this property value
    pub fn with_property_name(mut self, property_name: String) -> Self {
        self.property_name = property_name;
        self
    }

    /// Sets the data type for this property value
    pub fn with_data_type(mut self, data_type: String) -> Self {
        self.data_type = data_type;
        self
    }

    /// Sets the bytes value for this property value
    pub fn with_bytes_value(mut self, bytes_value: Option<Vec<u8>>) -> Self {
        self.bytes_value = bytes_value;
        self
    }

    /// Sets the boolean value for this property value
    pub fn with_boolean_value(mut self, boolean_value: Option<bool>) -> Self {
        self.boolean_value = boolean_value;
        self
    }

    /// Sets the number value for this property value
    pub fn with_number_value(mut self, number_value: Option<i64>) -> Self {
        self.number_value = number_value;
        self
    }

    /// Sets the string value for this property value
    pub fn with_string_value(mut self, string_value: Option<String>) -> Self {
        self.string_value = string_value;
        self
    }

    /// Sets the enum value for this property value
    pub fn with_enum_value(mut self, enum_value: Option<i32>) -> Self {
        self.enum_value = enum_value;
        self
    }

    /// Sets the struct values for this property value
    pub fn with_struct_values(mut self, struct_values: Vec<PropertyValue>) -> Self {
        self.struct_values = struct_values;
        self
    }

    /// Sets the LatLong value for this property value
    pub fn with_lat_long_value(mut self, lat_long_value: Option<LatLongValue>) -> Self {
        self.lat_long_value = lat_long_value;
        self
    }

    /// Sets the start commit number for this property value
    pub fn with_start_commit_number(mut self, start_commit_num: i64) -> Self {
        self.start_commit_num = start_commit_num;
        self
    }

    /// Sets the end commit number for this property value
    pub fn with_end_commit_number(mut self, end_commit_num: i64) -> Self {
        self.end_commit_num = end_commit_num;
        self
    }

    /// Sets the service ID for this property value
    pub fn with_service_id(mut self, service_id: Option<String>) -> Self {
        self.service_id = service_id;
        self
    }

    pub fn build(self) -> Result<PropertyValue, ProductBuilderError> {
        let PropertyValueBuilder {
            product_id,
            product_address,
            property_name,
            data_type,
            bytes_value,
            boolean_value,
            number_value,
            string_value,
            enum_value,
            struct_values,
            lat_long_value,
            start_commit_num,
            end_commit_num,
            service_id,
        } = self;

        if product_id.is_empty() {
            return Err(ProductBuilderError::MissingRequiredField(
                "Missing product_id".to_string(),
            ));
        };

        if product_address.is_empty() {
            return Err(ProductBuilderError::MissingRequiredField(
                "Missing product_address".to_string(),
            ));
        };

        if property_name.is_empty() {
            return Err(ProductBuilderError::MissingRequiredField(
                "Missing product_name".to_string(),
            ));
        };

        if data_type.is_empty() {
            return Err(ProductBuilderError::MissingRequiredField(
                "Missing data_type".to_string(),
            ));
        };

        if start_commit_num >= end_commit_num {
            return Err(ProductBuilderError::MissingRequiredField(
                "start_commit_number must be less than end_commit_num".to_string(),
            ));
        };

        if end_commit_num <= start_commit_num {
            return Err(ProductBuilderError::MissingRequiredField(
                "end_commit_number must be greater than start_commit_num".to_string(),
            ));
        };

        Ok(PropertyValue {
            product_id,
            product_address,
            property_name,
            data_type,
            bytes_value,
            boolean_value,
            number_value,
            string_value,
            enum_value,
            struct_values,
            lat_long_value,
            start_commit_num,
            end_commit_num,
            service_id,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ProductList {
    data: Vec<Product>,
    paging: Paging,
}

impl ProductList {
    pub fn new(data: Vec<Product>, paging: Paging) -> Self {
        Self { data, paging }
    }

    /// Returns the data for the product list
    pub fn data(&self) -> Vec<Product> {
        self.data.to_vec()
    }

    /// Returns the paging information for the product list
    pub fn paging(&self) -> &Paging {
        &self.paging
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatLongValue {
    pub latitude: i64,
    pub longitude: i64,
}

pub trait ProductStore {
    /// Adds a product to the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `product` - The product to be added
    fn add_product(&self, product: Product) -> Result<(), ProductStoreError>;

    /// Gets a product from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `product_id` - The ID of the product to be fetched
    ///  * `service_id` - The service ID to fetch the product for
    fn get_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Product>, ProductStoreError>;

    /// Gets a list of products from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `service_id` - The service ID to fetch the product for
    ///  * `offset` - The index of the first in storage to retrieve
    ///  * `limit` - The number of items to retrieve from the offset
    fn list_products(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<ProductList, ProductStoreError>;

    /// Updates a product in the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `product` - The updated product
    ///  * `service_id` - The service ID to fetch the product for
    ///  * `current_commit_num` - The current commit height
    fn update_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError>;

    /// Deletes a product from the underlying storage
    ///
    /// # Arguments
    ///
    ///  * `address` - The address of the record to be deleted
    ///  * `current_commit_num` - The current commit height
    fn delete_product(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError>;
}

impl<PS> ProductStore for Box<PS>
where
    PS: ProductStore + ?Sized,
{
    fn add_product(&self, product: Product) -> Result<(), ProductStoreError> {
        (**self).add_product(product)
    }

    fn get_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Product>, ProductStoreError> {
        (**self).get_product(product_id, service_id)
    }

    fn list_products(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<ProductList, ProductStoreError> {
        (**self).list_products(service_id, offset, limit)
    }

    fn update_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError> {
        (**self).update_product(product_id, service_id, current_commit_num)
    }

    fn delete_product(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), ProductStoreError> {
        (**self).delete_product(address, current_commit_num)
    }
}
