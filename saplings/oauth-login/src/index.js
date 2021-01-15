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

import {
  registerConfigSapling,
  registerApp,
  hideCanopy,
  setUser
} from 'splinter-saplingjs';
import React from 'react';
import ReactDOM from 'react-dom';
import './index.css';
import App from './App';

registerConfigSapling('login', () => {
  if (
    window.$CANOPY.redirectedFrom &&
    window.$CANOPY.redirectedFrom.includes('access_token')
  ) {
    // Extract user token information
    const accessTokenRegex = /[?|&]access_token=([^&#]*)|&|$/;
    const token = window.$CANOPY.redirectedFrom.match(accessTokenRegex)[1];
    // Extract user information
    const userIdRegex = /[?|&]user_id=([^&#]*)|&|$/;
    let userId = window.$CANOPY.redirectedFrom.match(userIdRegex)[1];
    if (!userId) {
      userId = 'OAuthUser';
    }
    const displayNameRegex = /[?|&]display_name=([^&#]*)|&|$/;
    let displayName = window.$CANOPY.redirectedFrom.match(displayNameRegex)[1];
    if (!displayName) {
      displayName = 'OAuthUser';
    }
    // Set Canopy user
    setUser({
      token,
      userId,
      displayName: decodeURI(displayName)
    });
    // Set the OAuth logout route
    sessionStorage.setItem('LOGOUT', '/oauth/logout');

    window.$CANOPY.redirectedFrom = window.location.href;
    window.location.replace('/');
  } else if (window.location.pathname === '/login') {
    let errorMessage = null;
    if (
      window.$CANOPY.redirectedFrom &&
      window.$CANOPY.redirectedFrom.includes('error')
    ) {
      const errorMessageRegex = /\?error=([^&#]*)|&|$/;
      const error = window.$CANOPY.redirectedFrom.match(errorMessageRegex)[1];
      errorMessage = error;
    }
    hideCanopy();
    registerApp(domNode => {
      ReactDOM.render(<App errorMessage={errorMessage} />, domNode);
    });
  }
});
