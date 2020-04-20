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
import { useLocalNodeState } from '../state/localNode';
import { useCircuitsState } from '../state/circuits';

import CircuitsTable from './circuitsTable/Table';

import './Content.scss';

const Content = () => {
  const [circuitState, circuitsDispatch] = useCircuitsState();

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
              filter: {
                filterTerm: event.target.value.toLowerCase()
              }
            });
          }}
        />
      </div>
      <CircuitsTable
        circuits={circuitState.filteredCircuits}
        dispatch={circuitsDispatch}
      />
    </div>
  );
};

export default Content;
