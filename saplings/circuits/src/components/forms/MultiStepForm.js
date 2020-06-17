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
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';

import './MultiStepForm.scss';

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
  children,
  isStepValidFn
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
    <div className="multi-step-form" style={style}>
      <div className="form-header">
        <h5>{formName}</h5>
        <div className="info">
          <div className="step-counter">
            <span className="completion-percentage">
              {`${((step - 1) / (children.length - 1)) * 100}%`}
            </span>
            <div
              className="progress-tracker"
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
                  <div className="step-box">
                    {i < step - 1 && <FontAwesomeIcon icon="check" />}
                  </div>
                  <span className="step-label">{s.props.label}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
      <div className="form-wrapper">
        <form className="steps-form">
          {children.map(child =>
            React.cloneElement(child, { currentStep: step })
          )}
        </form>
        <div className="actions">
          {step > 1 && (
            <button className="form-button" type="button" onClick={previous}>
              Previous
            </button>
          )}
          {step < children.length && (
            <button
              type="button"
              className="form-button confirm"
              disabled={!isStepValidFn(step)}
              onClick={next}
            >
              Next
            </button>
          )}
          {step === children.length && (
            <button
              type="button"
              onClick={submit}
              className="form-button submit"
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
  style: PropTypes.oneOfType([PropTypes.object, PropTypes.array]),
  disabled: PropTypes.bool,
  error: PropTypes.bool,
  children: PropTypes.node,
  isStepValidFn: PropTypes.func
};

MultiStepForm.defaultProps = {
  formName: '',
  style: undefined,
  disabled: false,
  error: false,
  children: undefined,
  isStepValidFn: () => {
    return true;
  }
};
