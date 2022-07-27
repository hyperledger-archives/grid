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

import React from 'react';
import PropTypes from 'prop-types';

import { ServiceTable } from 'App/components/forms/propose_circuit/service';
import NodeCard from './NodeCard';

import { useLocalNodeState } from '../state/localNode';

import { Service } from '../data/circuits';
import { Node } from '../data/nodeRegistry';

import './ProposalReview.scss';

const ProposalReview = ({
  members,
  services,
  managementType,
  comments,
  metadata,
  displayName
}) => {
  const localNodeID = useLocalNodeState();
  return (
    <div className="proposal-review-container">
      <div className="nodes-container">
        <div className="title">Nodes</div>
        <div className="member-nodes content-container">
          {members.map(node => {
            const local = node.identity === localNodeID;
            return (
              <div key={`member-${node.identity}`} className="member-node">
                <NodeCard node={node} isLocal={local} isSelectable={false} />
              </div>
            );
          })}
        </div>
      </div>
      <div class-name="services-container">
        <div className="title">Services</div>
        <div className="services">
          <ServiceTable services={services} nodes={members} />
        </div>
      </div>
      <div className="details-container">
        <div className="title">Details</div>
        <div className="label">Display Name</div>
        <div className="field-value">{displayName}</div>
        <div className="label">Management type</div>
        <div className="field-value">{managementType}</div>
        <div className="label">Comments</div>
        <div className="field-value">{comments}</div>
      </div>
      <div className="metadata-container">
        <div className="title">Application metadata</div>
        {metadata.encoding && (
          <div>
            <div className="label">Encoding</div>
            <div className="field-value">{metadata.encoding}</div>
          </div>
        )}

        <div className="label">Value</div>
        <div className="field-value">{metadata.metadata}</div>
      </div>
    </div>
  );
};

ProposalReview.propTypes = {
  members: PropTypes.arrayOf(PropTypes.instanceOf(Node)).isRequired,
  services: PropTypes.arrayOf(PropTypes.instanceOf(Service)).isRequired,
  comments: PropTypes.string.isRequired,
  managementType: PropTypes.string.isRequired,
  metadata: PropTypes.shape({
    metadata: PropTypes.string,
    encoding: PropTypes.string
  }).isRequired,
  displayName: PropTypes.string.isRequired
};

ProposalReview.defaultProps = {};

export default ProposalReview;
