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

import '../Content.scss';
import '../MainHeader.scss';
import './CircuitDetails.scss';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import {
  faQuestionCircle,
  faArrowLeft,
  faExclamationTriangle
} from '@fortawesome/free-solid-svg-icons';
import PropTypes from 'prop-types';
import { useParams, Link } from 'react-router-dom';
import React from 'react';

import { Circuit } from '../../data/circuits';
import { useCircuitState } from '../../state/circuits';
import { getNodeRegistry } from '../../api/splinter';
import ServiceDetails from './ServiceDetails';

const CircuitDetails = () => {
  const { circuitId } = useParams();
  const [circuit] = useCircuitState(circuitId);

  if (!circuit) {
    return <div />;
  }

  let requiresAction = '';
  if (circuit.awaitingApproval()) {
    requiresAction = (
      <div className="requires-action">
        <FontAwesomeIcon icon={faExclamationTriangle} />
        <span>Awaiting Approval</span>
      </div>
    );
  }

  return (
    <div>
      <div className="main-header">
        <div className="circuit-header">
          <div className="back-button">
            <Link to="/circuits">
              <FontAwesomeIcon icon={faArrowLeft} />
              <span>Circuits</span>
            </Link>
          </div>
          {requiresAction}
          <div className="circuit-title">
            <h4>{`Circuit ${circuitId}`}</h4>
            <div className="managementType">
              {circuit.managementType}
              <span>
                <FontAwesomeIcon icon={faQuestionCircle} />
              </span>
            </div>
          </div>
        </div>
      </div>
      <div className="main-content">
        <div className="midContent">
          <div className="circuit-stats">
            <div className="stat total-circuits">
              <span className="stat-count circuits-count">
                {circuit.members.length}
              </span>
              Nodes
            </div>
            <div className="stat action-required">
              <span className="stat-count action-required-count">
                {circuit.roster.length}
              </span>
              Services
            </div>
          </div>
        </div>

        <NodesTable circuit={circuit} nodes={circuit.members} />
      </div>
    </div>
  );
};

const contains = (list, val) => !!list.find(v => v === val);

const NodesTable = ({ circuit, nodes }) => {
  const [fullNodes, setNodes] = React.useState(null);
  const [toggledRow, setToggledRow] = React.useState(null);

  React.useEffect(() => {
    const fetchNodes = async () => {
      try {
        const apiNodes = await getNodeRegistry();
        const filteredNodes = apiNodes.filter(node =>
          contains(nodes, node.identity)
        );
        setNodes(filteredNodes);
      } catch (e) {
        throw Error(`Unable to fetch nodes from the node registry: ${e}`);
      }
    };

    fetchNodes();
  }, [nodes]);

  if (fullNodes === null) {
    return <div />;
  }

  let rows = [
    <tr>
      <td colSpan="5" className="no-nodes-msg">
        No Nodes found for this circuit
      </td>
    </tr>
  ];

  if (fullNodes.length > 0) {
    rows = fullNodes.map((node, idx) => {
      let endpoints = 'N/A';
      if (node.endpoints.length > 0) {
        endpoints = node.endpoints.reduce((acc, endpoint) => {
          if (acc.length > 0) {
            acc.push(<br />);
          }
          acc.push(endpoint);
          return acc;
        }, []);
      }

      let detailsRow = '';
      if (toggledRow === idx) {
        detailsRow = (
          <tr className="service-details-row">
            <td colSpan="5">
              <ServiceDetails
                services={circuit.roster.filter(service =>
                  contains(service.allowedNodes, node.identity)
                )}
              />
            </td>
          </tr>
        );
      }

      return [
        <tr
          className="table-row"
          onClick={() => {
            if (toggledRow === idx) {
              setToggledRow(null);
            } else {
              setToggledRow(idx);
            }
          }}
        >
          <td>{node.identity}</td>
          <td>{node.displayName}</td>
          <td>
            {node.metadata.company || node.metadata.organization || 'N/A'}
          </td>
          <td>{endpoints}</td>
          <td>
            <NodeStatus circuit={circuit} nodeId={node.identity} />
          </td>
        </tr>,
        detailsRow
      ];
    });
  }

  return (
    <div className="table-container">
      <table className="nodes-table">
        <tr className="table-header">
          <th>ID</th>
          <th>Alias</th>
          <th>Company</th>
          <th>Endpoints</th>
          <th>Status</th>
        </tr>
        {rows}
      </table>
    </div>
  );
};

NodesTable.propTypes = {
  circuit: PropTypes.arrayOf(Circuit).isRequired,
  nodes: PropTypes.arrayOf(Object).isRequired
};

const NodeStatus = ({ circuit, nodeId }) => {
  if (circuit.actionRequired(nodeId)) {
    return <span className="status awaiting-approval">Awaiting approval</span>;
  }

  return '';
};

export default CircuitDetails;
