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

import { getProperty } from '../data/property-parsing';

const ProductStateContext = React.createContext();
const ProductDispatchContext = React.createContext();

const defaultState = {
  unfilteredProducts: [],
  products: [],
  filters: {
    gtin: '',
    brandName: '',
    productDescription: '',
    gpc: '',
    netContent: '',
    targetMarket: 0
  }
};

const filterProducts = (products, filters) => {
  let filteredProducts = products;

  if (filters.gtin) {
    filteredProducts = filteredProducts.filter(product => {
      return product.product_id.includes(filters.gtin);
    });
  }
  if (filters.brandName) {
    filteredProducts = filteredProducts.filter(product => {
      const brandName = getProperty('brand_name', product.properties);
      return brandName.includes(filters.brandName);
    });
  }
  if (filters.productDescription) {
    filteredProducts = filteredProducts.filter(product => {
      const description = getProperty(
        'product_description',
        product.properties
      );
      return description.includes(filters.productDescription);
    });
  }
  if (filters.gpc) {
    filteredProducts = filteredProducts.filter(product => {
      const gpc = getProperty('gpc', product.properties);
      return gpc.includes(filters.gpc);
    });
  }
  if (filters.netContent) {
    filteredProducts = filteredProducts.filter(product => {
      const netContent = getProperty('net_content', product.properties);
      return netContent.includes(filters.netContent);
    });
  }
  if (filters.targetMarket) {
    filteredProducts = filteredProducts.filter(product => {
      const targetMarket = getProperty('target_market', product.properties);
      return targetMarket === parseInt(filters.targetMarket, 10);
    });
  }
  return filteredProducts;
};

const productReducer = (state, action) => {
  switch (action.type) {
    case 'set': {
      const updatedState = {
        ...state,
        unfilteredProducts: action.payload.products,
        products: filterProducts(action.payload.products, state.filters)
      };
      return updatedState;
    }
    case 'filter': {
      const filteredProducts = filterProducts(
        state.unfilteredProducts,
        action.payload.filters
      );
      const updatedState = {
        ...state,
        products: filteredProducts,
        filters: action.payload.filters
      };
      return updatedState;
    }
    case 'reset': {
      const updatedState = {
        ...state,
        products: state.unfilteredProducts,
        filters: defaultState.filters
      };
      return updatedState;
    }
    default:
      throw new Error(`unhandled action type: ${action.type}`);
  }
};

function ProductProvider({ children }) {
  const [state, dispatch] = React.useReducer(productReducer, defaultState);
  return (
    <ProductStateContext.Provider value={state}>
      <ProductDispatchContext.Provider value={dispatch}>
        {children}
      </ProductDispatchContext.Provider>
    </ProductStateContext.Provider>
  );
}

ProductProvider.propTypes = {
  children: PropTypes.node.isRequired
};

function useProductState() {
  const context = React.useContext(ProductStateContext);
  if (context === undefined) {
    throw new Error('useProductState must be used within a ProductProvider');
  }
  return context;
}

function useProductDispatch() {
  const context = React.useContext(ProductDispatchContext);
  if (context === undefined) {
    throw new Error('useProductDispatch must be used within a ProductProvider');
  }
  return context;
}

export { ProductProvider, useProductState, useProductDispatch };
