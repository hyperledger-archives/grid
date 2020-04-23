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

import { useReducer } from 'react';
import mockCircuits from '../mockData/mockCircuits';
import mockProposals from '../mockData/mockProposals';
import { processCircuits } from '../data/processCircuits';

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
      const filteredCircuits = filterCircuits(state.circuits, action.filter);
      return { ...state, filteredCircuits };
    }
    default:
      throw new Error(`unhandled action type: ${action.type}`);
  }
};

function useCircuitsState() {
  const circuits = processCircuits(mockCircuits.concat(mockProposals));

  const [circuitState, circuitsDispatch] = useReducer(circuitsReducer, {
    circuits,
    filteredCircuits: circuits
  });
  return [circuitState, circuitsDispatch];
}

export { useCircuitsState };
