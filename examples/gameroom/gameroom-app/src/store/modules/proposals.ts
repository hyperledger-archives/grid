// Copyright 2018-2020 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { GameroomProposal, Ballot } from '@/store/models';
import { listProposals, proposalVote, submitPayload } from '@/store/api';
import { signPayload } from '@/utils/crypto';

interface Vote {
  proposalID: string;
  ballot: Ballot;
}

export interface ProposalState {
  proposals: GameroomProposal[];
}

const proposalState = {
  proposals: ([] as GameroomProposal[]),
};

const getters = {
  getProposalList(state: ProposalState) {
    return proposalState.proposals;
  },
};

const actions = {
  async vote({ dispatch, rootGetters }: any, vote: Vote) {
    const user = rootGetters['user/getUser'];
    try {
      const payload = await proposalVote(vote.ballot, vote.proposalID);
      const signedPayload = signPayload(payload, user.privateKey);
      await dispatch('votes/vote', vote.proposalID, {root: true});
      const response = await submitPayload(signedPayload);
      return response;
    } catch (err) {
      console.error(err);
      throw err;
    }
  },
  async listProposals({ commit }: any) {
    const proposals = await listProposals();
    commit('setProposals', proposals);
  },
};

const mutations = {
  setProposals(state: ProposalState, proposals: GameroomProposal[]) {
    proposalState.proposals = proposals;
  },
};

export default {
  namespaced: true,
  name: 'proposals',
  state: proposalState,
  getters,
  actions,
  mutations,
};
