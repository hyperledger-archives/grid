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

import React from 'react';
import PropTypes from 'prop-types';
import { Circuit } from '../../data/processCircuits';
import TableRow from './TableRow';
import TableHeader from './TableHeader';

import './CircuitsTable.scss';

const CircuitsTable = ({ circuits, dispatch }) => {
  return (
    <div className="table-container">
      <table className="circuits-table">
        <TableHeader dispatch={dispatch} circuits={circuits} />
        {circuits.map(item => {
          return <TableRow circuit={item} />;
        })}
      </table>
    </div>
  );
};

CircuitsTable.propTypes = {
  circuits: PropTypes.arrayOf(Circuit).isRequired,
  dispatch: PropTypes.func.isRequired
};

export default CircuitsTable;
