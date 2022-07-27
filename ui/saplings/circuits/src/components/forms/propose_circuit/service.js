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

import React, { useEffect, useState, useReducer } from 'react';
import PropTypes from 'prop-types';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import {
  faEdit,
  faTrash,
  faSync,
  faPlusCircle,
  faMinusCircle
} from '@fortawesome/free-solid-svg-icons';
import {
  Button,
  InputRow,
  ListBoxSelect,
  TextField
} from 'App/components/forms/controls';
import { useLocalNodeState } from 'App/state/localNode';
import { Service, generateID } from 'App/data/circuits';
import { Node } from 'App/data/nodeRegistry';

import './service.scss';

const NodeDisplay = ({ node }) => {
  const localNodeID = useLocalNodeState();

  if (!node) {
    return <div className="node-display" />;
  }

  return (
    <div className="node-display">
      <div className="field-wrapper">
        <div className="field-header">Name</div>
        <div className="field-value">{node.displayName}</div>
      </div>
      <div className="field-wrapper">
        <div className="field-header">ID</div>
        <div className="field-value">
          {`${node.identity}${localNodeID === node.identity ? ' (Local)' : ''}`}
        </div>
      </div>
    </div>
  );
};

NodeDisplay.propTypes = {
  node: PropTypes.instanceOf(Node).isRequired
};

export const ServiceTable = ({
  services,
  nodes,
  isEdit,
  onServiceEdit,
  onServiceDelete
}) => {
  let tableBody;
  if (services.length === 0) {
    tableBody = (
      <tr>
        <td className="no-services" colSpan="5">
          <center>At least one service must be defined for a circuit.</center>
        </td>
      </tr>
    );
  } else {
    const controls = service => {
      if (isEdit) {
        return (
          <td className="edit-controls">
            <div>
              <Button
                title={`Edit service ${service.serviceId}`}
                onClick={() => onServiceEdit(service.serviceId)}
                label={<FontAwesomeIcon icon={faEdit} />}
              />
              <Button
                title={`Delete service ${service.serviceId}`}
                onClick={() => onServiceDelete(service.serviceId)}
                label={<FontAwesomeIcon icon={faTrash} />}
              />
            </div>
          </td>
        );
      }

      return null;
    };

    tableBody = services.map(service => {
      const node = nodes.find(n => n.identity === service.allowedNodes[0]);
      return (
        <tr key={service.serviceId}>
          <td>{service.serviceType}</td>
          <td>{service.serviceId}</td>
          <td>
            <NodeDisplay node={node} />
          </td>
          <td className="arguments">
            {Object.entries(service.arguments).map(([key, value]) => {
              return (
                <React.Fragment key={key}>
                  {`${key}: ${value}`}
                  <br />
                </React.Fragment>
              );
            })}
          </td>
          {controls(service)}
        </tr>
      );
    });
  }

  return (
    <table className="service-table">
      <thead className="table">
        <tr>
          <th>Service Type</th>
          <th>Service ID</th>
          <th>Node</th>
          <th className="arguments">Arguments</th>
          {isEdit ? <th className="controls">&nbsp;</th> : null}
        </tr>
      </thead>
      <tbody>{tableBody}</tbody>
    </table>
  );
};

ServiceTable.propTypes = {
  services: PropTypes.arrayOf(PropTypes.instanceOf(Service)).isRequired,
  nodes: PropTypes.arrayOf(PropTypes.instanceOf(Node)).isRequired,
  isEdit: PropTypes.bool,
  onServiceEdit: PropTypes.func,
  onServiceDelete: PropTypes.func
};

ServiceTable.defaultProps = {
  isEdit: false,
  onServiceEdit: () => null,
  onServiceDelete: () => null
};

const formValueReducer = (state, [action, ...values]) => {
  if (action === 'reset') {
    return { 'service-args': [{}] };
  }

  if (action === 'init') {
    const [service] = values;
    if (service) {
      const serviceArgs = Object.entries(service.arguments).map(
        ([key, value]) => ({
          key,
          value
        })
      );
      serviceArgs.push({});

      return {
        'service-node': service.allowedNodes[0],
        'service-id': service.serviceId,
        'service-type': service.serviceType,
        'service-args': serviceArgs
      };
    }
    return state;
  }

  if (action === 'service-arg-add') {
    const serviceArgs = state['service-args'];
    serviceArgs.push({});
    return { ...state, 'service-args': serviceArgs };
  }

  if (action === 'service-arg-remove') {
    const [idx] = values;
    const serviceArgs = state['service-args'];
    serviceArgs.splice(idx, 1);
    return { ...state, 'service-args': serviceArgs };
  }

  if (action === 'service-arg-set') {
    const [idx, prop, value] = values;
    const serviceArgs = state['service-args'];
    const currentValue = serviceArgs[idx] || {};
    serviceArgs[idx] = { ...currentValue, [prop]: value };
    return { ...state, 'service-args': serviceArgs };
  }

  if (action) {
    // field set from a regular input
    const [value] = values;
    return { ...state, [action]: value };
  }

  // no action supplied.
  return state;
};

