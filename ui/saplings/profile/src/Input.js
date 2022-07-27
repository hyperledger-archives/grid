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
import proptypes from 'prop-types';
import './Input.scss';

export function Input({ type, id, label, name, value, onChange, required }) {
  return (
    <div className="grid-input">
      <input
        type={type}
        id={id}
        aria-label={`${label} field`}
        placeholder={`${label}`}
        name={name}
        value={value}
        onChange={onChange}
        required={required}
      />
      <label htmlFor={id}>{label}</label>
    </div>
  );
}

Input.propTypes = {
  type: proptypes.string,
  id: proptypes.string,
  label: proptypes.string,
  name: proptypes.string,
  value: proptypes.any,
  onChange: proptypes.func.isRequired,
  required: proptypes.bool
};

Input.defaultProps = {
  type: 'text',
  id: undefined,
  label: undefined,
  name: undefined,
  value: null,
  required: false
};
