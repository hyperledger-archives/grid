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
import PropTypes from 'prop-types';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { SimpleForm } from './SimpleForm';
import { Input } from './Input';

import './DeleteProductForm.scss';
import './forms.scss';

export function DeleteProductForm({ closeFn, gtin }) {
  const [confirmGtin, setConfirmGtin] = useState('');
  const [valid, setValid] = useState(false);
  const [errors, setErrors] = useState([]);

  const gtinValidator = '\\b\\d{8}(?:\\d{4,6})?\\b';

  const handleChange = e => {
    const { value, validity } = e.target;
    setConfirmGtin(value);
    setValid(value === '' && validity.valid ? false : validity.valid);
    setErrors(value === gtin ? [] : ['GTIN values do not match']);
  };

  const reset = () => {
    setConfirmGtin('');
    setValid(false);
    setErrors([]);
  };

  const submit = () => {
    reset();
    closeFn();
  };

  return (
    <div id="delete-product-form" className="modalForm">
      <FontAwesomeIcon icon="times" className="close" onClick={closeFn} />
      <div className="content">
        <SimpleForm
          formName="Delete Product"
          handleSubmit={submit}
          disabled={!valid || !!errors.length}
        >
          <div id="delete-warning">
            <span>Are you sure you want to delete product:</span>
            <span id="gtin-label">
              GTIN: <span id="gtin">{gtin}</span>
            </span>
            <span>
              You cannot undo this action. Please re-enter the GTIN below to
              confirm.
            </span>
            <Input
              type="text"
              label="GTIN"
              name="gtin"
              value={confirmGtin}
              pattern={gtinValidator}
              onChange={handleChange}
            />
          </div>
          {!!errors.length && (
            <div className="error-messages">
              {errors.map(error => (
                <span>{error}</span>
              ))}
            </div>
          )}
        </SimpleForm>
      </div>
    </div>
  );
}

DeleteProductForm.propTypes = {
  closeFn: PropTypes.func.isRequired,
  gtin: PropTypes.string.isRequired
};
