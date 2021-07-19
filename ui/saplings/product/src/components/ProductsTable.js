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

import React, { useState, useMemo, useEffect } from 'react';
import parser, {j2xParser as Parser} from 'fast-xml-parser';
import { matchSorter } from 'match-sorter';
import _ from 'lodash';
import { Chip, Chips } from './Chips';
import { Table } from './Table';
import { useServiceState } from '../state/service-context';
import { Input } from './Input';
import { listProducts } from '../api/grid';
import Placeholder from '../img_placeholder.svg';
import './ProductsTable.scss';

function ProductsTable() {
  const [products, setProducts] = useState([]);
  const [loading, setLoading] = useState(false);
  const { selectedService, selectedCircuit } = useServiceState();
  const [filterInputState, setFilterInputState] = useState({
    type: 'product name',
    value: ''
  });
  const initialFilterState = [
    {
      type:'product name',
      value: ''
    },
    {
      type: 'gtin',
      value: ''
    },
    {
      type: 'owner',
      value: ''
    }
  ]
  const [filterState, setFilterState] = useState(initialFilterState);

  const productXMLToJSON = product => {
    const productJSON = [];
    const xmlProperty = _.find(product.properties, ['name', 'GDSN_3_1']).string_value;
    try {
      productJSON.push(parser.parse(xmlProperty, {ignoreAttributes: false}, false));
    } catch (err) {
      console.error(err);
    }
    return productJSON;
  }

  useEffect(() => {
    const getProducts = async () => {
      if (selectedService !== 'none') {
        try {
          setLoading(true);
          const productList = await listProducts(selectedService);
          productList.map(p => productXMLToJSON(p))
          setProducts(productList.map(p => ({
            product: p,
            data: productXMLToJSON(p)
          })));
          setLoading(false);
        } catch (e) {
          console.error(`Error listing products: ${e}`);
        }
      } else {
        setProducts([]);
      }
    };

    getProducts();
  }, [selectedService]);

  const handleFilterInputChange = e => {
    const { name, value } = e.target;
    switch (name) {
      case 'filter-type':
        setFilterInputState({
          type: value,
          value: filterInputState.value
        });
        break;
      case 'filter-value':
        setFilterInputState({
          type: filterInputState.type,
          value
        });
        break;
      default:
        break;
    }
  };

  const handleAddFilter = () => {
      filterState.find(f => f.type === filterInputState.type).value = filterInputState.value;
      setFilterState([
        ...filterState
      ])

      setFilterInputState({
        type: filterInputState.type,
        value: ''
      })
  }

  const handleRemoveFilter = c => {
    filterState.find(f => f.type === c).value = '';
      setFilterState([
        ...filterState
      ])
  }

  // eslint-disable-next-line camelcase
  const downloadXMLGDSN3_1 = row => {
    const t = productXMLToJSON(row.original.product)[0];
    if (t) {
      const j2x = new Parser({format: true, ignoreAttributes: false});
      let xml = {
        gridTradeItems: {
          "@_xmlns:ns0": "urn:gs1:gdsn:food_and_beverage_ingredient:xsd:3",
          "@_xmlns:ns10": "urn:gs1:gdsn:trade_item_hierarchy:xsd:3",
          "@_xmlns:ns11": "urn:gs1:gdsn:trade_item_lifespan:xsd:3",
          "@_xmlns:ns12": "urn:gs1:gdsn:trade_item_measurements:xsd:3",
          "@_xmlns:ns13": "urn:gs1:gdsn:trade_item_temperature_information:xsd:3",
          "@_xmlns:ns2": "urn:gs1:gdsn:consumer_instructions:xsd:3",
          "@_xmlns:ns3": "urn:gs1:gdsn:food_and_beverage_preparation_serving:xsd:3",
          "@_xmlns:ns4": "urn:gs1:gdsn:marketing_information:xsd:3",
          "@_xmlns:ns5": "urn:gs1:gdsn:nutritional_information:xsd:3",
          "@_xmlns:ns6": "urn:gs1:gdsn:packaging_marking:xsd:3",
          "@_xmlns:ns7": "urn:gs1:gdsn:place_of_item_activity:xsd:3",
          "@_xmlns:ns8": "urn:gs1:gdsn:referenced_file_detail_information:xsd:3",
          "@_xmlns:ns9": "urn:gs1:gdsn:trade_item_description:xsd:3",
          "@_xmlns:xsi": "http://www.w3.org/2001/XMLSchema-instance",
          "@_xsi:noNamespaceSchemaLocation": "gridTradeItems.xsd",
          ...t
        }
      }
      xml = j2x.parse(xml);
      const filename = `${row.values.gtin}.xml`;
      const blob = new Blob([xml], { type: 'text/plain' });
      const el = document.createElement('a');
      el.setAttribute('href', window.URL.createObjectURL(blob));
      el.setAttribute('download', filename);
      el.dataset.downloadurl = ['text/plain', el.download, el.href].join(':');

      el.click();
    }
  }

  const fuzzyTextFilter = (rows, id, filterValue) => {
    return matchSorter(rows, filterValue, { keys: [row => row.values[id]] });
  }

  const accessProductImage = row => {
    const ns = row.data[0].tradeItem.tradeItemInformation.extension["ns8:referencedFileDetailInformationModule"];
    if (ns) {
      const img = _.find(ns, (r) => r.referencedFileTypeCode === "PRODUCT_IMAGE")
      if (img) {
        return img.uniformResourceIdentifier
      }
    }
    return undefined;
  }

  const accessProductName = row => {
    if (row.data[0].tradeItem.tradeItemInformation.extension["ns9:tradeItemDescriptionModule"]) {
      return row.data[0].tradeItem.tradeItemInformation.extension["ns9:tradeItemDescriptionModule"].tradeItemDescriptionInformation.regulatedProductName["#text"];
    }
    return undefined;
  }

  const filterTypes = useMemo(
    () => ({
      fuzzyText: fuzzyTextFilter,
    })
  )

  const filters = useMemo(
    () => filterState
  )

  const data = useMemo(
    () => products
  )

  const columns = useMemo(
    () => [
      {
        Header: 'GTIN',
        accessor: 'data[0].tradeItem.gtin',
        id: 'gtin',
        filter: 'fuzzyText'
      },
      {
        Header: "Image",
        accessor: accessProductImage,
        style: {
          textAlign: 'center',
        },
        id: 'image',
        /* eslint-disable react/prop-types, react/destructuring-assignment */
        Cell: props => { return (
          <div className="img-wrapper">
            <img src={props.value ? props.value : Placeholder} alt={`${props.row.values.gtin} thumbnail`} className="product-image" />
          </div>
        )}
        /* eslint-enable react/prop-types, react/destructuring-assignment */
      },
      {
        Header: 'Product Name',
        accessor: accessProductName,
        id: 'product name',
        // eslint-disable-next-line react/prop-types, react/destructuring-assignment
        Cell: props => <a href={`/product/${props.row.values.gtin}`}>{props.value}</a>,
        filter: 'fuzzyText'
      },
      {
        Header: 'Owner',
        accessor: 'product.orgName',
        filter: 'fuzzyText',
        id: 'owner'
      },
    ]
  )

  return (
    <div className="products-table-container">
      <h1 className="selected-service">{selectedCircuit === 'none' ? 'No circuit selected' : selectedCircuit}</h1>
      <div className="table-utils">
        <div className="util-wrapper">
          <div className="filters">
            <div className="inputs">
              <Input type="select" icon="filter_list" label="Filter By" name="filter-type" value={filterInputState.type} onChange={handleFilterInputChange}>
                <option value="product name" default>
                  Product name
                </option>
                <option value="gtin">GTIN</option>
                <option value="owner">Owner</option>
              </Input>
              <Input type="text" icon="search" label="Product Name" name="filter-value" value={filterInputState.value} onChange={handleFilterInputChange} />
              <button className="btn-primary" onClick={handleAddFilter} type="button">Add Filter</button>
            </div>
            <div className="filter-list">
              {filterState.some(f => f.value !== '') &&
                <>
                  <Chips>
                    {
                      filterState.map(({type, value}) => (value !== '' && <Chip deleteable key={type} label={`${type}: ${value}`} removeFn={() => handleRemoveFilter(type)} />))
                    }
                  </Chips>
                  <button className="btn-primary btn-min" onClick={() => setFilterState(initialFilterState)} type="button">Clear Filters</button>
                </>
              }
            </div>
          </div>
          <div className="actions" />
        </div>
      </div>
      <div className="table">
        <Table columns={columns} data={data} filterTypes={filterTypes} filters={filters} actions={[{action: downloadXMLGDSN3_1, icon: 'code'}]} loading={loading} />
      </div>
    </div>
  );
}

export default ProductsTable;
