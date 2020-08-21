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
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { faKey } from '@fortawesome/free-solid-svg-icons'

import './KeyTable.scss';

const KeyTable = ({ keys, activeKey, onActivate, onEdit }) => {

  let rows;
  if (keys.length === 0) {
    rows = (
      <tr>
        <td colSpan="3" className="keys-empty">You have no keys</td>
      </tr>
    );
  } else  {
    rows = keys.map(key => {
      const isActive = activeKey === key.public_key;
      return (
        <tr key={key.public_key}>
          <td>
            <FontAwesomeIcon
              icon={faKey}
              pull="left"
              size="3x"
              className={isActive ? 'active' : ''}
            />
            <div className="key-details">
              <span className="key-name">{key.display_name}</span>
              <span>{key.public_key}</span>
            </div>
          </td>
          <td>
            <button
              className="link-btn"
              onClick={e => {
                e.preventDefault();
                onEdit(key);
              }}
            >
              Edit
            </button>
            {!isActive &&
              <button
                className="link-btn"
                onClick={e => {
                  e.preventDefault();
                  onActivate(key);
                }}
              >
                Activate
              </button>}
          </td>
        </tr>
      )
    });
  }

  return (
    <table className="keys-table">
      <thead>
        <tr>
          <th>Key</th>
          <th className="keys-ops" aria-label="Operations" />
        </tr>
      </thead>
      <tbody>{rows}</tbody>
    </table>
  );
};

KeyTable.propTypes = {
  keys: PropTypes.arrayOf(PropTypes.object).isRequired,
  activeKey: PropTypes.string,
  onActivate: PropTypes.func.isRequired,
  onEdit: PropTypes.func.isRequired,
};

KeyTable.defaultProps = {
  activeKey: null
};

export default KeyTable;
