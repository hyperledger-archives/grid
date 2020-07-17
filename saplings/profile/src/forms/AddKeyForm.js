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
import { encryptKey, getSharedConfig } from 'splinter-saplingjs';
import { Secp256k1Context, Secp256k1PrivateKey } from 'transact-sdk-javascript';
import crypto from 'crypto';
import { MultiStepForm, Step, StepInput } from './MultiStepForm';
import { Loader } from '../Loader';
import { http } from '../http';

import './AddKeyForm.scss';

export function AddKeyForm({ successFn }) {
  const [state, setState] = useState({
    name: null,
    publicKey: null,
    privateKey: null,
    password: null
  });

  const [loading, setLoading] = useState(false);
  const [loadingState, setLoadingState] = useState(null);
  const [error, setError] = useState(null);

  const reset = () => {
    setState({
      name: null,
      publicKey: null,
      privateKey: null,
      password: null
    });
    setError(null);
    setLoading(false);
  };

  async function submitAddKey() {
    setLoading(true);

    // Validate the public and private keys are related:
    const context = new Secp256k1Context();
    try {
      const privateKey = Secp256k1PrivateKey.fromHex(state.privateKey);
      const publicKey = context.getPublicKey(privateKey).asHex().toLowerCase();
      const submittedPublicKey = state.publicKey.trim().toLowerCase();
      if (publicKey !== submittedPublicKey) {
        setError("The private key provided is not valid for the given public key.");
        setLoadingState('failure');
        return;
      }
    } catch (err) {
        setError("The private key provided is not a valid key.");
        setLoadingState('failure');
        return;
    }

    const canopyUser = JSON.parse(window.sessionStorage.getItem('CANOPY_USER'));
    const keySecret = crypto
      .createHash('sha256')
      .update(state.password)
      .digest('hex');
    const encryptedPrivateKey = JSON.parse(
      encryptKey(state.privateKey, keySecret)
    );
    const body = JSON.stringify({
      display_name: state.name,
      encrypted_private_key: encryptedPrivateKey,
      public_key: state.publicKey,
      user_id: canopyUser.userId
    });

    try {
      const { splinterURL } = getSharedConfig().canopyConfig;
      await http('POST', `${splinterURL}/biome/keys`, body, request => {
        request.setRequestHeader('Authorization', `Bearer ${canopyUser.token}`);
      });
      reset();
      successFn(JSON.parse(body));
    } catch (err) {
      try {
        const e = JSON.parse(err);
        setError(e.message);
      } catch {
        setError(err.toString());
      }
      setLoadingState('failure');
    }
  }

  const handleChange = event => {
    const { name, value } = event.target;
    setState({
      ...state,
      [name]: value
    });
  };

  const generateKeys = e => {
    e.preventDefault();
    const context = new Secp256k1Context();
    const privKey = context.newRandomPrivateKey();
    const pubKey = context.getPublicKey(privKey);
    setState({
      ...state,
      publicKey: pubKey.asHex(),
      privateKey: privKey.asHex()
    });
  };

  return (
    <div className="wrapper form-wrapper">
      {!loading && (
        <MultiStepForm
          formName="Add key"
          handleSubmit={submitAddKey}
          disabled={
            !(
              state.name &&
              state.privateKey &&
              state.publicKey &&
              state.password
            )
          }
        >
          <Step step={1}>
            <StepInput
              type="text"
              label="Key name"
              id="key-name"
              name="name"
              value={state.name}
              onChange={handleChange}
              required
            />
            <StepInput
              type="text"
              label="Public key"
              id="public-key"
              name="publicKey"
              value={state.publicKey}
              onChange={handleChange}
              required
            />
            <StepInput
              type="password"
              label="Private key"
              id="private-key"
              name="privateKey"
              value={state.privateKey}
              onChange={handleChange}
              required
            />
            <button onClick={generateKeys} style={{ marginTop: '0.5rem' }}>
              Generate keys
            </button>
          </Step>
          <Step step={2}>
            <StepInput
              type="password"
              label="Password"
              id="password"
              name="password"
              value={state.password}
              onChange={handleChange}
              required
            />
          </Step>
        </MultiStepForm>
      )}
      {loading && <Loader state={loadingState} />}
      {error && (
        <div className="error-wrapper">
          <div
            className="error"
            style={{ color: 'var(--color-failure', wordWrap: 'break-word' }}
          >
            <span>{error}</span>
          </div>
          <div className="actions">
            <button onClick={reset}>Reset</button>
          </div>
        </div>
      )}
    </div>
  );
}

AddKeyForm.propTypes = {
  successFn: proptypes.func.isRequired
};
