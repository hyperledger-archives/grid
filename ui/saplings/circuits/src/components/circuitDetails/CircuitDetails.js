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

import '../Content.scss';
import '../MainHeader.scss';
import './CircuitDetails.scss';

import { getUser } from 'splinter-saplingjs';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import {
  faQuestionCircle,
  faArrowLeft,
  faExclamationTriangle,
  faCaretDown,
  faCaretRight
} from '@fortawesome/free-solid-svg-icons';
import PropTypes from 'prop-types';
import { useParams, Link } from 'react-router-dom';
import React from 'react';

import { Circuit } from '../../data/circuits';
import { useCircuitState } from '../../state/circuits';
import { getNodeRegistry } from '../../api/splinter';
import ServiceDetails from './ServiceDetails';
import CircuitMetaData from './CircuitMetaData';
import VoteButton from './VoteButton';
import { useLocalNodeState } from '../../state/localNode';
import { OverlayModal } from '../OverlayModal';
import VoteOnProposalForm from '../forms/VoteOnProposalForm';
import { Node } from '../../data/nodeRegistry';

const Label = ({ labelClass, children }) => {
  return <div className={`label ${labelClass}`}>{children}</div>;
};

Label.propTypes = {
  labelClass: PropTypes.string.isRequired,
  children: PropTypes.node
};

Label.defaultProps = {
  children: undefined
};

const CircuitDetails = () => {
  const user = getUser();
  const { circuitId } = useParams();
  const [circuitState] = useCircuitState(circuitId);
  const localNodeID = useLocalNodeState();
  const [modalActive, setModalActive] = React.useState(false);
  const [nodes, setNodes] = React.useState([]);
  React.useEffect(() => {
    const fetchNodes = async () => {
      if (user && circuitState.circuit) {
        try {
          const apiNodes = await getNodeRegistry();
          const filteredNodes = apiNodes.filter(
            node =>
              !!circuitState.circuit.members.find(id => id === node.identity)
          );
          setNodes(filteredNodes);
        } catch (e) {
          throw Error(`Unable to fetch nodes from the node registry: ${e}`);
        }
      }
    };
    if (circuitState.circuit) {
      fetchNodes();
    }
  }, [circuitState, user]);

  const getCircuitTitle = () => {
    if (circuitState.circuit.displayName) {
      return (
        <div className="circuit-title">
          <h4>{`${circuitState.circuit.displayName}`}</h4>
          <h6>{`Circuit ${circuitId}`}</h6>
          <div className="managementType">
            {circuitState.circuit.managementType}
            <span>
              <FontAwesomeIcon icon={faQuestionCircle} />
            </span>
          </div>
        </div>
      );
    }

    return (
      <div className="circuit-title">
        <h4>{`Circuit ${circuitId}`}</h4>
        <div className="managementType">
          {circuitState.circuit.managementType}
          <span>
            <FontAwesomeIcon icon={faQuestionCircle} />
          </span>
        </div>
      </div>
    );
  };

  let requiresAction = '';
  if (circuitState.circuit && circuitState.circuit.awaitingApproval()) {
    requiresAction = (
      <div className="requires-action">
        <FontAwesomeIcon icon={faExclamationTriangle} />
        <span>Awaiting Approval</span>
      </div>
    );
  }

  return (
    <div>
      {circuitState.circuit && (
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
              <div className="mid-header-wrapper">
                {getCircuitTitle()}
                {circuitState.circuit.actionRequired(localNodeID) && (
                  <VoteButton
                    onClickFn={() => {
                      setModalActive(true);
                    }}
                  />
                )}
              </div>
            </div>
          </div>
          <div className="detail-content">
            <div className="main-content">
              <div className="midContent">
                <div className="circuit-stats">
                  <div className="stat total-circuits">
                    <span className="stat-count circuits-count">
                      {circuitState.circuit.members.length}
                    </span>
                    Nodes
                  </div>
                  <div className="stat action-required">
                    <span className="stat-count action-required-count">
                      {circuitState.circuit.roster.length}
                    </span>
                    Services
                  </div>
                </div>
              </div>

              <NodesTable circuit={circuitState.circuit} nodes={nodes} />
            </div>
            <CircuitMetaData circuit={circuitState.circuit} />
          </div>
          <OverlayModal open={modalActive}>
            <VoteOnProposalForm
              proposal={circuitState.circuit}
              nodes={nodes}
              closeFn={() => {
                setModalActive(false);
              }}
            />
          </OverlayModal>
        </div>
      )}
      {circuitState.error && (
        <div className="error-message">
          <h4>
            <FontAwesomeIcon icon={faExclamationTriangle} />
            {circuitState.error}
          </h4>
        </div>
      )}
    </div>
  );
};

