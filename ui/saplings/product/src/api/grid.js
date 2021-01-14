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

import { getSharedConfig } from 'splinter-saplingjs';

import { get } from './requests';

export const ProductProtocolVersion = '1';

const { gridURL } = getSharedConfig().appConfig;

const getOrgName = async (orgID, serviceID) => {
  const result = await get(
    `${gridURL}/organization/${orgID}?service_id=${serviceID}`
  );

  if (result.ok) {
    return result.json.name;
  }
  throw Error(result.data);
};

const getOrgNames = (products, serviceID) => {
  return Promise.all(
    products.map(async product => {
      const orgName = await getOrgName(product.owner, serviceID);
      return { ...product, orgName };
    })
  );
};

export const listProducts = async serviceID => {
  const result = await get(`${gridURL}/product?service_id=${serviceID}`,
    ProductProtocolVersion
  );

  if (result.ok) {
    const products = await getOrgNames(result.json, serviceID);
    return products;
  }
  throw Error(result.data);
};

export const fetchProduct = async (serviceID, productID) => {
  const result = await get(
    `${gridURL}/product/${productID}?service_id=${serviceID}`,
    ProductProtocolVersion
  );

  if (result.ok) {
    const product = result.json;
    const orgName = await getOrgName(product.owner, serviceID);
    return { ...product, orgName };
  }
  throw Error(result.data);
};
