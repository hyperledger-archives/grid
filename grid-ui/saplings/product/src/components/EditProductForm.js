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

import React, { useState, useEffect } from 'react';
import PropTypes from 'prop-types';
import _ from 'lodash';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faTimes } from '@fortawesome/free-solid-svg-icons';
import { MultiStepForm, Step, StepInput } from './MultiStepForm';
import { Chips, Chip } from './Chips';
import { getProperty } from '../data/property-parsing';

import './forms.scss';

export function EditProductForm({ closeFn, properties }) {
  const [attributes, setAttributes] = useState(
    properties.filter(property => property.name !== 'image_url')
  );
  const [attrState, setAttrState] = useState({
    type: '',
    name: '',
    value: ''
  });
  const [imgFile, setImgFile] = useState(null);
  const [imgLabel, setImgLabel] = useState('Upload product image');
  const [imgPreview, setImgPreview] = useState(null);

  useEffect(() => {
    const imageProp = properties.filter(
      property => property.name === 'image_url'
    );
    if (imageProp.length) {
      setImgPreview(imageProp[0].string_value);
    }
  }, [properties]);

  const handleAttrChange = e => {
    const { name, value } = e.target;
    setAttrState({
      ...attrState,
      [name]: value
    });
  };

  const addAttribute = e => {
    e.preventDefault();
    setAttributes([...attributes, attrState]);
    setAttrState({
      type: '',
      name: '',
      value: ''
    });
  };

  const createAttrData = attribute => {
    let data = {
      name: '',
      value: '',
      type: ''
    };
    if (getProperty(attribute.name, properties)) {
      data = {
        name: attribute.name,
        type: attribute.data_type,
        value: getProperty(attribute.name, properties)
      };
    } else {
      data = attribute;
    }
    return (
      <div className="attribute-data">
        <span className="name">{data.name}</span>
        <span className="type">{data.type}</span>
        <span className="value">{data.value}</span>
      </div>
    );
  };

  const removeAttr = attr => {
    setAttributes(attributes.filter(attribute => !_.isEqual(attribute, attr)));
  };

  const handleImgUpload = e => {
    e.preventDefault();
    const file = e.target.files[0];
    setImgLabel(file.name);

    const reader = new FileReader();
    reader.onloadend = () => {
      setImgFile(file);
      setImgPreview(reader.result);
    };

    reader.readAsDataURL(file);
    return imgFile;
  };

  const clearState = () => {
    setAttributes([]);
    setAttrState({
      type: '',
      name: '',
      value: ''
    });
    setImgFile(null);
    setImgPreview(null);
    setImgLabel('Upload product image');
  };

  const submitFn = () => {
    clearState();
  };

  return (
    <div id="edit-product-form" className="modalForm">
      <FontAwesomeIcon icon={faTimes} className="close" onClick={closeFn} />
      <div className="content">
        <MultiStepForm formName="Edit Product" handleSubmit={submitFn}>
          <Step step={1} label="Edit master data">
            <h6>Edit attributes</h6>
            <div className="form-group">
              <StepInput
                type="select"
                label="Attribute type"
                name="type"
                value={attrState.type}
                onChange={handleAttrChange}
              >
                <option value="">(none)</option>
                <option value="text" default>
                  Text
                </option>
                <option value="number">Number</option>
                <option value="boolean">Boolean</option>
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
                type={attrState.type}
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
          <Step step={2} label="Edit attachments">
            <h6>Add additional info</h6>
            <StepInput
              type="file"
              accept="image/png, image/jpeg"
              id="add-master-data-file"
              label={imgLabel}
              onChange={handleImgUpload}
            />
            {imgPreview && (
              <div className="preview-container">
                <img className="img-preview" src={imgPreview} alt="preview" />
              </div>
            )}
          </Step>
          <Step step={3} label="Review and submit">
            <h6>Review product changes</h6>
            {imgPreview && (
              <div className="preview-container">
                <img className="img-preview" src={imgPreview} alt="preview" />
              </div>
            )}
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
      </div>
    </div>
  );
}

EditProductForm.propTypes = {
  closeFn: PropTypes.func.isRequired,
  properties: PropTypes.array
};

EditProductForm.defaultProps = {
  properties: []
};
