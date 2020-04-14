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
import PropTypes from 'prop-types';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';

import { useServiceState } from '../state/service-context';
import ProductCard from './ProductCard';
import NotFound from './NotFound';
import Loading from './Loading';
import { getProperty } from '../data/property-parsing';
import { listProducts } from '../api/grid';
import './ProductsTable.scss';

function ProductsTable({ actions }) {
  const [products, setProducts] = useState([]);
  const { selectedService } = useServiceState();
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const getProducts = async () => {
      if (selectedService !== 'none') {
        setLoading(true);
        try {
          const productList = await listProducts(selectedService);
          setProducts(productList);
        } catch (e) {
          console.error(`Error listing products: ${e}`);
        }
      } else {
        setProducts([]);
      }
      setLoading(false);
    };

    getProducts();
  }, [selectedService]);

  const productCards = products.map(product => {
    return (
      <ProductCard
        key={product.product_id}
        gtin={product.product_id}
        name={getProperty('product_name', product.properties)}
        owner={product.orgName}
        imageURL={getProperty('image_url', product.properties)}
        editFn={actions.editProduct}
        properties={product.properties}
      />
    );
  });

  const getContent = () => {
    if (loading) {
      return <Loading />;
    }
    if (products.length === 0) {
      return <NotFound message="No Products" />;
    }
    return <div className="products-table">{productCards}</div>;
  };

  return (
    <div className="products-table-container">
      <div className="products-table-header">
        <h5 className="title">Products</h5>
        <hr />
      </div>
      {getContent()}
      <button
        className="fab add-product"
        type="button"
        onClick={actions.addProduct}
      >
        <FontAwesomeIcon icon="plus" />
      </button>
    </div>
  );
}

ProductsTable.propTypes = {
  actions: PropTypes.object.isRequired
};

export default ProductsTable;
