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

import React, { useState, useReducer, useEffect } from 'react';
import PropTypes from 'prop-types';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { Service, generateID } from '../data/circuits';
import { Node } from '../data/nodeRegistry';
import { PlusButton, MinusButton } from './PlusMinusButton';

import './ServiceCard.scss';

const isValidID = serviceId => {
  if (serviceId.length === 4) {
    const regex = /^[a-zA-Z0-9]+$/i;
    if (serviceId.match(regex)) {
      return true;
    }
    return false;
  }
  return false;
};

const generateServiceID = () => {
  return generateID(4);
};

const serviceReducer = (state, action) => {
  switch (action.type) {
    case 'set-service-type': {
      const { serviceType } = action;
      const { errors, service } = state;
      if (serviceType.length === 0) {
        errors.serviceType = 'Service type cannot be empty';
      } else {
        errors.serviceType = '';
      }
      service.serviceType = serviceType;

      return { ...state, service, errors };
    }
    case 'set-service-id': {
      const { serviceId } = action;
      const { errors, service } = state;
      if (serviceId.length === 0) {
        errors.serviceId = 'Service ID cannot be empty';
      } else if (!isValidID(serviceId)) {
        errors.serviceId =
          'Invalid service ID. It must be 4 characters long and contain only ASCII alphanumeric characters.';
      } else {
        errors.serviceId = '';
      }
      service.serviceId = serviceId;

      return { ...state, serviceId, errors };
    }
    case 'set-allowed-nodes': {
      const { allowedNodes } = action;
      const { errors, service } = state;
      if (allowedNodes.length === 0) {
        errors.allowedNodes = 'Allowed nodes cannot be empty';
      } else {
        errors.allowedNodes = '';
      }

      service.allowedNodes = allowedNodes;

      return { ...state, service, errors };
    }
    case 'check-allowed-nodes': {
      const { errors } = state;
      if (state.service.allowedNodes.length === 0) {
        errors.allowedNodes = 'Allowed nodes cannot be empty';
      } else {
        errors.allowedNodes = '';
      }

      return { ...state, errors };
    }
    case 'set-arguments': {
      const args = action.arguments;
      const { errors, service } = state;
      if (args.length === 0) {
        errors.arguments = 'Service arguments cannot be empty';
      } else {
        errors.arguments = '';
      }

      service.arguments = args;

      return { ...state, service, errors };
    }
    case 'set-service': {
      const { service } = action;
      return { ...state, service };
    }
    default:
      throw new Error(`unhandled action type: ${action.type}`);
  }
};

const argumentsReducer = (state, action) => {
  switch (action.type) {
    case 'add-field': {
      state.arguments.push({
        key: '',
        value: ''
      });
      return { ...state };
    }
    case 'remove-field': {
      const { index } = action;
      state.arguments.splice(index, 1);
      return { ...state };
    }
    case 'set-field-key': {
      const newState = state;
      const { key, index } = action;
      newState.arguments[index].key = key;
      if (key.length !== 0) {
        delete newState.errors[index];
      }

      if (key.length === 0 && newState.arguments[index].value.length !== 0) {
        const error = 'Key cannot be empty';
        newState.errors[index] = error;
      }

      return { ...newState };
    }
    case 'set-field-value': {
      const newState = state;
      const { value, index } = action;
      newState.arguments[index].value = value;
      if (newState.arguments[index].key.length === 0 && value.length !== 0) {
        const error = 'Key cannot be empty';
        newState.errors[index] = error;
      } else {
        delete newState.errors[index];
      }
      return { ...newState };
    }
    case 'reset-arguments': {
      const args = Object.entries(action.arguments).map(([key, value]) => {
        return {
          key,
          value
        };
      });
      return { ...state, arguments: args };
    }
    case 'filter-empty': {
      const filteredArgs = state.arguments.filter(arg => {
        if (arg.key.length > 0) {
          return true;
        }
        return false;
      });

      return { ...state, arguments: filteredArgs };
    }
    case 'clear': {
      return {
        arguments: [
          {
            key: '',
            value: ''
          }
        ],
        errors: {}
      };
    }
    default:
      throw new Error(`unhandled action type: ${action.type}`);
  }
};

