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
import Icon from '@material-ui/core/Icon';

export const NavItem = props => {
  const { path, icon, label, logo } = props;

  const classes = classnames('nav-tab', {
    'page-active': path === `/${window.location.pathname.split('/')[1]}`
  });

  return (
    <a href={path} className={classes}>
      <div className="border">
        <div className="icon">
          {logo ? (
            <img src={logo} className="brand-logo" alt={label} />
          ) : (
            <Icon>{icon}</Icon>
          )}
        </div>
      </div>
      <div className="label">{label}</div>
    </a>
  );
};

const iconImagePropsCheck = (props, propName, componentName) => {
  if (!props.icon && !props.logo) {
    return new Error(
      `One of 'logo' or 'icon' is required by '${componentName}' component`
    );
  }

  if (
    typeof props[propName] !== 'string' &&
    typeof props[propName] !== 'undefined'
  ) {
    return new Error(
      `Invalid prop '${propName}' passed to '${componentName}': must be a string`
    );
  }

  return true;
};

NavItem.propTypes = {
  icon: iconImagePropsCheck,
  label: PropTypes.string.isRequired,
  logo: iconImagePropsCheck,
  path: PropTypes.string.isRequired
};

NavItem.defaultProps = {
  icon: undefined,
  logo: undefined
};
