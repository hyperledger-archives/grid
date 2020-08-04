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

import PropTypes from 'prop-types';
import React from 'react';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faWrench, faCopy } from '@fortawesome/free-solid-svg-icons';

import { Service } from '../../data/circuits';
import './ServiceDetails.scss';

const copyToClipboard = val => {
  const el = document.createElement('textarea');
  el.value = val;
  el.setAttribute('readonly', '');

  // Hide the area.
  el.style.position = 'absolute';
  el.style.left = '-9999px';
  document.body.appendChild(el);

  /// select its content and copy it
  el.select();
  document.execCommand('copy');

  // remove the text area
  document.body.removeChild(el);
};

const ServiceDetails = ({ services }) => {
  const [selectedService, setSelectedService] = React.useState(0);
  if (services.length === 0) {
    return <div />;
  }

  const serviceArgStr = JSON.stringify(
    services[selectedService].arguments,
    null,
    2
  );

  return (
    <div className="service-details">
      <ul className="service-selector">
        {services.map((service, idx) => (
          <li className={idx === selectedService ? 'active' : ''}>
            <button type="button" onClick={() => setSelectedService(idx)}>
              <FontAwesomeIcon icon={faWrench} />
              <span className="service-id">{service.serviceId}</span>
              <span className="service-type">{service.serviceType}</span>
            </button>
          </li>
        ))}
      </ul>
      <div className="service-arguments">
        <div className="copy-icon">
          <FontAwesomeIcon
            icon={faCopy}
            onClick={() => copyToClipboard(serviceArgStr)}
          />
        </div>
        <textarea readOnly="true">{serviceArgStr}</textarea>
      </div>
    </div>
  );
};

ServiceDetails.propTypes = {
  services: PropTypes.arrayOf(PropTypes.instanceOf(Service)).isRequired
};

export default ServiceDetails;
