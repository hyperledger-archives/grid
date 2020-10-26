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
import './OAuthLoginButton.scss';

export function OAuthLoginButton() {
  return (
    <div className="oauth-login-button-wrapper">
      <div className="btn-header">
        <div className="btn-title">Log In</div>
      </div>
      <div className="btn-wrapper">
        <button type="button" className="button btn log-in">
          Log in using OAuth2.0
        </button>
      </div>
    </div>
  );
}
