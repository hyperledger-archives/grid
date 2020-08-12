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

import Dropdown from 'react-dropdown';

import 'react-dropdown/style.css';
import './index.scss';

const InputWrapper = ({ label, error, children }) => {
  return (
    <div className="input-control input-wrapper">
      <div className="label">{label}</div>
      <div className="input-fields">{children}</div>
      <div className="form-error">{error}</div>
    </div>
  );
};

InputWrapper.propTypes = {
  label: PropTypes.oneOfType([PropTypes.string, PropTypes.element]),
  error: PropTypes.oneOfType([PropTypes.string, PropTypes.element]),
  children: PropTypes.oneOfType([
    PropTypes.element,
    PropTypes.arrayOf(PropTypes.element)
  ])
};

InputWrapper.defaultProps = {
  label: '',
  error: '',
  children: []
};

export const ListBoxSelect = ({
  name,
  label,
  error,
  value,
  onChange,
  options
}) => {
  let selectedOpt = options.find(opt => opt.value === value);
  if (!selectedOpt && options.length > 0) {
    [selectedOpt] = options;
  } else if (!selectedOpt) {
    selectedOpt = { value: '', content: '' };
  }

  return (
    <InputWrapper label={label} error={error}>
      <Dropdown
        className="listbox-dropdown"
        placeholderClassName="listbox-dropdown-placeholder"
        controlClassName="listbox-dropdown-control"
        menuClassName="listbox-dropdown-placeholder"
        placeholder={selectedOpt.content}
        options={options.map(opt => ({ value: opt.value, label: opt.content }))}
        onChange={({ value: evtValue }) => {
          const evt = {
            preventDefault: () => null,
            target: { name, value: evtValue }
          };
          onChange(evt);
        }}
      />
    </InputWrapper>
  );
};

ListBoxSelect.propTypes = {
  name: PropTypes.string.isRequired,
  value: PropTypes.string,
  label: PropTypes.string,
  error: PropTypes.string,
  onChange: PropTypes.func.isRequired,
  options: PropTypes.arrayOf(
    PropTypes.shape({
      value: PropTypes.string.isRequired,
      content: PropTypes.oneOfType([PropTypes.string, PropTypes.element])
        .isRequired
    })
  )
};

ListBoxSelect.defaultProps = {
  value: '',
  error: '',
  label: '',
  options: [{ value: '', content: '' }]
};

export const TextField = ({
  name,
  label,
  value,
  error,
  children,
  ...props
}) => (
  <InputWrapper label={label} error={error}>
    <div className="text-field-wrapper">
      {React.createElement('input', {
        ...props,
        type: 'text',
        name,
        value: value || ''
      })}
      {children}
    </div>
  </InputWrapper>
);

TextField.propTypes = {
  name: PropTypes.string,
  label: PropTypes.oneOfType([PropTypes.string, PropTypes.element]),
  value: PropTypes.string,
  error: PropTypes.oneOfType([PropTypes.string, PropTypes.element]),
  children: PropTypes.oneOfType([
    PropTypes.element,
    PropTypes.arrayOf(PropTypes.element)
  ])
};

TextField.defaultProps = {
  name: '',
  label: '',
  value: '',
  error: '',
  children: []
};

export const InputRow = ({ className, children, ...props }) =>
  React.createElement(
    'div',
    {
      className: `input-row ${className}`,
      ...props
    },
    children
  );

InputRow.propTypes = {
  className: PropTypes.string,
  children: PropTypes.oneOfType([
    PropTypes.element,
    PropTypes.arrayOf(PropTypes.element)
  ])
};

InputRow.defaultProps = {
  className: '',
  children: []
};

export const Button = ({ label, type, ...props }) => {
  // this circumvents a linting issue around type and props spreading
  const e = React.createElement;
  return e(
    'button',
    {
      type: type || 'button',
      ...props
    },
    <span>{label}</span>
  );
};

Button.propTypes = {
  label: PropTypes.oneOfType([PropTypes.string, PropTypes.element]).isRequired,
  type: PropTypes.string
};

Button.defaultProps = {
  type: null
};
