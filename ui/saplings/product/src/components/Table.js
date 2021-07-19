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
import React, {useEffect} from 'react'
import PropTypes from 'prop-types';
import Icon from '@material-ui/core/Icon';
import CircularProgress from '@material-ui/core/CircularProgress';
import { useTable, useFilters, usePagination } from 'react-table';
import { Input } from './Input';
import './Table.scss';

export function Table({ columns, data, filterTypes, filters, actions, loading }) {
  const {
    getTableProps,
    getTableBodyProps,
    headerGroups,
    prepareRow,
    page,
    canPreviousPage,
    canNextPage,
    nextPage,
    previousPage,
    setPageSize,
    setFilter,
    state: { pageIndex, pageSize },
  } = useTable(
    {
      columns,
      data,
      initialState: { pageIndex: 0 },
      filterTypes,
    },
    useFilters,
    usePagination,
  )

  useEffect(() => {
    filters.map(({type, value}) => {
      setFilter(type, value);
      return null;
    })
  }, [filters]);

  const handleRowsShownChange = e => {
    const {value} = e.target;
    setPageSize(Number(value));
  }

  const Pagination = () => (
    <div className="pagination">
      <span className="results">{data.length} Results</span>
      <div className="rpp">
        <span className="label">Rows per page: </span>
        <Input className="value" type="select" value={pageSize} onChange={e => handleRowsShownChange(e)}>
          <option value="10">10</option>
          <option value="25">25</option>
          <option value="50">50</option>
        </Input>
      </div>
      <span className="range">{`${pageIndex + 1}-${Math.min(data.length, (pageIndex +1) * pageSize)} of ${data.length}`}</span>
      <Icon className="page-step" onClick={() => previousPage()} disabled={!canPreviousPage}>
        chevron_left
      </Icon>
      <Icon className="page-step" onClick={() => nextPage()} disabled={!canNextPage}>
        chevron_right
      </Icon>
    </div>
  );

  return (
    <>
      {loading ? <CircularProgress className="loading" size="1.5rem" color="inherit" /> : <Pagination />}
      <div className="table-wrapper">
        <table {...getTableProps()}>
          <thead>
            {headerGroups.map(headerGroup => (
              <tr {...headerGroup.getHeaderGroupProps()}>
                {headerGroup.headers.map(column => (
                  <th {...column.getHeaderProps([
                    {
                      className: column.className,
                      style: column.style,
                    }
                  ])}>{column.render('Header')}</th>
                ))}
                <th>Action</th>
              </tr>
            ))}
          </thead>
          <tbody {...getTableBodyProps()}>
            {page.map(row => {
              prepareRow(row)
              return (
                <tr {...row.getRowProps()}>
                  {row.cells.map(cell => {
                    return <td {...cell.getCellProps()}>{cell.render('Cell')}</td>
                  })}
                  { actions.length > 0 &&
                    <td>
                      <div className="action-button-wrapper">
                        <div className="action-button">
                          <Icon>more_horiz_icon</Icon>
                          <span className="action-options">
                            {actions.map(a =>
                                <button
                                  className="key-action-btn"
                                  title="Download XML"
                                  type="button"
                                  onClick={e => {
                                    e.preventDefault();
                                    a.action(row);
                                  }}
                                >
                                  <Icon>{a.icon}</Icon>
                              </button>
                            )}
                          </span>
                        </div>
                      </div>
                    </td>
                  }
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
      {!loading && <Pagination />}
    </>
  )
}

Table.propTypes = {
  columns: PropTypes.array.isRequired,
  data: PropTypes.array.isRequired,
  filterTypes: PropTypes.object,
  filters: PropTypes.array,
  actions: PropTypes.arrayOf(
    PropTypes.shape({
      action: PropTypes.func,
      icon: PropTypes.string,
    })
  ),
  loading: PropTypes.bool
}

Table.defaultProps = {
  filterTypes: {},
  filters: [],
  actions: [],
  loading: false
}
