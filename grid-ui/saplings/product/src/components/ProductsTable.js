/**
 * Copyright 2018-2020 Cargill Incorporated
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
 */

import React, { useState, useEffect } from 'react';
import { useServiceState } from '../state/service-context';

import './ProductsTable.scss';
import ProductCard from './ProductCard';
import mockProducts from '../test/mock-products';
import { getProperty } from '../data/property-parsing';

function ProductsTable() {
  const [products, setProducts] = useState(mockProducts);
  const { selectedService } = useServiceState();

  useEffect(() => {
    if (selectedService === 'all') {
      setProducts(mockProducts);
    } else {
      setProducts(
        mockProducts.filter(product => product.service_id === selectedService)
      );
    }
  }, [selectedService]);

  const productCards = products.map(product => {
    return (
      <ProductCard
        key={product.product_id}
        productID={product.product_id}
        gtin={getProperty('gtin', product.properties)}
        name={getProperty('product_name', product.properties)}
        owner={product.owner}
        imageURL={getProperty('image_url', product.properties)}
      />
    );
  });

  return (
    <div className="products-table-container">
      <div className="products-table-header">
        <h5 className="title">Products</h5>
        <hr />
      </div>
      <div className="products-table">{productCards}</div>
    </div>
  );
}

export default ProductsTable;
