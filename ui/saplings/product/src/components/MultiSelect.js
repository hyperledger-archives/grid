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

import React, { useEffect, useState, useRef, useReducer } from 'react';
import PropTypes from 'prop-types';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import useOnClickOutside from '../hooks/on-click-outside';

import './MultiSelect.scss';

const reducer = (state, action) => {
  switch (action.type) {
    case 'add':
      return [...new Set([...state, ...action.payload])];
    case 'remove':
      return state.filter(item => item !== action.payload);
    case 'clear':
      return [];
    default:
      throw new Error(`Invalid action type: ${action.type}`);
  }
};

export const MultiSelect = ({ listItems, value, placeholder, onChange }) => {
  const [listOpen, setListOpen] = useState(false);
  const [headerText, setHeaderText] = useState(placeholder);
  const [selectedValues, dispatchSelectedValues] = useReducer(reducer, value);
  const caretUp = <FontAwesomeIcon className="icon" icon="caret-up" />;
  const caretDown = <FontAwesomeIcon className="icon" icon="caret-down" />;

  const checkSelected = item => {
    return selectedValues.includes(item.value);
  };

  const handleSelect = item => {
    if (checkSelected(item)) {
      dispatchSelectedValues({ type: 'remove', payload: item.value });
    } else {
      dispatchSelectedValues({ type: 'add', payload: [item.value] });
    }
  };

  useEffect(() => {
    onChange(selectedValues);
  }, [selectedValues, onChange]);

  useEffect(() => {
    if (selectedValues.length > 0) {
      setHeaderText(`${selectedValues.length} selected`);
    } else {
      setHeaderText(placeholder);
    }
  }, [selectedValues, placeholder]);

  const handleSelectAll = () => {
    if (selectedValues.length === listItems.length) {
      dispatchSelectedValues({ type: 'clear' });
    } else {
      dispatchSelectedValues({
        type: 'add',
        payload: listItems.map(item => item.value)
      });
    }
  };

  const ref = useRef();
  useOnClickOutside(ref, () => setListOpen(false));

  return (
    <div className="multi-select-wrapper" value={selectedValues}>
      <div
        className="multi-select-header"
        role="button"
        tabIndex="0"
        onClick={() => setListOpen(!listOpen)}
        onKeyPress={() => setListOpen(!listOpen)}
      >
        <div className="multi-select-header-text">{headerText}</div>
        {listOpen ? caretUp : caretDown}
      </div>
      {listOpen && (
        <ul className="multi-select-options-list">
          <MultiSelectOption
            label="Select all"
            onClick={handleSelectAll}
            onKeyPress={handleSelectAll}
          />
          {listItems.map(item => (
            <MultiSelectOption
              value={item.value}
              label={item.label}
              onClick={() => handleSelect(item)}
              onKeyPress={() => handleSelect(item)}
              selected={checkSelected(item)}
            />
          ))}
        </ul>
      )}
    </div>
  );
};

const MultiSelectOption = ({ value, label, onClick, onKeyPress, selected }) => {
  return (
    <div
      className="multi-select-option"
      role="button"
      value={value}
      tabIndex="0"
      onClick={onClick}
      onKeyPress={onKeyPress}
      selected={selected}
    >
      {label}
      {selected && <FontAwesomeIcon icon="check" />}
    </div>
  );
};

MultiSelect.propTypes = {
  listItems: PropTypes.array,
  value: PropTypes.array,
  placeholder: PropTypes.string,
  onChange: PropTypes.func.isRequired
};

MultiSelect.defaultProps = {
  listItems: [],
  value: [],
  placeholder: ''
};

MultiSelectOption.propTypes = {
  value: PropTypes.oneOfType([PropTypes.string, PropTypes.number]),
  label: PropTypes.string,
  onClick: PropTypes.func.isRequired,
  onKeyPress: PropTypes.func,
  selected: PropTypes.bool
};

MultiSelectOption.defaultProps = {
  value: undefined,
  label: '',
  onKeyPress: undefined,
  selected: false
};
