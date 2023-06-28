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
import Icon from '@material-ui/core/Icon';
import { get } from '../../request';

export const Logout = () => {
  const onLogout = async () => {
    if (window.sessionStorage.getItem('CANOPY_USER')) {
      const { splinterURL } = window.$CANOPY.getSharedConfig().canopyConfig;
      const { token } = JSON.parse(
        window.sessionStorage.getItem('CANOPY_USER')
      );

      sessionStorage.removeItem('CANOPY_USER');
      if (window.sessionStorage.getItem('CANOPY_KEYS')) {
        window.sessionStorage.removeItem('CANOPY_KEYS');
        if (window.sessionStorage.getItem('KEY_SECRET')) {
          window.sessionStorage.removeItem('KEY_SECRET');
        }
      }

      if (sessionStorage.getItem('LOGOUT')) {
        try {
          await get(
            `${splinterURL}${window.sessionStorage.getItem('LOGOUT')}`,
            request => {
              request.setRequestHeader('Authorization', `Bearer ${token}`);
            }
          );
          window.sessionStorage.removeItem('LOGOUT');
          window.location.href = `${window.location.origin}/login`;
        } catch (err) {
          switch (err.status) {
            case 401:
              window.location.href = `${window.location.origin}/login`;
              break;
            default:
              break;
          }
        }
      } else {
        window.location.href = `${window.location.origin}/login`;
      }
    } else {
      window.location.href = `${window.location.origin}/login`;
    }
  };

  return (
    <button type="button" className="Logout" title="logout" onClick={onLogout}>
      <div className="border">
        <div className="icon">
          <Icon>exit_to_app_icon</Icon>
        </div>
      </div>
      <div className="label">Logout</div>
    </button>
  );
};
