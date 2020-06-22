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

const filterCircuitsByTerm = filterBy => {
  if (filterBy.filterTerm.length > 0) {
    return circuit => {
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
          service => service.serviceType.indexOf(filterBy.filterTerm) > -1
        ).length > 0
      ) {
        return true;
      }
      return false;
    };
  }

  return null;
};

const filterCircuitsByStatus = filterBy => {
  return circuit => {
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
  };
};

const sortCircuits = ({ field, ascendingOrder }) => {
  const order = ascendingOrder ? -1 : 1;
  switch (field) {
    case 'comments': {
      return (circuitA, circuitB) => {
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
      };
    }
    case 'circuitID': {
      return (circuitA, circuitB) => {
        if (circuitA.id < circuitB.id) {
          return order;
        }
        if (circuitA.id > circuitB.id) {
          return -order;
        }
        return 0;
      };
    }
    case 'serviceCount': {
      return (circuitA, circuitB) => {
        if (
          circuitA.numUniqueServiceTypes() < circuitB.numUniqueServiceTypes()
        ) {
          return order;
        }
        if (
          circuitA.numUniqueServiceTypes() > circuitB.numUniqueServiceTypes()
        ) {
          return -order;
        }
        return 0;
      };
    }
    case 'managementType': {
      return (circuitA, circuitB) => {
        if (circuitA.managementType < circuitB.managementType) {
          return order;
        }
        if (circuitA.managementType > circuitB.managementType) {
          return -order;
        }
        return 0;
      };
    }
    default:
      return null;
  }
};

const applyStateFns = intermediateState => {
  const {
    circuitSortFn,
    termFilterFn,
    statusFilterFn,
    circuits
  } = intermediateState;

  let filteredCircuits = circuits.filter(circuit => {
    if (termFilterFn && !termFilterFn(circuit)) {
      return false;
    }
    if (statusFilterFn && !statusFilterFn(circuit)) {
      return false;
    }
    return true;
  });

  if (circuitSortFn) {
    filteredCircuits = filteredCircuits.sort(circuitSortFn);
  }

  return {
    ...intermediateState,
    filteredCircuits
  };
};

const circuitsReducer = (state, action) => {
  switch (action.type) {
    case 'set': {
      const { circuits } = action;
      const intermediateState = {
        ...state,
        isSet: true,
        circuits
      };
      return applyStateFns(intermediateState);
    }
    case 'sort': {
      const circuitSortFn = sortCircuits(action.sort);
      if (circuitSortFn) {
        return {
          ...state,
          circuitSortFn,
          filteredCircuits: state.filteredCircuits.sort(circuitSortFn)
        };
      }

      return {
        ...state,
        circuitSortFn
      };
    }
    case 'filterByTerm': {
      const termFilterFn = filterCircuitsByTerm(action.filter);
      const intermediateState = {
        ...state,
        termFilterFn
      };

      return applyStateFns(intermediateState);
    }
    case 'filterByStatus': {
      const statusFilterFn = filterCircuitsByStatus(action.filter);
      const intermediateState = {
        ...state,
        statusFilterFn
      };

      return applyStateFns(intermediateState);
    }
    default:
      throw new Error(`unhandled action type: ${action.type}`);
  }
};

function useCircuitsState() {
  const [circuitState, circuitsDispatch] = useReducer(circuitsReducer, {
    isSet: false,
    circuits: [],
    termFilterFn: null,
    statusFilterFn: null,
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
    const intervalId = setInterval(() => loadCircuit(), 10000);

    // call it initially.
    loadCircuit();

    return function cleanup() {
      clearInterval(intervalId);
    };
  }, [stateCircuitId]);

  return [circuit, setCircuitId];
}

export { useCircuitsState, useCircuitState };
