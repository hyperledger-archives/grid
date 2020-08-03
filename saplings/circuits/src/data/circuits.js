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

import yaml from 'js-yaml';

import Paging from './paging';

/**
 * Convert the arguments value of either a proposal or a circuit to an object
 * containing the values parsed from JSON, if applicable.
 */
const argsToObject = coll => {
  let kvPairs = [];
  if (Array.isArray(coll)) {
    kvPairs = coll;
  } else if (typeof coll === 'object') {
    kvPairs = Object.entries(coll);
  } else {
    throw new Error(`Unsupported argument type: ${typeof coll}`);
  }

  return kvPairs.reduce((acc, [k, encodedVal]) => {
    try {
      acc[k] = JSON.parse(encodedVal);
    } catch (e) {
      acc[k] = encodedVal;
    }
    return acc;
  }, {});
};

const metadataFromJson = encoded => {
  let asString = encoded;
  try {
    const unencoded = Buffer.from(encoded, 'hex');
    asString = unencoded.toString();
    return JSON.parse(asString);
  } catch (jsonException) {
    // try yaml
    try {
      return yaml.safeLoad(asString);
    } catch (yamlException) {
      return encoded;
    }
  }
};

function Service(jsonSource) {
  if (!(this instanceof Service)) {
    return new Service(jsonSource);
  }

  if (jsonSource) {
    this.serviceId = jsonSource.service_id;
    this.serviceType = jsonSource.service_type;
    this.allowedNodes = jsonSource.allowed_nodes;
    this.arguments = argsToObject(jsonSource.arguments);
  } else {
    this.serviceId = '';
    this.serviceType = '';
    this.allowedNodes = [];
    this.arguments = {};
  }
}

function Circuit(data) {
  if (!(this instanceof Circuit)) {
    return new Circuit(data);
  }
  if (data.proposal_type) {
    this.id = data.circuit_id;
    this.status = 'Pending';
    this.members = data.circuit.members.map(member => {
      return member.node_id;
    });
    this.roster = data.circuit.roster.map(s => new Service(s));
    this.managementType = data.circuit.management_type;
    this.applicationMetadata = metadataFromJson(
      data.circuit.application_metadata
    );
    this.encodedApplicationData = data.circuit.application_metadata;
    this.comments = data.circuit.comments;
    this.proposal = {
      votes: data.votes,
      requester: data.requester,
      requesterNodeID: data.requester_node_id,
      proposalType: data.proposal_type,
      circuitHash: data.circuit_hash
    };
  } else {
    this.id = data.id;
    this.status = 'Active';
    this.members = data.members;
    this.roster = data.roster.map(s => new Service(s));
    this.managementType = data.management_type;
    this.applicationMetadata = metadataFromJson(data.application_metadata);
    this.encodedApplicationData = data.application_metadata;
    this.comments = 'N/A';
    this.proposal = {
      votes: [],
      requester: '',
      requesterNodeID: '',
      proposalType: '',
      circuitHash: ''
    };
  }
}

function ListCircuitsResponse(data) {
  this.data = data.data.map(item => {
    return new Circuit(item);
  });
  this.paging = new Paging(data);
}

function awaitingApproval() {
  return this.status === 'Pending';
}

function actionRequired(nodeId) {
  if (!this.awaitingApproval()) {
    return false;
  }

  if (this.proposal.requesterNodeID === nodeId) {
    return false;
  }

  return !this.proposal.votes.find(vote => vote.voter_node_id === nodeId);
}

function numUniqueServiceTypes() {
  return new Set(this.roster.map(service => service.serviceType)).size;
}

function listServiceTypesCount() {
  const count = {};
  this.roster.forEach(service => {
    if (count[service.serviceType]) {
      count[service.serviceType] += 1;
    } else {
      count[service.serviceType] = 1;
    }
  });
  return count;
}

Circuit.prototype.awaitingApproval = awaitingApproval;
Circuit.prototype.actionRequired = actionRequired;
Circuit.prototype.numUniqueServiceTypes = numUniqueServiceTypes;
Circuit.prototype.listServiceTypesCount = listServiceTypesCount;

const generateID = length => {
  let id = '';
  for (let i = 0; i < length; i += 1) {
    const capitalize = Math.random() >= 0.5;
    let idChar = Math.random()
      .toString(36)
      .substring(2, 3);
    if (capitalize) {
      idChar = idChar.toUpperCase();
    }
    id += idChar;
  }

  return id;
};

export { Circuit, ListCircuitsResponse, Service, generateID };
