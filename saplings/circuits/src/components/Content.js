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

import React, { useReducer } from 'react';
import { useLocalNodeState } from '../state/localNode';
import mockCircuits from '../mockData/mockCircuits';
import mockProposals from '../mockData/mockProposals';
import { processCircuits } from '../data/processCircuits';

import './Content.scss';

const circuitsReducer = (state, action) => {
  switch (action.type) {
    case 'sort': {
      const sortedCircuits = action.sortCircuits(
        state.filteredCircuits,
        action.sort
      );
      return { ...state, filteredCircuits: sortedCircuits };
    }
    case 'filter': {
      const filteredCircuits = action.filterCircuits(
        state.circuits,
        action.filter
      );
      return { ...state, filteredCircuits };
    }
    default:
      throw new Error(`unhandled action type: ${action.type}`);
  }
};

const filterCircuits = (circuits, filterBy) => {
  if (filterBy.filterTerm.length === 0) {
    return circuits;
  }
  const filteredCircuits = circuits.filter(circuit => {
    if (circuit.id.toLowerCase().indexOf(filterBy.filterTerm) > -1) {
      return true;
    }
    if (
      circuit.managementType.toLowerCase().indexOf(filterBy.filterTerm) > -1
    ) {
      return true;
    }
    if (circuit.comments.toLowerCase().indexOf(filterBy.filterTerm) > -1) {
      return true;
    }
    if (
      circuit.members.filter(
        member => member.toLowerCase().indexOf(filterBy.filterTerm) > -1
      ).length > 0
    ) {
      return true;
    }
    if (
      circuit.roster.filter(
        service => service.service_type.indexOf(filterBy.filterTerm) > -1
      ).length > 0
    ) {
      return true;
    }
    return false;
  });

  return filteredCircuits;
};

const Content = () => {
  const circuits = processCircuits(mockCircuits.concat(mockProposals));

  const [circuitState, circuitsDispatch] = useReducer(circuitsReducer, {
    circuits,
    filteredCircuits: circuits
  });
  const nodeID = useLocalNodeState();
  const totalCircuits = circuitState.circuits.length;
  let actionRequired = 0;
  if (nodeID !== 'unknown') {
    actionRequired = circuitState.circuits.filter(circuit =>
      circuit.actionRequired(nodeID)
    ).length;
  }

  return (
    <div className="main-content">
      <div className="midContent">
        <div className="circuit-stats">
          <div className="stat total-circuits">
            <span className="stat-count circuits-count">{totalCircuits}</span>
            Circuits
          </div>
          <div className="stat action-required">
            <span className="stat-count action-required-count">
              {actionRequired}
            </span>
            {actionRequired > 1 ? 'Actions required' : 'Action required'}
          </div>
        </div>
        <input
          className="filterTable"
          type="text"
          placeholder="Filter"
          onKeyUp={event => {
            circuitsDispatch({
              type: 'filter',
              filterCircuits,
              filter: {
                filterTerm: event.target.value.toLowerCase()
              }
            });
          }}
        />
      </div>
    </div>
  );
};

export default Content;
