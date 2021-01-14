/**
 * Copyright 2018-2021 Cargill Incorporated
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

import React from 'react';
import { Link } from 'react-router-dom';
import PropTypes from 'prop-types';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';

import ProductProperty from './ProductProperty';
import './ProductCard.scss';

function ProductCard(props) {
  const {
    gtin,
    name,
    service,
    owner,
    imageURL,
    editFn,
    deleteFn,
    properties
  } = props;

  return (
    <div className="product-card">
      <button
        type="button"
        className="product-card-delete-button"
        onClick={() => deleteFn(gtin)}
      >
        <FontAwesomeIcon className="icon" icon="trash-alt" />
      </button>
      <button
        type="button"
        className="product-card-edit-button"
        onClick={() => editFn(gtin, owner, service, properties)}
      >
        <FontAwesomeIcon className="icon" icon="pen-square" />
      </button>
      <Link className="link" to={`/product/products/${gtin}`}>
        <div className="product-card-content">
          <div className="product-card-properties">
            <ProductProperty className="nowrap" label="GTIN" value={gtin} />
            <ProductProperty className="nowrap" label="Product" value={name} />
            <ProductProperty className="nowrap" label="Owner" value={owner} />
          </div>
          {imageURL && (
            <img className="product-card-image" src={imageURL} alt={name} />
          )}
        </div>
      </Link>
    </div>
  );
}

ProductCard.propTypes = {
  gtin: PropTypes.string.isRequired,
  name: PropTypes.string.isRequired,
  owner: PropTypes.string.isRequired,
  service: PropTypes.string.isRequired,
  imageURL: PropTypes.string,
  editFn: PropTypes.func.isRequired,
  deleteFn: PropTypes.func.isRequired,
  properties: PropTypes.array.isRequired
};

ProductCard.defaultProps = {
  imageURL: null
};

export default ProductCard;
