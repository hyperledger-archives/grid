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

import React, { useState } from 'react';
import classnames from 'classnames';
import PropTypes from 'prop-types';
import { Input } from '../Input';

import './MultiStepForm.scss';

export function StepInput({
  type,
  label,
  id,
  name,
  value,
  onChange,
  required
}) {
  return (
    <Input
      type={type}
      label={label}
      id={id}
      name={name}
      value={value}
      onChange={onChange}
      required={required}
    />
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

  const _previous = () => {
    setStep(step - 1);
  };

  const _next = () => {
    setStep(step + 1);
  };

  const submit = () => {
    handleSubmit();
    setStep(1);
  };

  return (
    <div className="multiStepForm" style={style}>
      <h2>{formName}</h2>
      <div className="stepCounter">
        <div
          className="progressTracker"
          style={{
            '--form-progress': `${((step - 1) / (children.length - 1)) * 100}%`
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
              <span>{i + 1}</span>
            </div>
          ))}
        </div>
      </div>

      <form onSubmit={handleSubmit}>
        {children.map(child =>
          React.cloneElement(child, { currentStep: step })
        )}
      </form>
      <div className="actions">
        {step > 1 && <button onClick={_previous}>Previous</button>}
        {step < children.length && <button onClick={_next}>Next</button>}
        {step === children.length && (
          <button
            onClick={submit}
            className="submit"
            disabled={disabled || error}
          >
            Submit
          </button>
        )}
      </div>
    </div>
  );
}

StepInput.propTypes = {
  type: PropTypes.string,
  label: PropTypes.string,
  id: PropTypes.string,
  name: PropTypes.string,
  value: PropTypes.any,
  onChange: PropTypes.func.isRequired,
  required: PropTypes.bool
};

StepInput.defaultProps = {
  type: 'text',
  label: undefined,
  id: undefined,
  name: undefined,
  value: null,
  required: false
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
  formName: PropTypes.string.isRequired,
  handleSubmit: PropTypes.func.isRequired,
  children: PropTypes.node,
  style: PropTypes.object,
  disabled: PropTypes.bool,
  error: PropTypes.bool
};

MultiStepForm.defaultProps = {
  children: undefined,
  style: undefined,
  disabled: false,
  error: false
};
