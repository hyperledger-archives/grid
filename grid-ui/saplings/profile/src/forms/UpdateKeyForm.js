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
import React, { useState } from 'react';
import proptypes from 'prop-types';
import { getSharedConfig } from 'splinter-saplingjs';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faTimes } from '@fortawesome/free-solid-svg-icons';
import { Input } from '../Input';
import { http } from '../http';

import './UpdateKeyForm.scss';

export function UpdateKeyForm({ userKey, closeFn }) {
  const [keyName, setKeyName] = useState(null);
  const [error, setError] = useState(false);
  const [errorMsg, setErrorMsg] = useState(null);

  const submitUpdateKey = async e => {
    e.preventDefault();
    const canopyUser = JSON.parse(window.sessionStorage.getItem('CANOPY_USER'));
    const body = JSON.stringify({
      public_key: userKey.public_key,
      new_display_name: keyName
    });

    try {
      const { splinterURL } = getSharedConfig().canopyConfig;
      await http(
        'PATCH',
        `${splinterURL}/biome/users/${canopyUser.userId}/keys`,
        body,
        request => {
          request.setRequestHeader(
            'Authorization',
            `Bearer ${canopyUser.token}`
          );
        }
      );
      closeFn();
    } catch (err) {
      const { message } = JSON.parse(err);
      setErrorMsg(message);
      setError(true);
    }
  };

  const handleChange = event => {
    const { value } = event.target;
    setKeyName(value);
  };

  const reset = () => {
    setError(false);
    setErrorMsg(null);
    setKeyName(null);
  };

  return (
    <div className="wrapper">
      {!error && (
        <>
          <h2>Update key name</h2>
          <div className="info display-name">
            <span>Current name: </span>
            <b>{userKey.display_name}</b>
          </div>
          <div className="info public-key">
            <span>public key: </span>
            <b>{userKey.public_key}</b>
          </div>
          <form id="change-key-form" aria-label="Change-key-form">
            <Input
              type="text"
              label="New key name"
              id="change-key"
              value={keyName}
              onChange={handleChange}
              required
            />
            <button
              className="submit"
              onClick={submitUpdateKey}
              disabled={!keyName}
            >
              Submit
            </button>
          </form>
        </>
      )}
      {error && (
        <div className="error-wrapper">
          <FontAwesomeIcon className="icon" icon={faTimes} />
          <div className="error">
            <span>{errorMsg}</span>
          </div>
          <div className="actions-wrapper">
            <button onClick={reset}>Reset</button>
          </div>
        </div>
      )}
    </div>
  );
}

UpdateKeyForm.propTypes = {
  userKey: proptypes.object.isRequired,
  closeFn: proptypes.func.isRequired
};