const contains = (list, val) => !!list.find(v => v === val);

const NodesTable = ({ circuit, nodes }) => {
  const [toggledRow, setToggledRow] = React.useState(null);

  if (nodes === null) {
    return <div />;
  }

  nodes.sort((nodeA, nodeB) => {
    const nodeIdA = nodeA.identity.toLowerCase();
    const nodeIdB = nodeB.identity.toLowerCase();
    if (nodeIdA < nodeIdB) {
      return -1;
    }
    if (nodeIdA > nodeIdB) {
      return 1;
    }
    return 0;
  });

  let rows = [
    <tr key="no-nodes">
      <td colSpan="5" className="no-nodes-msg">
        No Nodes found for this circuit
      </td>
    </tr>
  ];

  if (nodes.length > 0) {
    rows = nodes.flatMap((node, idx) => {
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

      let toggledIcon = <FontAwesomeIcon icon={faCaretRight} />;
      if (toggledRow === idx) {
        toggledIcon = <FontAwesomeIcon icon={faCaretDown} />;
      }
      const rowsForNode = [
        <tr
          key={node.identity}
          className="table-row"
          onClick={() => {
            if (toggledRow === idx) {
              setToggledRow(null);
            } else {
              setToggledRow(idx);
            }
          }}
        >
          <td>
            <span className="toggle">{toggledIcon}</span>
            {node.identity}
          </td>
          <td>{node.displayName}</td>
          <td>
            {node.metadata.company || node.metadata.organization || 'N/A'}
          </td>
          <td>{endpoints}</td>
          <td>
            <NodeStatus circuit={circuit} nodeId={node.identity} />
          </td>
        </tr>
      ];
      if (toggledRow === idx) {
        rowsForNode.push(
          <tr key={`service-${node.identity}`} className="service-details-row">
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

      return rowsForNode;
    });
  }

  return (
    <div className="nodes-content">
      <table className="nodes-table">
        <thead>
          <tr className="table-header">
            <th>ID</th>
            <th>Alias</th>
            <th>Company</th>
            <th>Endpoints</th>
            <th>&nbsp;</th>
          </tr>
        </thead>
        <tbody>{rows}</tbody>
      </table>
    </div>
  );
};

NodesTable.propTypes = {
  circuit: PropTypes.instanceOf(Circuit).isRequired,
  nodes: PropTypes.arrayOf(PropTypes.instanceOf(Node)).isRequired
};

const NodeStatus = ({ circuit, nodeId }) => {
  const localNodeID = useLocalNodeState();
  let localLabel = '';
  if (localNodeID === nodeId) {
    localLabel = <Label labelClass="local">Local</Label>;
  }

  let pendingVoteLabel = '';
  if (circuit.actionRequired(nodeId)) {
    pendingVoteLabel = <Label labelClass="pending-vote">Pending Vote</Label>;
  }

  let requesterLabel = '';
  if (circuit.proposal.requesterNodeID === nodeId) {
    requesterLabel = <Label labelClass="requester">Requester</Label>;
  }

  return (
    <div className="labels">
      {localLabel}
      {pendingVoteLabel}
      {requesterLabel}
    </div>
  );
};

NodeStatus.propTypes = {
  circuit: PropTypes.instanceOf(Circuit).isRequired,
  nodeId: PropTypes.string.isRequired
};

export default CircuitDetails;
