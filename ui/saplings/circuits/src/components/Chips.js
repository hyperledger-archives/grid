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
import { Node } from '../data/nodeRegistry';

import './Chips.scss';

export function Chips({ nodes, localNodeID, removeFn }) {
  return (
    <div className="chips">
      {nodes.map(node => {
        const isLocal = node.identity === localNodeID;
        return (
          <Chip
            key={node.identity}
            node={node}
            isLocal={isLocal}
            removeFn={() => removeFn(node)}
            deleteable={!isLocal}
          />
        );
      })}
    </div>
  );
}

function Chip({ node, isLocal, removeFn, deleteable }) {
  return (
    <div className="chip">
      <span className="node-field node-name">{node.identity}</span>
      {isLocal && <div className="node-local">Local</div>}
      {deleteable && (
        <FontAwesomeIcon icon="times" className="delete" onClick={removeFn} />
      )}
    </div>
  );
}

Chips.propTypes = {
  nodes: PropTypes.arrayOf(PropTypes.instanceOf(Node)).isRequired,
  localNodeID: PropTypes.string,
  removeFn: PropTypes.func
};

Chips.defaultProps = {
  localNodeID: '',
  removeFn: undefined
};

Chip.propTypes = {
  node: PropTypes.instanceOf(Node).isRequired,
  isLocal: PropTypes.bool,
  removeFn: PropTypes.func,
  deleteable: PropTypes.bool
};

Chip.defaultProps = {
  isLocal: false,
  removeFn: undefined,
  deleteable: false
};
