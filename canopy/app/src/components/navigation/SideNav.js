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
import classnames from 'classnames';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';

import NavItem from 'components/navigation/NavItem';

import 'components/navigation/SideNav.scss';

function SideNav(props) {
  const { userSaplingRoutes } = props;
  const userSaplingTabs = userSaplingRoutes.map(
    ({ path, displayName, logo }) => {
      return <NavItem key={path} path={path} label={displayName} logo={logo} />;
    }
  );

  return (
    <div className="side-nav">
      <a href="/" className="brand">
        <h5>Canopy</h5>
      </a>
      <hr />
      <div className="nav-items">{userSaplingTabs}</div>
      <hr className="bottom" />
      <ProfileTab />
    </div>
  );
}

function ProfileTab() {
  const profileClasses = classnames('profile-tab', {
    'page-active': `${window.location.pathname.split('/')[1]}` === 'profile'
  });

  return (
    <a href="/profile" className={profileClasses}>
      <FontAwesomeIcon className="icon" icon="user-circle" />
      <div className="label">
        <div>username</div>
        <div className="key-name">active key name</div>
      </div>
    </a>
  );
}

SideNav.propTypes = {
  userSaplingRoutes: PropTypes.arrayOf(
    PropTypes.shape({
      path: PropTypes.string.isRequired,
      displayName: PropTypes.string.isRequired
    })
  )
};

SideNav.defaultProps = {
  userSaplingRoutes: []
};

export default SideNav;
