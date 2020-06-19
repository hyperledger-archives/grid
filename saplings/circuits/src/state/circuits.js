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

import { useReducer, useEffect, useState } from 'react';
import {
  listProposals,
  getProposal,
  listCircuits,
  getCircuit
} from '../api/splinter';

import { Circuit, ListCircuitsResponse } from '../data/circuits';

const filterCircuitsByTerm = (circuits, filterBy) => {
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

const sortCircuits = (circuits, action) => {
  const order = action.ascendingOrder ? -1 : 1;
  switch (action.sortBy) {
    case 'comments': {
      const sorted = circuits.sort((circuitA, circuitB) => {
        if (circuitA.comments === 'N/A' && circuitB.comments !== 'N/A') {
          return 1; // 'N/A should always be at the bottom'
        }
        if (circuitA.comments !== 'N/A' && circuitB.comments === 'N/A') {
          return -1; // 'N/A should always be at the bottom'
        }
        if (circuitA.comments < circuitB.comments) {
          return order;
        }
        if (circuitA.comments > circuitB.comments) {
          return -order;
        }
        return 0;
      });

      return sorted;
    }
    case 'circuitID': {
      const sorted = circuits.sort((circuitA, circuitB) => {
        if (circuitA.id < circuitB.id) {
          return order;
        }
        if (circuitA.id > circuitB.id) {
          return -order;
        }
        return 0;
      });

      return sorted;
    }
    case 'serviceCount': {
      const sorted = circuits.sort((circuitA, circuitB) => {
        if (circuitA.roster.length < circuitB.roster.length) {
          return order;
        }
        if (circuitA.roster.length > circuitB.roster.length) {
          return -order;
        }
        return 0;
      });

      return sorted;
    }
    case 'managementType': {
      const sorted = circuits.sort((circuitA, circuitB) => {
        if (circuitA.managementType < circuitB.managementType) {
          return order;
        }
        if (circuitA.managementType > circuitB.managementType) {
          return -order;
        }
        return 0;
      });

      return sorted;
    }
    default:
      return circuits;
  }
};

const filterCircuitsByStatus = (circuits, filterBy) => {
  const filteredCircuits = circuits.filter(circuit => {
    let include = false;
    if (filterBy.awaitingApproval) {
      include = include || circuit.awaitingApproval();
    }
    if (filterBy.actionRequired) {
      include = include || circuit.actionRequired(filterBy.nodeID);
    }
    if (!filterBy.awaitingApproval && !filterBy.actionRequired) {
      include = true;
    }
    return include;
  });
  return filteredCircuits;
};

const circuitsReducer = (state, action) => {
  switch (action.type) {
    case 'set': {
      const { circuits } = action;
      return { ...state, isSet: true, filteredCircuits: circuits, circuits };
    }
    case 'sort': {
      const sortedCircuits = sortCircuits(state.filteredCircuits, action.sort);
      return { ...state, filteredCircuits: sortedCircuits };
    }
    case 'filterByTerm': {
      const filteredByTerm = filterCircuitsByTerm(
        state.circuits,
        action.filter
      );
      let filteredCircuits = filteredByTerm;
      if (state.filteredByStatus.length > 0) {
        filteredCircuits = filteredByTerm.filter(
          circuit => state.filteredByStatus.indexOf(circuit) > -1
        );
      }
      return {
        ...state,
        filteredByTerm,
        filteredCircuits
      };
    }
    case 'filterByStatus': {
      const filteredByStatus = filterCircuitsByStatus(
        state.circuits,
        action.filter
      );

      let filteredCircuits = filteredByStatus;
      if (state.filteredByTerm.length > 0) {
        filteredCircuits = filteredByStatus.filter(
          circuit => state.filteredByTerm.indexOf(circuit) > -1
        );
      }

      return {
        ...state,
        filteredByStatus,
        filteredCircuits
      };
    }
    default:
      throw new Error(`unhandled action type: ${action.type}`);
  }
};

function useCircuitsState() {
  const [circuitState, circuitsDispatch] = useReducer(circuitsReducer, {
    isSet: false,
    circuits: [],
    filteredByTerm: [],
    filteredByStatus: [],
    filteredCircuits: []
  });

  useEffect(() => {
    const getCircuits = async () => {
      if (!circuitState.isSet) {
        try {
          const apiCircuits = await listCircuits();
          const apiProposals = await listProposals();

          const circuits = new ListCircuitsResponse(apiCircuits);
          const proposals = new ListCircuitsResponse(apiProposals);

          circuitsDispatch({
            type: 'set',
            circuits: circuits.data.concat(proposals.data)
          });
        } catch (e) {
          throw Error(`Error fetching circuits from the splinter daemon: ${e}`);
        }
      }
    };
    getCircuits();
  }, [circuitState]);

  return [circuitState, circuitsDispatch];
}

function useCircuitState(circuitId) {
  const [stateCircuitId, setCircuitId] = useState(circuitId);
  const [circuit, setCircuit] = useState(null);

  useEffect(() => {
    const loadCircuit = async () => {
      if (stateCircuitId) {
        let apiCircuit = null;
        try {
          apiCircuit = await getCircuit(stateCircuitId);
        } catch (circuitError) {
          try {
            apiCircuit = await getProposal(stateCircuitId);
          } catch (proposalError) {
            throw Error(
              `Unable to fetch ${stateCircuitId} from the splinter daemon`
            );
          }
        }

        setCircuit(new Circuit(apiCircuit));
      }
    };
    loadCircuit();
  }, [stateCircuitId]);

  return [circuit, setCircuitId];
}

export { useCircuitsState, useCircuitState };
