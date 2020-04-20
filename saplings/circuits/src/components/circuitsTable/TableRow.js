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
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';

import { useLocalNodeState } from '../../state/localNode';
import { Circuit } from '../../data/processCircuits';

const proposalStatus = (circuit, nodeID) => {
  const exclamation = (
    <span className="status-icon">
      <FontAwesomeIcon icon="exclamation" />
    </span>
  );
  const awaiting = (
    <span className="status awaiting-approval">Awaiting approval</span>
  );
  return (
    <div className="proposal-status">
      {circuit.actionRequired(nodeID) ? (
        <span className="status action-required">
          Action required
          {exclamation}
        </span>
      ) : (
        ''
      )}
      {awaiting}
    </div>
  );
};

const TableRow = ({ circuit }) => {
  const nodeID = useLocalNodeState();
  return (
    <tr className="table-row">
      <td className="text-highlight">{circuit.id}</td>
      <td>
        {
          new Set(
            circuit.roster.map(service => {
              return service.service_type;
            })
          ).size
        }
      </td>
      <td>{circuit.managementType}</td>
      <td className={circuit.comments === 'N/A' ? 'text-grey' : ''}>
        <div className="circuit-comment">{circuit.comments}</div>
      </td>
      <td>
        {circuit.awaitingApproval() ? proposalStatus(circuit, nodeID) : ''}
      </td>
    </tr>
  );
};

TableRow.propTypes = {
  circuit: PropTypes.instanceOf(Circuit).isRequired
};

export default TableRow;
