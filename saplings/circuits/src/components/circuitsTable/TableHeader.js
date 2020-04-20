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

import React, { useState, useEffect, useReducer } from 'react';
import PropTypes from 'prop-types';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';

import { useLocalNodeState } from '../../state/localNode';
import { Circuit } from '../../data/processCircuits';

const filtersReducer = (state, action) => {
  switch (action.type) {
    case 'show': {
      // reset any filter options that were not applied
      const stageActionRequired = state.actionRequired;
      const stageAwaitingApproval = state.awaitingApproval;
      return {
        ...state,
        show: !state.show,
        stageActionRequired,
        stageAwaitingApproval
      };
    }
    case 'stage': {
      const { stageActionRequired, stageAwaitingApproval } = action;
      return { ...state, stageActionRequired, stageAwaitingApproval };
    }
    case 'apply': {
      const actionRequired = state.stageActionRequired;
      const awaitingApproval = state.stageAwaitingApproval;

      action.dispatch({
        type: 'filterByStatus',
        filter: {
          awaitingApproval,
          actionRequired,
          nodeID: action.nodeID
        }
      });
      return { ...state, actionRequired, awaitingApproval, show: false };
    }
    default:
      throw new Error(`unhandled action type: ${action.type}`);
  }
};

const TableHeader = ({ dispatch, circuits }) => {
  const nodeID = useLocalNodeState();
  const [sortedBy, setSortedBy] = useState({
    ascendingOrder: false,
    field: ''
  });
  const sortCircuitsBy = (sortBy, order) => {
    setSortedBy({ ascendingOrder: order, field: sortBy });
    dispatch({
      type: 'sort',
      sort: { sortBy, ascendingOrder: order }
    });
  };

  useEffect(() => {
    sortCircuitsBy(sortedBy.field, sortedBy.ascendingOrder);
  }, [circuits]);

  const caretDown = (
    <span className="caret">
      <FontAwesomeIcon icon="caret-down" />
    </span>
  );

  const caretUp = (
    <span className="caret">
      <FontAwesomeIcon icon="caret-up" />
    </span>
  );

  const sortableSymbol = (
    <span className="caret">
      <FontAwesomeIcon icon="sort" />
    </span>
  );

  const filterSymbol = selected => {
    return (
      <span className={selected ? 'caret text-highlight' : 'caret'}>
        <FontAwesomeIcon icon="filter" />
      </span>
    );
  };

  const exclamationCircle = (
    <span className="status-icon action-required">
      <FontAwesomeIcon icon="exclamation-circle" />
    </span>
  );

  const businessTime = (
    <span className="status-icon awaiting-approval">
      <FontAwesomeIcon icon="business-time" />
    </span>
  );

  const checkMark = hidden => {
    return (
      <span className={hidden ? 'status-icon hidden' : 'status-icon'}>
        <FontAwesomeIcon icon="check" />
      </span>
    );
  };

  const sortSymbol = fieldType => {
    if (sortedBy.field !== fieldType) {
      return sortableSymbol;
    }
    if (sortedBy.ascendingOrder) {
      return caretUp;
    }
    return caretDown;
  };

  const [filterSettings, setFilterSettings] = useReducer(filtersReducer, {
    show: false,
    actionRequired: false,
    awaitingApproval: false,
    stageActionRequired: false,
    stageAwaitingApproval: false
  });

  const filterOptions = (
    <div className={filterSettings.show ? 'filterStatus show' : 'filterStatus'}>
      <div className="statusOptions">
        <button
          className="filterOption"
          type="button"
          onClick={() => {
            setFilterSettings({
              type: 'stage',
              stageActionRequired: !filterSettings.stageActionRequired,
              stageAwaitingApproval: filterSettings.stageAwaitingApproval
            });
          }}
        >
          {exclamationCircle}
          Action required
          {checkMark(!filterSettings.stageActionRequired)}
        </button>
        <button
          className="filterOption"
          type="button"
          onClick={() => {
            setFilterSettings({
              type: 'stage',
              stageActionRequired: filterSettings.stageActionRequired,
              stageAwaitingApproval: !filterSettings.stageAwaitingApproval
            });
          }}
        >
          {businessTime}
          Awaiting approval
          {checkMark(!filterSettings.stageAwaitingApproval)}
        </button>
        <button
          type="button"
          className="apply-filter-btn"
          onClick={() => {
            setFilterSettings({
              type: 'apply',
              dispatch,
              nodeID
            });
          }}
        >
          Apply filter
        </button>
      </div>
    </div>
  );

  return (
    <tr className="table-header">
      <th onClick={() => sortCircuitsBy('circuitID', !sortedBy.ascendingOrder)}>
        Circuit ID
        {sortSymbol('circuitID')}
      </th>
      <th
        onClick={() => sortCircuitsBy('serviceCount', !sortedBy.ascendingOrder)}
      >
        Service count
        {sortSymbol('serviceCount')}
      </th>
      <th
        onClick={() => {
          sortCircuitsBy('managementType', !sortedBy.ascendingOrder);
        }}
      >
        Management Type
        {sortSymbol('managementType')}
      </th>
      <th onClick={() => sortCircuitsBy('comments', !sortedBy.ascendingOrder)}>
        Comments
        {sortSymbol('comments')}
      </th>
      <th>
        <div className="status-dropdown">
          <button
            type="button"
            onClick={() => {
              setFilterSettings({
                type: 'show'
              });
            }}
          >
            Status
            {filterSymbol(
              filterSettings.actionRequired || filterSettings.awaitingApproval
            )}
          </button>
          {filterOptions}
        </div>
      </th>
    </tr>
  );
};

TableHeader.propTypes = {
  dispatch: PropTypes.func.isRequired,
  circuits: PropTypes.arrayOf(Circuit).isRequired
};

export default TableHeader;
