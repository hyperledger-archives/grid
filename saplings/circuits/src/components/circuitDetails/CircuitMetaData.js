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

import PropTypes from 'prop-types';
import React from 'react';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faQuestionCircle } from '@fortawesome/free-solid-svg-icons';

import { Circuit } from '../../data/circuits';
import './CircuitMetaData.scss';

const MetadataRows = ({ metadata }) => {
  let metadataDetails;
  if (typeof metadata === 'object') {
    metadataDetails = (
      <table>
        <tbody>
          {Object.entries(metadata)
            .filter(([key]) => {
              return Object.prototype.hasOwnProperty.call(metadata, key);
            })
            .map(([key, value]) => {
              return (
                <tr key={key}>
                  <td>{key}</td>
                  <td>{value}</td>
                </tr>
              );
            })}
        </tbody>
      </table>
    );
  } else {
    metadataDetails = (
      <div className={metadata ? 'binary-metadata' : 'no-metadata'}>
        {metadata ? 'Binary Data' : 'None'}
      </div>
    );
  }

  return (
    <div className="metadata">
      <div className="metadata-title">Metadata</div>
      {metadataDetails}
    </div>
  );
};

MetadataRows.propTypes = {
  metadata: PropTypes.oneOfType([PropTypes.string, PropTypes.object]).isRequired
};

const CircuitMetaData = ({ circuit }) => {
  return (
    <div className="circuit-details">
      <div className="circuit-details-header">
        Circuit Details
        <span>
          <FontAwesomeIcon icon={faQuestionCircle} />
        </span>
      </div>
      <div className="comments">
        <div className="comments-title">Comments</div>
        <div className={!circuit.comments ? 'no-comments' : ''}>
          {circuit.comments ? circuit.comments.toString() : 'None'}
        </div>
      </div>
      <MetadataRows metadata={circuit.applicationMetadata} />
    </div>
  );
};

CircuitMetaData.propTypes = {
  circuit: PropTypes.instanceOf(Circuit).isRequired
};

export default CircuitMetaData;
