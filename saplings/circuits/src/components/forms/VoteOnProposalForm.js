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
import { useToasts } from 'react-toast-notifications';
import { useHistory } from 'react-router-dom';

import ProposalReview from '../ProposalReview';
import { Circuit } from '../../data/circuits';
import { Node } from '../../data/nodeRegistry';
import protos from '../../protobuf';
import { makeSignedPayload } from '../../api/payload';
import { postCircuitManagementPayload } from '../../api/splinter';
import { useLocalNodeState } from '../../state/localNode';

import './VoteOnProposalForm.scss';

const VoteOnProposalForm = ({ proposal, nodes, closeFn }) => {
  const { addToast } = useToasts();
  const history = useHistory();

  const localNodeID = useLocalNodeState();

  if (proposal === null || nodes === null) {
    return <div />;
  }

  const handleVote = async vote => {
    let voteEnum = null;
    let redirect = false;
    switch (vote) {
      case 'accept':
        voteEnum = protos.CircuitProposalVote.Vote.ACCEPT;
        break;
      case 'reject':
        voteEnum = protos.CircuitProposalVote.Vote.REJECT;
        redirect = true;
        break;
      default:
        throw new Error(`invalid vote: ${vote}`);
    }

    const votePayload = protos.CircuitProposalVote.create({
      circuitId: proposal.id,
      circuitHash: proposal.proposal.circuitHash,
      vote: voteEnum
    });

    const { privateKey } = window.$CANOPY.getKeys();

    const payload = makeSignedPayload(
      localNodeID,
      privateKey,
      votePayload,
      'voteCircuitProposal'
    );

    try {
      await postCircuitManagementPayload(payload);
      addToast('Vote submitted successfully', {
        appearance: 'success'
      });
      closeFn();
      if (redirect) {
        history.push(`/circuits`);
      }
    } catch (e) {
      addToast(`${e}`, { appearance: 'error' });
    }
  };

  return (
    <div className="vote-on-proposal-form">
      <div className="form-header">
        <div className="form-title">Vote on circuit proposal</div>
        <div className="help-text">
          Review the circuit information below and submit your vote.
        </div>
      </div>
      <div className="proposal-review-wrapper">
        <ProposalReview
          members={nodes}
          services={proposal.roster}
          comments={proposal.comments}
          metadata={{ metadata: proposal.applicationMetadata }}
          managementType={proposal.managementType}
        />
      </div>
      <div className="form-footer">
        <button className="form-button" type="button" onClick={closeFn}>
          Cancel
        </button>
        <div className="vote-buttons">
          <button
            className="form-button reject"
            type="button"
            onClick={() => {
              handleVote('reject');
            }}
          >
            Reject
          </button>
          <button
            className="form-button accept"
            type="button"
            onClick={() => {
              handleVote('accept');
            }}
          >
            Accept
          </button>
        </div>
      </div>
    </div>
  );
};

VoteOnProposalForm.propTypes = {
  proposal: PropTypes.instanceOf(Circuit).isRequired,
  nodes: PropTypes.arrayOf(Node).isRequired,
  closeFn: PropTypes.func.isRequired
};

export default VoteOnProposalForm;