export const ServiceForm = ({ nodes, onComplete, onCancel, service }) => {
  const [values, valueDispatch] = useReducer(formValueReducer, {
    'service-args': [{}]
  });
  const [errors, setErrors] = useState({});
  const [dirty, setDirty] = useState({});

  useEffect(() => {
    if (service) {
      valueDispatch(['init', service]);
    }
  }, [service]);

  const isComplete = () => {
    if (!values['service-type'] || values['service-type'].trim().length < 1) {
      return false;
    }
    if (!values['service-id'] || values['service-id'].trim().length < 1) {
      return false;
    }

    return !!values['service-node'];
  };

  const validators = {
    'service-id': val => {
      if (!val || val.length !== 4) {
        return 'Invalid service ID: It must be 4 characters long';
      }

      const regex = /^[a-zA-Z0-9]+$/i;
      if (!val.match(regex)) {
        return 'Invalid service ID. It must contain only ASCII alphanumeric characters.';
      }
      return null;
    },
    'service-type': val => {
      if (!val || val.length === 0) {
        return 'Service type cannot be empty';
      }
      return null;
    }
  };

  const onBlur = evt => {
    evt.preventDefault();
    const { target } = evt;
    const { name, value } = target;

    if (dirty[name] && validators[name]) {
      const fn = validators[name] || (() => null);
      setErrors({ ...errors, [name]: fn(value) });
    }
  };

  const onChange = evt => {
    evt.preventDefault();
    const { target } = evt;
    const { name, value } = target;
    const fn = validators[name] || (() => null);
    if (!fn(value)) {
      setErrors({ ...errors, [name]: null });
    }
    setDirty({ ...dirty, [name]: true });
    valueDispatch([name, value]);
  };

  const doReset = () => {
    valueDispatch(['reset']);
    setErrors({});
    setDirty({});
  };

  const onSubmit = evt => {
    evt.preventDefault();

    if (isComplete()) {
      const result = new Service();
      result.serviceId = values['service-id'];
      result.serviceType = values['service-type'];
      result.allowedNodes = values['service-node']
        ? [values['service-node']]
        : [];
      result.arguments = values['service-args']
        .filter(({ key }) => !!key)
        .reduce((acc, { key, value }) => ({ ...acc, [key]: value || '' }), {});

      // return the new result and the old
      onComplete(result, service);
      doReset();
    }
  };

  const doCancel = evt => {
    evt.preventDefault();
    doReset();
    onCancel();
  };

  const basicInput = (name, label, ...more) => (
    <TextField
      name={name}
      label={label}
      error={errors[name]}
      value={values[name]}
      onChange={onChange}
      onBlur={onBlur}
    >
      {more}
    </TextField>
  );

  const serviceArg = (idx, prop) => {
    const arg = values['service-args'][idx];
    if (arg) {
      return arg[prop] || '';
    }
    return '';
  };

  const serviceArgHandler = (idx, prop) => evt => {
    evt.preventDefault();
    valueDispatch(['service-arg-set', idx, prop, evt.target.value]);
  };

  const plusOrMinus = (pred, idx) => {
    if (pred) {
      return (
        <Button
          label={<FontAwesomeIcon icon={faMinusCircle} />}
          className="icon-button"
          onClick={() => valueDispatch(['service-arg-remove', idx])}
        />
      );
    }

    return (
      <Button
        label={<FontAwesomeIcon icon={faPlusCircle} />}
        className="icon-button"
        onClick={() => valueDispatch(['service-arg-add'])}
      />
    );
  };

  const serviceArgRow = idx => {
    const key = serviceArg(idx, 'key');
    return (
      <InputRow className="service-args" key={idx}>
        <TextField
          name="arg-key"
          label="Key"
          value={key}
          onChange={serviceArgHandler(idx, 'key')}
        />
        <TextField
          name="arg-value"
          label="Value"
          value={serviceArg(idx, 'value')}
          onChange={serviceArgHandler(idx, 'value')}
        />
        {plusOrMinus(
          idx < values['service-args'].length - 1,
          idx,
          key.length > 0
        )}
      </InputRow>
    );
  };

  const generateId = () => {
    setErrors({ ...errors, 'service-id': null });
    valueDispatch(['service-id', generateID(4)]);
  };

  return (
    <div className="form service-form">
      <span className="service-form-header">Add Service</span>
      <div className="form-content">
        <InputRow>
          {basicInput('service-type', 'Service Type')}
          {basicInput(
            'service-id',
            'Service ID',
            <Button
              label={<FontAwesomeIcon icon={faSync} />}
              className="icon-button"
              onClick={generateId}
            />
          )}
        </InputRow>
        <InputRow>
          <ListBoxSelect
            name="service-node"
            label="Node"
            onChange={onChange}
            value={values['service-node']}
            options={[
              {
                value: '',
                content: <div className="no-node-selected">Select a node</div>
              },
              ...nodes.map(node => ({
                value: node.identity,
                content: <NodeDisplay node={node} />
              }))
            ]}
          />
        </InputRow>

        <span className="service-args-header">
          Service Arguments (Optional)
        </span>
        {values['service-args'].map((_, idx) => serviceArgRow(idx))}

        <div className="button-group">
          <Button label="Cancel" className="form-button" onClick={doCancel} />
          <Button
            label={!values.edit ? 'Add' : 'Update'}
            className="form-button confirm"
            onClick={onSubmit}
            disabled={!isComplete()}
          />
        </div>
      </div>
    </div>
  );
};

ServiceForm.propTypes = {
  onComplete: PropTypes.func.isRequired,
  nodes: PropTypes.arrayOf(PropTypes.instanceOf(Node)).isRequired,
  service: PropTypes.instanceOf(Service),
  onCancel: PropTypes.func
};

ServiceForm.defaultProps = {
  service: null,
  onCancel: () => null
};
