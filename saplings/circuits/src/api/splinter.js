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

import { getSharedConfig } from 'splinter-saplingjs';
import { get, post } from './requests';
import { NodeRegistryResponse } from '../data/nodeRegistry';

const { splinterURL } = getSharedConfig().canopyConfig;

export const getNodeID = async () => {
  const result = await get(`${splinterURL}/status`);

  if (result.ok) {
    return result.json.node_id;
  }
  throw Error(result.data);
};

export const listProposals = async () => {
  const result = await get(`${splinterURL}/admin/proposals`);

  if (result.ok) {
    return result.json;
  }
  throw Error(result.data);
};

export const getProposal = async circuitId => {
  const result = await get(`${splinterURL}/admin/proposals/${circuitId}`);

  if (result.ok) {
    return result.json;
  }
  throw Error(result.data);
};

export const listCircuits = async () => {
  const result = await get(`${splinterURL}/admin/circuits`);

  if (result.ok) {
    return result.json;
  }
  throw Error(result.data);
};

export const getCircuit = async circuitId => {
  const result = await get(`${splinterURL}/admin/circuits/${circuitId}`);

  if (result.ok) {
    return result.json;
  }
  throw Error(result.data);
};

export const getNodeRegistry = async () => {
  const result = await get(`${splinterURL}/registry/nodes`);

  if (result.ok) {
    const response = new NodeRegistryResponse(result.json);
    return response.data;
  }
  throw Error(result.data);
};

export const postNode = async node => {
  const setHeader = request => {
    request.setRequestHeader('Content-Type', 'application/json');
  };

  const result = await post(
    `${splinterURL}/registry/nodes`,
    JSON.stringify(node),
    setHeader
  );

  if (result.ok) {
    return node;
  }
  throw Error(result.json.message);
};
