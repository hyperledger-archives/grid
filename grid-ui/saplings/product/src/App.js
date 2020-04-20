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

import React, { useState } from 'react';
import { BrowserRouter as Router, Switch, Route } from 'react-router-dom';
import { library } from '@fortawesome/fontawesome-svg-core';
import {
  faCaretUp,
  faCaretDown,
  faCheck,
  faPenSquare,
  faChevronLeft,
  faPlus,
  faTimes,
  faSpinner,
  faTrashAlt
} from '@fortawesome/free-solid-svg-icons';
import { ToastProvider } from 'react-toast-notifications';

import { ServiceProvider } from './state/service-context';
import FilterBar from './components/FilterBar';
import ProductsTable from './components/ProductsTable';
import ProductInfo from './components/ProductInfo';
import { AddProductForm } from './components/AddProductForm';
import { EditProductForm } from './components/EditProductForm';
import { DeleteProductForm } from './components/DeleteProductForm';
import './App.scss';

library.add(
  faCaretUp,
  faCaretDown,
  faCheck,
  faPenSquare,
  faChevronLeft,
  faPlus,
  faTimes,
  faSpinner,
  faTrashAlt
);

function App() {
  const initialFormState = {
    formName: '',
    params: {}
  };
  const [activeForm, setActiveForm] = useState(initialFormState);

  function addProduct() {
    setActiveForm({
      formName: 'add-product',
      params: {}
    });
  }

  function editProduct(productID, owner, service, properties) {
    setActiveForm({
      formName: 'edit-product',
      params: {
        properties,
        owner,
        productID,
        service
      }
    });
  }

  function deleteProduct(productID, owner, service) {
    setActiveForm({
      formName: 'delete-product',
      params: {
        productID,
        owner,
        service
      }
    });
  }

  function openForm(form) {
    const adata = { ...form.params } || {};
    switch (form.formName) {
      case 'add-product':
        return (
          <AddProductForm closeFn={() => setActiveForm(initialFormState)} />
        );
      case 'edit-product':
        return (
          <EditProductForm
            closeFn={() => setActiveForm(initialFormState)}
            properties={adata.properties}
            owner={adata.owner}
            productID={adata.productID}
            service={adata.service}
          />
        );
      case 'delete-product':
        return (
          <DeleteProductForm
            closeFn={() => setActiveForm(initialFormState)}
            owner={adata.owner}
            productID={adata.productID}
            service={adata.service}
          />
        );
      default:
    }
    return null;
  }

  return (
    <ServiceProvider>
      <ToastProvider>
        <div id="product-sapling" className="product-app">
          <FilterBar />
          <Router>
            <Switch>
              <Route exact path="/product">
                <ProductsTable
                  actions={{ addProduct, editProduct, deleteProduct }}
                />
              </Route>
              <Route path="/product/products/:id">
                <ProductInfo />
              </Route>
            </Switch>
          </Router>
          {activeForm && openForm(activeForm)}
        </div>
      </ToastProvider>
    </ServiceProvider>
  );
}

export default App;
