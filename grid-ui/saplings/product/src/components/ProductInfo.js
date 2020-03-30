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
import { useParams, Link } from 'react-router-dom';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';

import { useServiceState } from '../state/service-context';
import { getProperty } from '../data/property-parsing';
import ProductProperty from './ProductProperty';
import { fetchProduct } from '../api/grid';
import './ProductInfo.scss';

function ProductInfo() {
  const { id } = useParams();
  const [product, setProduct] = useState({ properties: [] });
  const { selectedService } = useServiceState();

  useEffect(() => {
    const getProduct = async () => {
      if (selectedService !== 'none') {
        try {
          const productResponse = await fetchProduct(selectedService, id);
          setProduct(productResponse);
        } catch (e) {
          console.error(`Error fetching product: ${e}`);
          setProduct({ properties: [] });
        }
      }
    };

    getProduct();
  }, [selectedService, id]);

  const imageURL = getProperty('image_url', product.properties);

  return (
    <div className="product-info-container">
      {imageURL && (
        <img src={imageURL} alt={product.id} className="product-image" />
      )}
      <ProductOverview
        gtin={getProperty('gtin', product.properties) || 'Unknown'}
        productName={
          getProperty('product_name', product.properties) || 'Unknown'
        }
        owner={product.owner || 'Unknown'}
      />
      <ProductProperties propertiesList={product.properties} />
    </div>
  );
}

function ProductOverview(props) {
  const { gtin, productName, owner } = props;

  return (
    <div className="product-overview-container">
      <Link className="back-link" to="/product">
        <FontAwesomeIcon icon="chevron-left" />
        <span className="back-link-text">Back</span>
      </Link>
      <ProductProperty className="large light" label="GTIN" value={gtin} />
      <ProductProperty
        className="large light"
        label="Product Name"
        value={productName}
      />
      <ProductProperty className="large light" label="Owner" value={owner} />
    </div>
  );
}

ProductOverview.propTypes = {
  gtin: PropTypes.string.isRequired,
  productName: PropTypes.string.isRequired,
  owner: PropTypes.string.isRequired
};

function ProductProperties(props) {
  const { propertiesList } = props;

  const primaryProperties = [
    { name: 'brand_name', data_type: 'STRING', label: 'Brand Name' },
    {
      name: 'product_description',
      data_type: 'STRING',
      label: 'Product Description'
    },
    { name: 'gpc', label: 'GPC' },
    { name: 'net_content', label: 'Net Content' },
    { name: 'target_market', label: 'Target Market' }
  ];

  const productProperties = primaryProperties.map(property => {
    const propertyValue = getProperty(property.name, propertiesList);

    if (propertyValue) {
      return (
        <ProductProperty
          className="large"
          label={property.label}
          value={propertyValue}
        />
      );
    }
    return <></>;
  });

  return (
    <div className="product-properties-container">
      <div className="product-properties-header">
        <h5 className="title">Product Info</h5>
        <hr />
      </div>
      <div className="product-properties-list">{productProperties}</div>
      <button type="button" className="full-info-button">
        VIEW FULL PRODUCT INFO
      </button>
    </div>
  );
}

ProductProperties.propTypes = {
  propertiesList: PropTypes.arrayOf(PropTypes.object).isRequired
};

export default ProductInfo;
