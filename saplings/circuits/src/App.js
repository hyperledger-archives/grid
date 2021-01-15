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

import React from 'react';
import './App.css';
import {
  faPlus,
  faCaretUp,
  faCaretDown,
  faExclamation,
  faFilter,
  faSort,
  faExclamationCircle,
  faBusinessTime,
  faCheck,
  faTimes,
  faPlusCircle,
  faMinusCircle,
  faPen,
  faTrash,
  faSyncAlt
} from '@fortawesome/free-solid-svg-icons';
import { BrowserRouter as Router, Switch, Route } from 'react-router-dom';
import { library } from '@fortawesome/fontawesome-svg-core';
import { ToastProvider } from 'react-toast-notifications';

import MainHeader from './components/MainHeader';
import { LocalNodeProvider } from './state/localNode';
import Content from './components/Content';
import { ProposeCircuitForm } from './components/forms/propose_circuit';
import CircuitDetails from './components/circuitDetails/CircuitDetails';

library.add(
  faPlus,
  faCaretUp,
  faCaretDown,
  faExclamation,
  faFilter,
  faSort,
  faExclamationCircle,
  faBusinessTime,
  faCheck,
  faTimes,
  faPlusCircle,
  faMinusCircle,
  faPen,
  faTrash,
  faSyncAlt
);

function App() {
  return (
    <div className="circuits-app">
      <LocalNodeProvider>
        <ToastProvider>
          <Router>
            <Switch>
              <Route exact path="/circuits">
                <MainHeader />
                <Content />
              </Route>
              <Route path="/circuits/propose">
                <ProposeCircuitForm />
              </Route>
              <Route path="/circuits/:circuitId">
                <CircuitDetails />
              </Route>
            </Switch>
          </Router>
        </ToastProvider>
      </LocalNodeProvider>
    </div>
  );
}

export default App;
