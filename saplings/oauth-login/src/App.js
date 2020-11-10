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
import { ToastProvider } from 'react-toast-notifications';
import './App.css';

import { OAuthLoginButton } from './components/OAuthLoginButton';

function App({ errorMessage }) {
  return (
    <div className="oauth-login-app">
      <ToastProvider>
        <OAuthLoginButton errorMessage={errorMessage} />
      </ToastProvider>
    </div>
  );
}

App.propTypes = {
  errorMessage: PropTypes.string
};

App.defaultProps = {
  errorMessage: null
};

export default App;
