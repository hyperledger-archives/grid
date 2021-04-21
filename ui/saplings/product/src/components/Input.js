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
import classNames from 'classnames';
import Icon from '@material-ui/core/Icon'
import './Input.scss';

export function Input({
  type,
  accept,
  className,
  id,
  label,
  name,
  value,
  pattern,
  onChange,
  required,
  multiple,
  icon,
  children
}) {
  return (
    <div className={classNames('grid-input', (type === 'select' && 'select'), (icon && 'icon'), className)}>
      {type === 'select' && (
        <>
          {icon &&
            <Icon className="icon">{icon}</Icon>
          }
          <select
            id={id}
            aria-label={`${label} select`}
            name={name}
            onChange={onChange}
            required={required}
            multiple={multiple}
            value={value}
          >
            {children}
          </select>
        </>
      )}
      {type === 'boolean' && (
        <>
          <select
            id={id}
            aria-label={`${label} select`}
            name={name}
            onChange={onChange}
            required={required}
            multiple={multiple}
            value={value}
          >
            <option value="" default>
              (none)
            </option>
            <option value>True</option>
            <option value={false}>False</option>
          </select>
        </>
      )}
      {type !== 'select' && type !== 'boolean' && (
        <>
          {icon &&
            <Icon className="icon">{icon}</Icon>
          }
          <input
            type={type}
            id={id}
            accept={accept}
            aria-label={`${label} field`}
            placeholder={label}
            name={name}
            value={value}
            pattern={pattern}
            onChange={onChange}
            required={required}
          />
        </>
      )}
    </div>
  );
}

Input.propTypes = {
  type: PropTypes.oneOf(['text', 'password', 'file', 'select', 'number']),
  accept: PropTypes.string,
  className: PropTypes.string,
  id: PropTypes.string,
  label: PropTypes.string,
  name: PropTypes.string,
  value: PropTypes.oneOfType([PropTypes.string, PropTypes.number]),
  pattern: PropTypes.string,
  onChange: PropTypes.func.isRequired,
  required: PropTypes.bool,
  multiple: PropTypes.bool,
  icon: PropTypes.string,
  children: PropTypes.node
};

Input.defaultProps = {
  type: 'text',
  accept: undefined,
  className: undefined,
  id: undefined,
  label: undefined,
  name: undefined,
  value: null,
  pattern: undefined,
  required: false,
  multiple: undefined,
  icon: undefined,
  children: undefined
};
