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

import React, { useState, useEffect, useReducer } from 'react';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { MultiStepForm, Step } from './MultiStepForm';
import { useNodeRegistryState } from '../../state/nodeRegistry';
import { useLocalNodeState } from '../../state/localNode';

import NodeCard from '../NodeCard';
import { OverlayModal } from '../OverlayModal';
import { NewNodeForm } from './NewNodeForm';

import { Chips } from '../Chips';

import './ProposeCircuitForm.scss';

const filterNodes = (state, input) => {
  const lowerInput = input.toLowerCase();
  const filteredNodes = state.availableNodes.filter(node => {
    if (node.identity.toLowerCase().indexOf(lowerInput) > -1) {
      if (state.showSelectedOnly) {
        const isSelected =
          state.selectedNodes.filter(
            selectedNode => node.identity === selectedNode.identity
          ).length > 0;
        if (!isSelected) {
          return false;
        }
      }
      return true;
    }
    if (node.displayName.toLowerCase().indexOf(lowerInput) > -1) {
      return true;
    }
    return false;
  });

  return filteredNodes;
};

const nodesReducer = (state, action) => {
  const minNodeCountError =
    'At least two nodes must be part of a circuit. Please select a node.';

  switch (action.type) {
    case 'filter': {
      const nodes = filterNodes(state, action.input);
      const filteredNodes = {
        nodes,
        filteredBy: action.input
      };
      return { ...state, filteredNodes };
    }
    case 'addLocalNode': {
      const localNode = action.node;
      state.selectedNodes.push(localNode);
      state.filteredNodes.nodes.sort((node1, node2) => {
        if (node1.identity === localNode.identity) {
          return -1;
        }
        if (node2.identity === localNode.identity) {
          return 1;
        }
        return 0;
      });
      return { ...state };
    }
    case 'showSelectedOnly': {
      const newState = state;
      newState.showSelectedOnly = true;
      const nodes = filterNodes(state, state.filteredNodes.filteredBy);
      const filteredNodes = {
        nodes,
        filteredBy: state.filteredNodes.filteredBy
      };
      return { ...newState, filteredNodes };
    }
    case 'showAllNodes': {
      const newState = state;
      newState.showSelectedOnly = false;
      const nodes = filterNodes(state, state.filteredNodes.filteredBy);
      const filteredNodes = {
        nodes,
        filteredBy: state.filteredNodes.filteredBy
      };
      return { ...newState, filteredNodes };
    }
    case 'toggleSelect': {
      const { node } = action;
      let alreadySelected = false;

      const selectedNodes = state.selectedNodes.filter(selectedNode => {
        if (node.identity === selectedNode.identity) {
          alreadySelected = true;
          return false;
        }
        return true;
      });

      if (!alreadySelected) {
        selectedNodes.push(node);
      }

      const nodes = filterNodes(state, state.filteredNodes.filteredBy);
      const filteredNodes = {
        nodes,
        filteredBy: state.filteredNodes.filteredBy
      };

      let { error } = state;
      if (selectedNodes.length >= 2) {
        error = '';
      } else {
        error = minNodeCountError;
      }

      return { ...state, selectedNodes, filteredNodes, error };
    }
    case 'removeSelect': {
      const { node } = action;
      const selectedNodes = state.selectedNodes.filter(
        item => item.identity !== node.identity
      );

      const nodes = filterNodes(state, state.filteredNodes.filteredBy);
      const filteredNodes = {
        nodes,
        filteredBy: state.filteredNodes.filteredBy
      };

      let error = '';
      if (selectedNodes.length < 2) {
        error = minNodeCountError;
      }
      return { ...state, selectedNodes, filteredNodes, error };
    }
    case 'addNode': {
      const { node } = action;
      state.availableNodes.push(node);
      const nodes = filterNodes(state, state.filteredNodes.filteredBy);
      const filteredNodes = {
        nodes,
        filteredBy: state.filteredNodes.filteredBy
      };

      return { ...state, filteredNodes };
    }
    case 'set': {
      const { nodes } = action;
      return {
        ...state,
        nodes,
        availableNodes: nodes,
        filteredNodes: {
          nodes,
          filteredBy: ''
        }
      };
    }
    default:
      throw new Error(`unhandled action type: ${action.type}`);
  }
};

