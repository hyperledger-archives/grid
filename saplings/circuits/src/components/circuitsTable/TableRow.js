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
import { useHistory } from 'react-router-dom';

import { useLocalNodeState } from '../../state/localNode';
import { Circuit } from '../../data/circuits';
import { useNodeRegistryState } from '../../state/nodeRegistry';

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
  const history = useHistory();
  const maxCountShow = 3;

  const nodes = useNodeRegistryState().filter(
    node => !!circuit.members.find(id => id === node.identity)
  );

  const serviceTypeCount = () => {
    const servicesCount = Object.entries(circuit.listServiceTypesCount());
    return servicesCount.map(([serviceType, count], index) => {
      if (index < maxCountShow) {
        return `${serviceType} (${count}) \n`;
      }
      if (index === maxCountShow) {
        return `and ${servicesCount.length - maxCountShow} more...`;
      }
      return '';
    });
  };

  const members = () => {
    return nodes.map((node, index) => {
      if (index < maxCountShow) {
        return `${node.displayName} \n`;
      }
      if (index === maxCountShow) {
        return `and ${nodes.length - maxCountShow} more...`;
      }
      return '';
    });
  };

  return (
    <tr
      className="table-row"
      onClick={() => {
        history.push(`/circuits/${circuit.id}`);
      }}
    >
      <td className={circuit.displayName === '' ? 'text-grey' : ''}>
        <div>{circuit.displayName}</div>
      </td>
      <td className="text-highlight">{circuit.id}</td>
      <td>{members()}</td>
      <td>{serviceTypeCount()}</td>
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
