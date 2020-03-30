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
import PropTypes from 'prop-types';

const ServiceStateContext = React.createContext();
const ServiceDispatchContext = React.createContext();

const serviceReducer = (state, action) => {
  switch (action.type) {
    case 'select': {
      return { ...state, selectedService: action.payload.serviceID };
    }
    case 'selectNone': {
      return { ...state, selectedService: 'none' };
    }
    case 'setServices': {
      return { ...state, services: action.payload.services };
    }
    default:
      throw new Error(`unhandled action type: ${action.type}`);
  }
};

function ServiceProvider({ children }) {
  const [state, dispatch] = React.useReducer(serviceReducer, {
    selectedService: 'none',
    services: []
  });

  return (
    <ServiceStateContext.Provider value={state}>
      <ServiceDispatchContext.Provider value={dispatch}>
        {children}
      </ServiceDispatchContext.Provider>
    </ServiceStateContext.Provider>
  );
}

ServiceProvider.propTypes = {
  children: PropTypes.node.isRequired
};

function useServiceState() {
  const context = React.useContext(ServiceStateContext);
  if (context === undefined) {
    throw new Error('useServiceState must be used within a ServiceProvider');
  }
  return context;
}

function useServiceDispatch() {
  const context = React.useContext(ServiceDispatchContext);
  if (context === undefined) {
    throw new Error('useServiceDispatch must be used within a ServiceProvider');
  }
  return context;
}

export { ServiceProvider, useServiceState, useServiceDispatch };
