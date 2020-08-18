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
import { Node } from '../data/nodeRegistry';

import './NodeCard.scss';

const NodeCard = ({ node, dispatcher, isLocal, isSelected, isSelectable }) => {
  const allowSelection = isSelectable && !isLocal && !isSelected;
  let localLabel = null;
  if (isLocal) {
    localLabel = <div className="node-local">Local</div>;
  }
  let handler = () => {};
  if (allowSelection) {
    handler = () => dispatcher(node);
  }

  let metadataRow = null;
  const metadata = Object.entries(node.metadata);
  if (metadata.length > 0) {
    metadataRow = (
      <div className="metadata col-span-4">
        {metadata.map(([key, value]) => (
          <div key={key} className="metadata-chip">{`${key}: ${value}`}</div>
        ))}
      </div>
    );
  }

  return (
    <div
      className={`node-card${allowSelection ? ' selectable' : ''}`}
      role="checkbox"
      aria-checked={isSelected}
      onClick={handler}
      onKeyUp={() => {}}
      tabIndex="0"
    >
      <div className="node-description">
        <div className="field-wrapper">
          <div className="field-header">Name</div>
          <div className="field-value">{node.displayName}</div>
        </div>
        <div className="field-wrapper">
          <div className="field-header">ID</div>
          <div className="field-value">{node.identity}</div>
        </div>
        <div className="field-wrapper">
          <div className="field-header">Endpoints</div>
          <div className="endpoints">
            {node.endpoints.map(endpoint => {
              return (
                <div key={endpoint} className="field-value">
                  {endpoint}
                </div>
              );
            })}
          </div>
        </div>
        <div className="node-labels">{localLabel}</div>
        {metadataRow}
      </div>
    </div>
  );
};

NodeCard.propTypes = {
  node: PropTypes.instanceOf(Node).isRequired,
  dispatcher: PropTypes.func,
  isLocal: PropTypes.bool,
  isSelected: PropTypes.bool,
  isSelectable: PropTypes.bool
};

NodeCard.defaultProps = {
  dispatcher: () => {},
  isLocal: false,
  isSelected: false,
  isSelectable: true
};

export default NodeCard;