const ServiceCard = ({
  service,
  applyServiceChanges,
  deleteService,
  isEditable,
  enterEditMode,
  isDeletable,
  nodes,
  localNodeID
}) => {
  const [editMode, setEditMode] = useState(enterEditMode);
  const [deleteOnDismiss, setDeleteOnDismiss] = useState(true);
  const [formComplete, setFormComplete] = useState(false);
  const [serviceState, serviceDispatcher] = useReducer(serviceReducer, {
    service: { ...service },
    errors: {
      serviceType: '',
      serviceId: '',
      allowedNodes: '',
      serviceArguments: ''
    }
  });
  const [showAllowedNodes, setShowAllowedNodes] = useState(false);
  const [argumentsState, argumentsDispatcher] = useReducer(argumentsReducer, {
    arguments: Object.entries(serviceState.service.arguments).map(
      ([key, value]) => {
        return {
          key,
          value
        };
      }
    ),
    errors: {}
  });

  useEffect(() => {
    serviceDispatcher({ type: 'set-service', service: { ...service } });
  }, [service]);

  useEffect(() => {
    const serviceTypeIsValid =
      serviceState.errors.serviceType.length === 0 &&
      serviceState.service.serviceType.length !== 0;

    const serviceIdIsValid =
      serviceState.errors.serviceId.length === 0 &&
      serviceState.service.serviceId.length !== 0;

    const allowedNodesIsValid =
      serviceState.errors.allowedNodes.length === 0 &&
      serviceState.service.allowedNodes.length !== 0;

    const serviceArgumentsIsValid =
      Object.keys(argumentsState.errors).length === 0;

    if (
      serviceTypeIsValid &&
      serviceIdIsValid &&
      allowedNodesIsValid &&
      serviceArgumentsIsValid
    ) {
      setFormComplete(true);
    } else {
      setFormComplete(false);
    }
  }, [serviceState, argumentsState]);

  const caretDown = (
    <span className="icon-btn caret">
      <FontAwesomeIcon icon="caret-down" />
    </span>
  );

  const caretUp = (
    <span className="icon-btn caret">
      <FontAwesomeIcon icon="caret-up" />
    </span>
  );

  const editButton = (
    <button
      className="icon-btn"
      type="button"
      title="Edit service"
      onClick={() => {
        setEditMode(true);
        setDeleteOnDismiss(false);
      }}
    >
      <FontAwesomeIcon icon="pen" />
    </button>
  );

  const deleteButton = (
    <button
      className="icon-btn"
      title="Delete service"
      type="button"
      disabled={!isDeletable}
      onClick={() => {
        deleteService();
      }}
    >
      <FontAwesomeIcon icon="trash" />
    </button>
  );

  const confirmButton = (
    <button
      className="icon-btn color-confirm"
      type="button"
      onClick={async () => {
        setEditMode(false);
        const args = {};
        argumentsState.arguments.forEach(arg => {
          if (arg.key.length > 0) {
            args[arg.key] = arg.value;
          }
        });
        serviceDispatcher({ type: 'set-arguments', arguments: args });
        argumentsDispatcher({ type: 'filter-empty' });
        applyServiceChanges({ ...serviceState.service, arguments: args });
      }}
      title="Submit changes"
      disabled={!formComplete}
    >
      <FontAwesomeIcon icon="check" />
    </button>
  );

  const dismissButton = (
    <button
      className="icon-btn color-danger"
      type="button"
      title="Dismiss changes"
      disabled={!isDeletable && deleteOnDismiss}
      onClick={() => {
        if (deleteOnDismiss) {
          deleteService();
        } else {
          serviceDispatcher({ type: 'set-service', service: { ...service } });
          argumentsDispatcher({
            type: 'reset-arguments',
            arguments: { ...service.arguments }
          });
          setEditMode(false);
        }
      }}
    >
      <FontAwesomeIcon icon="times" />
    </button>
  );

  const allowedNodes = (
    <input
      className="service-input"
      value={serviceState.service.allowedNodes}
      disabled
    />
  );

  const allowedNodeEditMode = (
    <div>
      <button
        type="button"
        className="allowed-nodes-dropdown service-input"
        onClick={() => {
          if (showAllowedNodes) {
            serviceDispatcher({ type: 'check-allowed-nodes' });
          }
          setShowAllowedNodes(!showAllowedNodes);
        }}
      >
        {serviceState.service.allowedNodes}
        {showAllowedNodes ? caretUp : caretDown}
      </button>
      <ul
        className={
          showAllowedNodes ? 'allowed-nodes-list show' : 'allowed-nodes-list'
        }
      >
        {nodes.map(node => {
          const selected =
            node.identity === serviceState.service.allowedNodes[0];
          return (
            <button
              key={`btn-${node.identity}`}
              type="button"
              className="allowed-nodes-item service-input"
              onClick={() => {
                setShowAllowedNodes(false);
                serviceDispatcher({
                  type: 'set-allowed-nodes',
                  allowedNodes: [node.identity]
                });
              }}
            >
              <div className="field-wrapper">
                <div className="field-header">Name</div>
                <div className="field-value">{node.displayName}</div>
              </div>
              <div className="field-wrapper">
                <div className="field-header">ID</div>
                <div className="field-value">{node.identity}</div>
                {localNodeID === node.identity && (
                  <div className="field-value">(Local)</div>
                )}
              </div>
              <FontAwesomeIcon
                icon="check"
                className={selected ? 'check-mark' : 'check-mark not-visible'}
              />
            </button>
          );
        })}
      </ul>
    </div>
  );

  const serviceArguments = () => {
    if (argumentsState.arguments.length === 0 && editMode) {
      argumentsDispatcher({ type: 'add-field' });
    }

    return argumentsState.arguments.map((arg, i) => {
      return (
        <div
          key={`args-${arg.key}`}
          className="arguments-input-wrapper flex-input"
        >
          <input
            className="service-input arguments-input"
            value={arg.key}
            placeholder="Key"
            disabled={!editMode}
            onChange={e => {
              const input = e.target.value;
              argumentsDispatcher({
                type: 'set-field-key',
                key: input,
                index: i
              });
            }}
          />
          <input
            className="service-input arguments-input"
            value={arg.value}
            placeholder="Value"
            disabled={!editMode}
            onChange={e => {
              const input = e.target.value;
              argumentsDispatcher({
                type: 'set-field-value',
                value: input,
                index: i
              });
            }}
          />
          {editMode && (
            <PlusButton
              actionFn={() => {
                argumentsDispatcher({
                  type: 'add-field'
                });
              }}
              display
            />
          )}
          {editMode && (
            <MinusButton
              actionFn={() => {
                argumentsDispatcher({
                  type: 'remove-field',
                  index: i
                });
              }}
              display={argumentsState.arguments.length > 1}
            />
          )}
          <div className="form-error">{argumentsState.errors[i]}</div>
        </div>
      );
    });
  };

  return (
    <div className="service-card">
      <div className="service-header bg-color-grey">
        {isEditable && !editMode && editButton}
        {isEditable && !editMode && deleteButton}
        {isEditable && editMode && confirmButton}
        {isEditable && editMode && dismissButton}
      </div>
      <div className="service-fields">
        <div className="service-field">
          <div className="field-name">Service type</div>
          <div className="field-input">
            <input
              className="service-input"
              value={serviceState.service.serviceType}
              onChange={e => {
                serviceDispatcher({
                  type: 'set-service-type',
                  serviceType: e.target.value
                });
              }}
              disabled={!editMode}
            />
            <div className="form-error">{serviceState.errors.serviceType}</div>
          </div>
        </div>
        <div className="service-field bg-color-grey">
          <div className="field-name">Service ID</div>
          <div className="field-input">
            <div className="flex-input">
              <input
                className="service-input service-id-input"
                value={serviceState.service.serviceId}
                onChange={e => {
                  serviceDispatcher({
                    type: 'set-service-id',
                    serviceId: e.target.value
                  });
                }}
                maxLength="4"
                disabled={!editMode}
              />
              {editMode && (
                <button
                  className="icon-btn"
                  type="button"
                  onClick={() => {
                    const id = generateServiceID();
                    serviceDispatcher({
                      type: 'set-service-id',
                      serviceId: id
                    });
                  }}
                  title="Generate service ID"
                >
                  <FontAwesomeIcon icon="sync-alt" />
                </button>
              )}
            </div>
            <div className="form-error">{serviceState.errors.serviceId}</div>
          </div>
        </div>
        <div className="service-field">
          <div className="field-name">Allowed node</div>
          <div className="field-input">
            {!editMode && allowedNodes}
            {editMode && allowedNodeEditMode}
            <div className="form-error">{serviceState.errors.allowedNodes}</div>
          </div>
        </div>
        <div className="service-field bg-color-grey">
          <div className="field-name">Service arguments</div>
          <div className="field-input service-arguments">
            {serviceArguments()}
          </div>
        </div>
      </div>
    </div>
  );
};

ServiceCard.propTypes = {
  service: PropTypes.instanceOf(Service).isRequired,
  applyServiceChanges: PropTypes.func,
  deleteService: PropTypes.func,
  enterEditMode: PropTypes.bool,
  isEditable: PropTypes.bool,
  isDeletable: PropTypes.bool,
  nodes: PropTypes.arrayOf(Node),
  localNodeID: PropTypes.string
};

ServiceCard.defaultProps = {
  applyServiceChanges: undefined,
  deleteService: undefined,
  enterEditMode: false,
  isEditable: false,
  isDeletable: true,
  nodes: [],
  localNodeID: ''
};

export default ServiceCard;
