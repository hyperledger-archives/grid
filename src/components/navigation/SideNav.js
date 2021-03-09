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
import classnames from 'classnames';
import Icon from '@material-ui/core/Icon';
import { Logout } from './Logout';
import { useUserSaplings } from '../../CanopyContext';

import { NavItem } from './NavItem';

export const SideNav = () => {
  const makeUserSaplingTabs = userSaplings =>
    userSaplings
      .map(({ displayName, namespace, icon }) => {
        return {
          path: `/${namespace}`,
          displayName,
          logo: icon
        };
      })
      .map(({ path, displayName, logo }) => {
        return (
          <NavItem key={path} path={path} label={displayName} logo={logo} />
        );
      });

  return (
    <>
      <a href="/" className="brand">
        <div />
      </a>
      <div className="nav-items">
        {makeUserSaplingTabs(useUserSaplings())}
        <div className="nav-tab">
          <div className="border">
            <div className="icon">
              <Icon eco_icon>eco_icon</Icon>
            </div>
          </div>
          <div className="label">Saplings</div>
        </div>
      </div>
      <div className="canopy-items">
        <ProfileTab />
      </div>
    </>
  );
};

function ProfileTab() {
  const profileClasses = classnames('profile-tab', 'tab', {
    'page-active': `${window.location.pathname.split('/')[1]}` === 'profile'
  });

  return (
    <a href="/profile" className={profileClasses}>
      <div className="border">
        <div className="icon">
          <Icon>person_icon</Icon>
        </div>
      </div>
      <div className="label">Profile</div>
      <Logout />
    </a>
  );
}
