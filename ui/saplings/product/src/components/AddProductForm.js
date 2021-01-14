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
import React, { useCallback, useEffect, useReducer, useState } from 'react';
import PropTypes from 'prop-types';
import { useToasts } from 'react-toast-notifications';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import _ from 'lodash';
import papa from 'papaparse';
import { useServiceState } from '../state/service-context';
import { MultiStepForm, Step, StepInput } from './MultiStepForm';
import { Chips, Chip } from './Chips';
import { MultiSelect } from './MultiSelect';
import Loading from './Loading';
import { addProduct } from '../api/transactions';

import './forms.scss';

const reducer = (state, action) => {
  switch (action.type) {
    case 'add':
      return [...new Set([...state, ...action.payload])];
    case 'remove':
      return state.filter(item => item !== action.payload);
    case 'set':
      return action.payload;
    case 'clear':
      return [];
    default:
      throw new Error(`Invalid action type: ${action.type}`);
  }
};

export function AddProductForm({ closeFn }) {
  const initialErrors = [
    'GTIN: please enter a valid GTIN',
    'Services: please select at least one service'
  ];
  const { services } = useServiceState();
  const [errors, dispatchErrors] = useReducer(reducer, initialErrors);
  const [orgId, setOrgId] = useState(null);
  const [gtin, setGtin] = useState(null);
  const [selectedServices, dispatchSelectedServices] = useReducer(reducer, []);
  const [fileLabel, setFileLabel] = useState('Upload master data file');
  const [attributes, setAttributes] = useState([]);
  const [attrState, setAttrState] = useState({
    type: '',
    name: '',
    value: ''
  });
  const [loading, setLoading] = useState(false);
  const { addToast } = useToasts();

  useEffect(() => {
    const errorMessage = 'Services: please select at least one service';
    if (!selectedServices.length) {
      dispatchErrors({ type: 'add', payload: [errorMessage] });
    } else {
      dispatchErrors({ type: 'remove', payload: errorMessage });
    }
  }, [selectedServices]);

  const gtinValidator = '\\b\\d{8}(?:\\d{4,6})?\\b';

  const handleGtinChange = e => {
    const { value } = e.target;
    const errorMessage = 'GTIN: please enter a valid GTIN';
    if (!e.target.validity.valid) {
      dispatchErrors({ type: 'add', payload: [errorMessage] });
    } else {
      dispatchErrors({ type: 'remove', payload: errorMessage });
    }
    setGtin(value);
  };

  const handleFileUpload = e => {
    setFileLabel(e.target.files[0].name);
    papa.parse(e.target.files[0], {
      complete(results) {
        setAttributes([...attributes, ...results.data]);
      },
      header: true,
      skipEmptyLines: true
    });
  };

  const handleServiceChange = useCallback(
    newServices => {
      dispatchSelectedServices({ type: 'set', payload: [...newServices] });
    },
    [dispatchSelectedServices]
  );

  const addAttribute = e => {
    e.preventDefault();
    setAttributes([...attributes, attrState]);
    setAttrState({
      type: '',
      name: '',
      value: ''
    });
  };

  const removeAttr = attr => {
    setAttributes(attributes.filter(attribute => !_.isEqual(attribute, attr)));
  };

  const handleAttrChange = e => {
    const { name, value } = e.target;
    setAttrState({
      ...attrState,
      [name]: value
    });
  };

  const handleOrganizationChange = e => {
    const { value } = e.target;
    setOrgId(value);
  };

  const createAttrData = attribute => {
    return (
      <div className="attribute-data">
        <span className="name">{attribute.name}</span>
        <span className="type">{attribute.type}</span>
        <span className="value">{attribute.value}</span>
      </div>
    );
  };

  const clearState = () => {
    setGtin(null);
    setOrgId('');
    dispatchSelectedServices({ type: 'clear' });
    dispatchErrors({ type: 'add', payload: initialErrors });
    setAttributes([]);
    setAttrState({
      type: '',
      name: '',
      value: ''
    });
    setFileLabel('Upload master data file');
  };

  const submitCallback = () => {
    addToast('Submitted successfully', { appearance: 'success' });
    setLoading(false);
    closeFn();
  };

  const submitFn = async () => {
    const keys = JSON.parse(sessionStorage.getItem('CANOPY_KEYS'));
    await addProduct(
      {
        productId: gtin,
        orgName: orgId,
        properties: attributes,
        services: selectedServices
      },
      keys,
      submitCallback
    );
    clearState();
    setLoading(true);
  };

  const makeListItems = () => {
    return services.map(service => ({
      label: service,
      value: service
    }));
  };

  return (
    <div className="modalForm">
      <FontAwesomeIcon icon="times" className="close" onClick={closeFn} />
      <div className="content">
        {loading && <Loading />}
        {!loading && (
          <MultiStepForm
            formName="Add Product"
            handleSubmit={submitFn}
            disabled={!!errors.length}
          >
            <Step step={1} label="Specify product">
              <StepInput
                type="text"
                label="GTIN"
                name="gtin"
                value={gtin}
                pattern={gtinValidator}
                onChange={handleGtinChange}
              />
              <div className="divider" />
              <h6>Select service(s)</h6>
              <MultiSelect
                listItems={makeListItems()}
                placeholder="No services selected"
                onChange={handleServiceChange}
                value={selectedServices}
              />
              <StepInput
                type="text"
                label="Organization ID"
                name="orgId"
                value={orgId}
                onChange={handleOrganizationChange}
              />
            </Step>
            <Step step={2} label="Add master data">
              <StepInput
                type="file"
                accept="text/csv"
                id="add-master-data-file"
                label={fileLabel}
                onChange={handleFileUpload}
              />
              <div className="divider" />
              <h6>Add attributes</h6>
              <div className="form-group">
                <StepInput
                  type="select"
                  label="Attribute type"
                  name="type"
                  value={attrState.type}
                  onChange={handleAttrChange}
                >
                  <option value="">(none)</option>
                  <option value="STRING" default>
                    Text
                  </option>
                  <option value="NUMBER">Number</option>
                  <option value="BOOLEAN">Boolean</option>
                </StepInput>
              </div>
              <div className="form-group">
                <StepInput
                  type="text"
                  label="Attribute name"
                  name="name"
                  value={attrState.name}
                  onChange={handleAttrChange}
                />
                <StepInput
                  type={attrState.type.toLowerCase()}
                  label="Attribute value"
                  name="value"
                  value={attrState.value}
                  onChange={handleAttrChange}
                />
                <button
                  className="confirm"
                  type="button"
                  onClick={addAttribute}
                  disabled={
                    !(attrState.type && attrState.name && attrState.value)
                  }
                >
                  Add
                </button>
              </div>
              <Chips>
                {attributes.map(attribute => {
                  const data = createAttrData(attribute);
                  return (
                    <Chip
                      label={attribute.name}
                      data={data}
                      removeFn={() => removeAttr(attribute)}
                      deleteable
                    />
                  );
                })}
              </Chips>
            </Step>
            <Step step={3} label="Review and submit">
              {!!errors.length && (
                <div className="error-messages">
                  {errors.map(error => (
                    <span>{error}</span>
                  ))}
                </div>
              )}
              <h6>Review new product</h6>
              <span>
                GTIN: <b>{gtin}</b>
              </span>
              <h6>Selected services</h6>
              <Chips>
                {selectedServices.length > 0 &&
                  selectedServices.map(service => {
                    const data = services.filter(s => s === service);

                    return data.length && <Chip label={data[0]} />;
                  })}
                {!selectedServices.length && <span>No services selected</span>}
              </Chips>
              <h6>Attributes</h6>
              <Chips>
                {attributes.length > 0 &&
                  attributes.map(attribute => {
                    const data = createAttrData(attribute);
                    return <Chip label={attribute.name} data={data} />;
                  })}
                {!attributes.length && <span>No attributes entered</span>}
              </Chips>
            </Step>
          </MultiStepForm>
        )}
      </div>
    </div>
  );
}

AddProductForm.propTypes = {
  closeFn: PropTypes.func.isRequired
};
