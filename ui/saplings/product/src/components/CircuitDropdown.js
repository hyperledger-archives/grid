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

import React, {useEffect} from 'react';
import PropTypes from 'prop-types';
import { Input } from './Input';
import { useServiceState, useServiceDispatch, parseServiceID } from '../state/service-context';
import { listScabbardServices } from '../api/splinter';
import './CircuitDropdown.scss';

const CircuitDropdown = ({
  className
}) => {
  const { services, selectedService } = useServiceState();
  const serviceDispatch = useServiceDispatch();

  useEffect(() => {
    const getServices = async () => {
      try {
        const servicesList = await listScabbardServices();
        serviceDispatch({
          type: 'setServices',
          payload: {
            services: servicesList
          }
        });
      } catch (e) {
        console.error(`Error listing services: ${e}`);
      }
    };

    getServices();
  }, [serviceDispatch]);

  const circuitOptions = services.map(s => (
    <option key={s} value={s}>{parseServiceID(s).circuit}</option>
  ));

  const handleChange = e => {
    const {value} = e.target;
    serviceDispatch({
      type: 'select',
      payload: {
        serviceID: value
      }
    });
  }

  return (
    <Input type="select" className={`${className} circuit-select`} onChange={handleChange} value={selectedService}>
      <option value="none">None</option>
      {circuitOptions}
    </Input>
  );
};

CircuitDropdown.propTypes = {
  className: PropTypes.string
}

CircuitDropdown.defaultProps = {
  className: undefined
}

export default CircuitDropdown;
