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
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import PropTypes from 'prop-types';

import { getPropertyValue, formatPropertyName } from '../data/property-parsing';
import './PropertyDetailsModal.scss';

function PropertyDetailsModal(props) {
  const { closeFn, propertiesList } = props;

  return (
    <div className="modalForm">
      <FontAwesomeIcon icon="times" className="close" onClick={closeFn} />
      <div className="content propertiesTable-wrapper">
        <table className="propertiesTable">
          <tr className="propertiesTable-header">
            <th>Attribute Name</th>
            <th>Value</th>
          </tr>
          {propertiesList
            .filter(property => property.name !== 'image_url')
            .map(property => {
              let value;
              try {
                value = getPropertyValue(property);
              } catch (e) {
                console.error(e);
                value = 'unknown';
              }
              return (
                <PropertyDetailsRow
                  name={formatPropertyName(property.name)}
                  value={value}
                />
              );
            })}
        </table>
      </div>
    </div>
  );
}

PropertyDetailsModal.propTypes = {
  closeFn: PropTypes.func.isRequired,
  propertiesList: PropTypes.array.isRequired
};

function PropertyDetailsRow(props) {
  const { name, value } = props;

  return (
    <tr className="propertiesTable-row">
      <td>{name}</td>
      <td>{value}</td>
    </tr>
  );
}

PropertyDetailsRow.propTypes = {
  name: PropTypes.string.isRequired,
  value: PropTypes.string.isRequired
};

export default PropertyDetailsModal;
