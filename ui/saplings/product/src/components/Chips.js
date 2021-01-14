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
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';

import './Chips.scss';

export function Chips({ children }) {
  return <div className="chips">{children}</div>;
}

export function Chip({ label, removeFn, data, deleteable }) {
  return (
    <div className="chip-group">
      <div className="chip">
        <span className="label">{label}</span>
        {deleteable && (
          <FontAwesomeIcon icon="times" className="delete" onClick={removeFn} />
        )}
      </div>
      <div className="chip-data">{data}</div>
    </div>
  );
}

Chips.propTypes = {
  children: PropTypes.node
};

Chips.defaultProps = {
  children: undefined
};

Chip.propTypes = {
  label: PropTypes.string,
  removeFn: PropTypes.func,
  data: PropTypes.oneOfType([PropTypes.string, PropTypes.object]),
  deleteable: PropTypes.bool
};

Chip.defaultProps = {
  label: '',
  removeFn: undefined,
  data: undefined,
  deleteable: false
};
