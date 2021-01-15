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
import React, { useState } from 'react';
import proptypes from 'prop-types';
import './ActionList.scss';

export function ActionList({ children }) {
  const [open, setOpen] = useState(0);
  return (
    <div className="action-list">
      <button
        className={`flat action-button${open ? ' open' : ''}`}
        onClick={() => setOpen(!open)}
      >
        Actions
        <div className="hamburger">
          <div className="top" />
          <div className="middle" />
          <div className="bottom" />
        </div>
      </button>
      <div className={`actions${open ? ' open' : ''}`}>{children}</div>
    </div>
  );
}

ActionList.propTypes = {
  children: proptypes.node.isRequired
};
