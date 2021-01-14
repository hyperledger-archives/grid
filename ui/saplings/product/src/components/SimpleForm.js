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
import PropTypes from 'prop-types';

import './SimpleForm.scss';

export function SimpleForm({
  formName,
  handleSubmit,
  style,
  disabled,
  error,
  children
}) {
  return (
    <div className="simpleForm" style={style}>
      <div className="info">
        <h5>{formName}</h5>
      </div>
      <div className="formWrapper">
        <form>{children}</form>
        <div className="actions">
          <button
            type="button"
            onClick={handleSubmit}
            className="submit"
            disabled={disabled || error}
          >
            Submit
          </button>
        </div>
      </div>
    </div>
  );
}

SimpleForm.propTypes = {
  formName: PropTypes.string,
  handleSubmit: PropTypes.func.isRequired,
  style: PropTypes.object,
  disabled: PropTypes.bool,
  error: PropTypes.bool,
  children: PropTypes.node
};

SimpleForm.defaultProps = {
  formName: '',
  style: undefined,
  disabled: false,
  error: false,
  children: undefined
};
