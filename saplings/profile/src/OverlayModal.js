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
import PropTypes from 'prop-types';
import classnames from 'classnames';
import './OverlayModal.scss';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faTimes } from '@fortawesome/free-solid-svg-icons';

export function OverlayModal({ open, closeFn, children }) {
  return (
    <div className={classnames('overlay-modal', 'modal', open && 'open')}>
      <FontAwesomeIcon
        icon={faTimes}
        className="close"
        onClick={closeFn}
        tabIndex="0"
      />
      <div className="content">{children}</div>
    </div>
  );
}

OverlayModal.defaultProps = {
  open: false,
  children: undefined
};

OverlayModal.propTypes = {
  open: PropTypes.bool,
  closeFn: PropTypes.func.isRequired,
  children: PropTypes.node
};