export function ProposeCircuitForm() {
  const allNodes = useNodeRegistryState();
  const localNodeID = useLocalNodeState();
  const [modalActive, setModalActive] = useState(false);
  const [localNode] = allNodes.filter(node => node.identity === localNodeID);
  const [nodesState, nodesDispatcher] = useReducer(nodesReducer, {
    selectedNodes: [],
    availableNodes: [],
    showSelectedOnly: false,
    filteredNodes: {
      nodes: [],
      filteredBy: ''
    },
    error: ''
  });

  const nodesAreValid = () => {
    return nodesState.selectedNodes.length >= 2;
  };

  useEffect(() => {
    if (allNodes) {
      nodesDispatcher({
        type: 'set',
        nodes: allNodes
      });
    }
  }, [allNodes]);

  useEffect(() => {
    if (localNode) {
      nodesDispatcher({
        type: 'addLocalNode',
        node: localNode
      });
    }
  }, [localNode]);

  return (
    <MultiStepForm
      formName="Propose Circuit"
      handleSubmit={() => {}}
      disabled={!nodesAreValid()}
    >
      <Step step={1} label="Add nodes">
        <div className="step-header">
          <div className="step-title">Add nodes</div>
          <div className="help-text">
            Select the nodes that are part of the circuit
          </div>
        </div>
        <div className="node-registry-wrapper">
          <div className="selected-nodes-wrapper">
            <div className="selected-nodes-header">
              <div className="title">Selected nodes</div>
            </div>
            <div className="form-error">{nodesState.error}</div>
            <div className="selected-nodes">
              <Chips
                nodes={nodesState.selectedNodes}
                localNodeID={localNodeID}
                removeFn={node => {
                  nodesDispatcher({ type: 'removeSelect', node });
                }}
              />
            </div>
          </div>
          <div className="available-nodes">
            <div className="available-nodes-header">
              <div className="select-filter">
                Show:
                <button
                  type="button"
                  className={
                    nodesState.showSelectedOnly
                      ? 'no-style-btn'
                      : 'no-style-btn selected'
                  }
                  onClick={() => nodesDispatcher({ type: 'showAllNodes' })}
                >
                  {`All nodes (${nodesState.availableNodes.length})`}
                </button>
                <span className="filter-separator">|</span>
                <button
                  type="button"
                  className={
                    nodesState.showSelectedOnly
                      ? 'no-style-btn selected'
                      : 'no-style-btn'
                  }
                  onClick={() => nodesDispatcher({ type: 'showSelectedOnly' })}
                >
                  {`Selected nodes (${nodesState.selectedNodes.length})`}
                </button>
              </div>
              <input
                type="text"
                placeholder="Filter"
                className="search-nodes-input"
                onKeyUp={event => {
                  nodesDispatcher({
                    type: 'filter',
                    input: event.target.value
                  });
                }}
              />
            </div>
            <ul>
              {nodesState.filteredNodes.nodes.map(node => {
                const local = node.identity === localNodeID;
                const selected =
                  nodesState.selectedNodes.filter(selectedNode => {
                    return node.identity === selectedNode.identity;
                  }).length > 0;
                return (
                  <li className="node-item">
                    <NodeCard
                      node={node}
                      dispatcher={targetNode => {
                        nodesDispatcher({
                          type: 'toggleSelect',
                          node: targetNode
                        });
                      }}
                      isLocal={local}
                      isSelected={selected}
                    />
                  </li>
                );
              })}
              <button
                className="form-button new-node-button"
                type="button"
                onClick={() => {
                  setModalActive(true);
                }}
              >
                <FontAwesomeIcon icon="plus" />
              </button>
              <div className="button-label">Add new node to registry</div>
            </ul>
          </div>
        </div>
        <OverlayModal open={modalActive}>
          <NewNodeForm
            closeFn={() => setModalActive(false)}
            successCallback={node => {
              nodesDispatcher({
                type: 'addNode',
                node
              });
              nodesDispatcher({
                type: 'toggleSelect',
                node
              });
            }}
          />
        </OverlayModal>
      </Step>
      <Step step={2} label="Add services">
        <input type="text" placeholder="test" />
      </Step>
      <Step step={3} label="Add circuit details">
        <input type="text" placeholder="test" />
      </Step>
      <Step step={4} label="Add metadata">
        <input type="text" placeholder="test" />
      </Step>
      <Step step={5} label="Review and submit">
        <input type="text" placeholder="test" />
      </Step>
    </MultiStepForm>
  );
}
