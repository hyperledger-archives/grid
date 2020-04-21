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
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import PropTypes from 'prop-types';

import { useProductState, useProductDispatch } from '../state/product-context';
import { Input } from './Input';
import { getCountryByISO } from '../data/iso';
import { getProperty } from '../data/property-parsing';
import './SearchModal.scss';

function SearchModal(props) {
  const { closeFn } = props;
  const productDispatch = useProductDispatch();
  const productState = useProductState();
  const [filters, setFilters] = useState(productState.filters);

  const applyFilters = () => {
    productDispatch({
      type: 'filter',
      payload: { filters }
    });
    closeFn();
  };

  const clearFilters = () => {
    productDispatch({
      type: 'reset'
    });
  };

  useEffect(() => {
    setFilters(productState.filters);
  }, [productState]);

  const handleGTINChange = event => {
    setFilters({ ...filters, gtin: event.target.value });
  };

  const handleBrandNameChange = event => {
    setFilters({ ...filters, brandName: event.target.value });
  };

  const handleDescriptionChange = event => {
    setFilters({ ...filters, productDescription: event.target.value });
  };

  const handleGPCChange = event => {
    setFilters({ ...filters, gpc: event.target.value });
  };

  const handleNetContentChange = event => {
    setFilters({ ...filters, netContent: event.target.value });
  };

  const handleTargetMarketChange = event => {
    setFilters({ ...filters, targetMarket: event.target.value });
  };

  const getTargetMarketListItems = () => {
    const codes = [
      ...new Set(
        productState.unfilteredProducts.map(product =>
          getProperty('target_market', product.properties)
        )
      )
    ];

    return codes.map(code => (
      <option value={code}>{getCountryByISO(code)}</option>
    ));
  };

  return (
    <div className="modalForm">
      <FontAwesomeIcon icon="times" className="close" onClick={closeFn} />
      <div className="content search-container">
        <div className="header">
          <h5 className="title">Search</h5>
          <hr />
        </div>
        <div className="body">
          <div className="fields">
            <div className="input-wrapper">
              <Input
                type="text"
                id="gtin"
                label="GTIN"
                value={filters.gtin}
                onChange={handleGTINChange}
              />
            </div>
            <div className="input-wrapper">
              <Input
                type="text"
                id="brandName"
                label="Brand Name"
                value={filters.brandName}
                onChange={handleBrandNameChange}
              />
            </div>
            <div className="input-wrapper">
              <Input
                type="text"
                id="productDescription"
                label="Product Description"
                value={filters.productDescription}
                onChange={handleDescriptionChange}
              />
            </div>
            <div className="input-wrapper">
              <Input
                type="number"
                id="gpc"
                label="GPC"
                value={filters.gpc}
                onChange={handleGPCChange}
              />
            </div>
            <div className="input-wrapper">
              <Input
                type="text"
                id="netContent"
                label="Net Content"
                value={filters.netContent}
                onChange={handleNetContentChange}
              />
            </div>
            <div className="input-wrapper">
              <Input
                type="select"
                id="targetMarket"
                label="Target market"
                value={filters.targetMarket}
                onChange={handleTargetMarketChange}
              >
                <option value="" default>
                  All
                </option>
                {getTargetMarketListItems()}
              </Input>
            </div>
          </div>
        </div>
        <div className="actions">
          <button type="button" onClick={clearFilters}>
            Reset
          </button>
          <button type="button" className="submit" onClick={applyFilters}>
            Search
          </button>
        </div>
      </div>
    </div>
  );
}

SearchModal.propTypes = {
  closeFn: PropTypes.func.isRequired
};

export default SearchModal;
