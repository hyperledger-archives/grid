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

import React, { useEffect, useState } from 'react';
import proptypes from 'prop-types';
import { faPlus } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import {
  decryptKey,
  getKeys,
  getSharedConfig,
  getUser,
  setKeys as setSigningKeys
} from 'splinter-saplingjs';
import { ActionList } from './ActionList';
import { KeyCard } from './KeyCard';
import { ChangePasswordForm } from './forms/ChangePasswordForm';
import { AddKeyForm } from './forms/AddKeyForm';
import { UpdateKeyForm } from './forms/UpdateKeyForm';
import { EnterPasswordForm } from './forms/EnterPasswordForm';
import { OverlayModal } from './OverlayModal';
import { http } from './http';

export function Profile() {
  const [modalActive, setModalActive] = useState(false);
  const [form, setForm] = useState({
    formName: '',
    params: {}
  });
  const [keys, setKeys] = useState([]);
  const [stateKeys, setStateKeys] = useState(getKeys);
  const user = getUser();

  useEffect(() => {
    async function fetchUserKeys() {
      if (user) {
        try {
          const { splinterURL } = getSharedConfig().canopyConfig;
          const userKeys = await http(
            'GET',
            `${splinterURL}/biome/users/${user.userId}/keys`,
            {},
            request => {
              request.setRequestHeader('Authorization', `Bearer ${user.token}`);
            }
          );
          setKeys(JSON.parse(userKeys).data);
        } catch (err) {
          switch (err.code) {
            case '401':
              window.location.href = `${window.location.origin}/login`;
              break;
            default:
              break;
          }
        }
      } else {
        window.location.href = `${window.location.origin}/login`;
      }
    }
    fetchUserKeys();
  }, [user]);

  const openModalForm = (formName, params) => {
    const name = formName || '';
    const adata = { ...params } || {};
    setForm({
      formName: name,
      params: adata
    });
    setModalActive(true);
  };

  const updateKeyCallback = async () => {
    setModalActive(false);
    try {
      const { splinterURL } = getSharedConfig().canopyConfig;
      const userKeys = await http(
        'GET',
        `${splinterURL}/biome/users/${user.userId}/keys`,
        {},
        request => {
          request.setRequestHeader('Authorization', `Bearer ${user.token}`);
        }
      );
      setKeys(JSON.parse(userKeys).data);
    } catch (err) {
      switch (err.code) {
        case '401':
          window.location.href = `${window.location.origin}/login`;
          break;
        default:
          break;
      }
    }
  };

  const formView = ({ formName, params }) => {
    switch (formName) {
      case 'add-key':
        return <AddKeyForm successFn={params.successFn} />;
      case 'update-key':
        return (
          <UpdateKeyForm userKey={params.key} closeFn={updateKeyCallback} />
        );
      case 'update-password':
        return <ChangePasswordForm />;
      case 'enter-password':
        return (
          <EnterPasswordForm
            callbackFn={params.callbackFn}
            errorMessage={params.errorMessage}
          />
        );
      default:
        return null;
    }
  };

  formView.propTypes = {
    formName: proptypes.string,
    params: proptypes.object
  };

  formView.defaultProps = {
    formName: '',
    params: undefined
  };

  const activateKey = key => {
    const { public_key, encrypted_private_key } = key;
    const keySecret = sessionStorage.getItem('KEY_SECRET');
    if (keySecret) {
      try {
        const privateKey = decryptKey(
          JSON.parse(encrypted_private_key),
          keySecret
        );
        setStateKeys({
          publicKey: public_key,
          privateKey
        });
        setSigningKeys({
          publicKey: public_key,
          privateKey
        });
        setModalActive(false);
      } catch (err) {
        openModalForm('enter-password', {
          callbackFn: () => activateKey(key),
          errorMessage: `Unable to decrypt key. Error: ${err.message}`
        });
        throw new Error(err.message);
      }
    } else {
      openModalForm('enter-password', { callbackFn: () => activateKey(key) });
    }
  };

  const addKeyCallback = key => {
    setKeys([...keys, key]);
    setModalActive(false);
  };

  return (
    <div id="profile">
      <section className="user-info">
        <div className="display-name info-field">
          <div className="info">
            <h1 className="value">{user && user.displayName}</h1>
          </div>
        </div>
        <ActionList className="user-actions">
          <button
            className="flat"
            onClick={() => openModalForm('update-password')}
          >
            Change password
          </button>
        </ActionList>
      </section>
      <section className="user-keys">
        <h3 id="keys-label">Keys</h3>
        <div className="key-list">
          {keys.length === 0 && <span>No keys added yet</span>}
          {keys.length > 0 &&
            keys.map(key => (
              <KeyCard
                key={key.public_key}
                userKey={key}
                isActive={stateKeys && key.public_key === stateKeys.publicKey}
                setActiveFn={() => activateKey(key)}
                editFn={() => openModalForm('update-key', { key })}
              />
            ))}
        </div>
        <button
          className="fab add-key"
          onClick={() =>
            openModalForm('add-key', {
              successFn: value => addKeyCallback(value)
            })
          }
        >
          <FontAwesomeIcon icon={faPlus} className="icon" />
        </button>
      </section>
      <OverlayModal open={modalActive} closeFn={() => setModalActive(false)}>
        {formView(form)}
      </OverlayModal>
    </div>
  );
}
