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

import { getUser } from 'splinter-saplingjs';
import { useReducer, useEffect, useState } from 'react';
import {
  listProposals,
  getProposal,
  listCircuits,
  getCircuit
} from '../api/splinter';

import { Circuit, ListCircuitsResponse } from '../data/circuits';

const REFREST_INTERVAL = 10000; // ten seconds;

const filterCircuitsByTerm = filterBy => {
  if (filterBy.filterTerm.length > 0) {
    return circuit => {
      if (circuit.displayName.toLowerCase().indexOf(filterBy.filterTerm) > -1) {
        return true;
      }
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
    case 'displayName': {
      return (circuitA, circuitB) => {
        if (!circuitA.displayName && circuitB.displayName) {
          return 1; // 'Empty display names should always be at the bottom'
        }
        if (circuitA.displayName && !circuitB.displayName) {
          return -1; // 'Empty display names should always be at the bottom'
        }
        if (circuitA.displayName < circuitB.displayName) {
          return order;
        }
        if (circuitA.displayName > circuitB.displayName) {
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
    case 'memberCount': {
      return (circuitA, circuitB) => {
        if (circuitA.members.length < circuitB.members.length) {
          return order;
        }
        if (circuitA.members.length > circuitB.members.length) {
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
  const user = getUser();
  const [circuitState, circuitsDispatch] = useReducer(circuitsReducer, {
    circuits: [],
    termFilterFn: null,
    statusFilterFn: null,
    filteredCircuits: []
  });

  useEffect(() => {
    const getCircuits = async () => {
      if (user) {
        try {
          const apiCircuits = await listCircuits(user.token);
          const apiProposals = await listProposals(user.token);

          const circuits = new ListCircuitsResponse(apiCircuits);
          const proposals = new ListCircuitsResponse(apiProposals);

          circuitsDispatch({
            type: 'set',
            circuits: circuits.data.concat(proposals.data)
          });
        } catch (error) {
          throw Error(
            `Error fetching circuits from the splinter daemon: ${error.json.message}`
          );
        }
      }
    };
    const intervalId = setInterval(() => getCircuits(), REFREST_INTERVAL);
    getCircuits();
    return function cleanup() {
      clearInterval(intervalId);
    };
  }, [user]);

  return [circuitState, circuitsDispatch];
}

function useCircuitState(circuitId) {
  const user = getUser();
  const [stateCircuitId, setCircuitId] = useState(circuitId);
  const [circuitState, setCircuit] = useState({
    circuit: null,
    error: ''
  });

  useEffect(() => {
    const loadCircuit = async () => {
      if (user && stateCircuitId && !circuitState.error) {
        let apiCircuit = null;
        try {
          apiCircuit = await getCircuit(stateCircuitId, user.token);
        } catch (circuitError) {
          if (circuitError.code === '401') {
            setCircuit({
              circuit: null,
              error: `User is not authorized to access this resource: ${circuitError.json.message}`
            });
          } else {
            try {
              apiCircuit = await getProposal(stateCircuitId, user.token);
            } catch (proposalError) {
              setCircuit({
                circuit: null,
                error: `Unable to fetch circuit from splinterd: ${proposalError.json.message}`
              });
            }
          }
        }
        if (apiCircuit && !circuitState.error) {
          setCircuit({
            circuit: new Circuit(apiCircuit),
            error: ''
          });
        }
      }
    };
    const intervalId = setInterval(() => loadCircuit(), REFREST_INTERVAL);

    // call it initially.
    loadCircuit();

    return function cleanup() {
      clearInterval(intervalId);
    };
  }, [stateCircuitId, user]);

  return [circuitState, setCircuitId];
}

export { useCircuitsState, useCircuitState };
