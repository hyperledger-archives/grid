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

import React from 'react';
import PropTypes from 'prop-types';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';

import './ProductCard.scss';

function ProductCard(props) {
  const { gtin, name, owner, imageURL } = props;

  return (
    <div className="product-card">
      <button type="button" className="product-card-edit-button">
        <FontAwesomeIcon className="icon" icon="pen-square" />
      </button>
      <div className="product-card-content">
        <div className="product-card-properties">
          <Property label="GTIN" value={gtin} />
          <Property label="Product Name" value={name} />
          <Property label="Owner" value={owner} />
        </div>
        {imageURL && (
          <img className="product-card-image" src={imageURL} alt={name} />
        )}
      </div>
    </div>
  );
}

ProductCard.propTypes = {
  gtin: PropTypes.string.isRequired,
  name: PropTypes.string.isRequired,
  owner: PropTypes.string.isRequired,
  imageURL: PropTypes.string
};

ProductCard.defaultProps = {
  imageURL: null
};

function Property(props) {
  const { label, value } = props;

  return (
    <div className="product-card-property">
      <div className="label">{label}</div>
      <div className="value">{value}</div>
    </div>
  );
}

Property.propTypes = {
  label: PropTypes.string.isRequired,
  value: PropTypes.string.isRequired
};

export default ProductCard;
