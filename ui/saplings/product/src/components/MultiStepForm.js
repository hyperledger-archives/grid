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

import React, { useState } from 'react';
import classnames from 'classnames';
import PropTypes from 'prop-types';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { Input } from './Input';

import './MultiStepForm.scss';

export function StepInput({
  type,
  accept,
  label,
  id,
  name,
  value,
  pattern,
  onChange,
  required,
  multiple,
  children
}) {
  return (
    <Input
      type={type}
      accept={accept}
      label={label}
      id={id}
      name={name}
      value={value}
      pattern={pattern}
      onChange={onChange}
      multiple={multiple}
      required={required}
    >
      {children}
    </Input>
  );
}

export function Step({ step, currentStep, children }) {
  if (step !== currentStep) {
    return null;
  }
  return <div className="step">{children}</div>;
}

export function MultiStepForm({
  formName,
  handleSubmit,
  style,
  disabled,
  error,
  children
}) {
  const [step, setStep] = useState(1);

  const previous = e => {
    e.preventDefault();
    setStep(step - 1);
  };

  const next = e => {
    e.preventDefault();
    setStep(step + 1);
  };

  const submit = () => {
    handleSubmit();
    setStep(1);
  };

  return (
    <div className="multiStepForm" style={style}>
      <div className="info">
        <h5>{formName}</h5>
        <div className="stepCounter">
          <div
            className="progressTracker"
            style={{
              '--form-progress': `${((step - 1) / (children.length - 1)) *
                100}%`
            }}
          />
          <div className="steps">
            {children.map((s, i) => (
              <div
                className={classnames(
                  'step',
                  i === step - 1 && 'active',
                  i < step - 1 && 'entered'
                )}
              >
                <div className="stepBox">
                  {i < step - 1 && <FontAwesomeIcon icon="check" />}
                </div>
                <span className="stepLabel">{s.props.label}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
      <div className="formWrapper">
        <form>
          {children.map(child =>
            React.cloneElement(child, { currentStep: step })
          )}
        </form>
        <div className="actions">
          {step > 1 && (
            <button type="button" onClick={previous}>
              Previous
            </button>
          )}
          {step < children.length && (
            <button type="button" className="confirm" onClick={next}>
              Next
            </button>
          )}
          {step === children.length && (
            <button
              type="button"
              onClick={submit}
              className="submit"
              disabled={disabled || error}
            >
              Submit
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

StepInput.propTypes = {
  type: PropTypes.oneOf(['text', 'password', 'file', 'select', 'number']),
  accept: PropTypes.string,
  label: PropTypes.string,
  id: PropTypes.string,
  name: PropTypes.string,
  value: PropTypes.oneOfType([PropTypes.string, PropTypes.number]),
  pattern: PropTypes.string,
  onChange: PropTypes.func.isRequired,
  required: PropTypes.bool,
  multiple: PropTypes.bool,
  children: PropTypes.node
};

StepInput.defaultProps = {
  type: 'text',
  accept: undefined,
  label: '',
  id: undefined,
  name: undefined,
  value: null,
  pattern: undefined,
  required: false,
  multiple: false,
  children: undefined
};

Step.propTypes = {
  step: PropTypes.number,
  currentStep: PropTypes.number,
  children: PropTypes.node
};

Step.defaultProps = {
  step: 1,
  currentStep: 1,
  children: undefined
};

MultiStepForm.propTypes = {
  formName: PropTypes.string,
  handleSubmit: PropTypes.func.isRequired,
  style: PropTypes.object,
  disabled: PropTypes.bool,
  error: PropTypes.bool,
  children: PropTypes.node
};

MultiStepForm.defaultProps = {
  formName: '',
  style: undefined,
  disabled: false,
  error: false,
  children: undefined
};
