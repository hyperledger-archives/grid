// Copyright 2019 Cargill Incorporated
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

import { VuexModule, Module, getModule, Action, Mutation } from 'vuex-module-decorators';
import store from '@/store';
import { GameroomProposal, Ballot } from '@/store/models';
import { listProposals, proposalVote, submitPayload } from '@/store/api';
import { signPayload } from '@/utils/crypto';

interface Vote {
  proposalID: string;
  ballot: Ballot;
}

@Module({
  namespaced: true,
  name: 'proposals',
  store,
  dynamic: true,
})
class ProposalsModule extends VuexModule {
  proposals: GameroomProposal[] = [];

  @Mutation
  setProposals(proposals: GameroomProposal[]) { this.proposals = proposals;  }

  @Action({ rawError: true })
  async vote(vote: Vote) {
    const user = this.context.rootGetters['user/getUser'];
    try {
      const payload = await proposalVote(vote.ballot, vote.proposalID);
      const signedPayload = signPayload(payload, user.privateKey);
      const response = await submitPayload(signedPayload);
      return response;
    } catch (e) {
      console.error(e);
      throw e;
    }
  }

  @Action({ commit: 'setProposals' })
  async listProposals() {
    const proposals = await listProposals();
    return proposals;
  }

  get proposalList() {
    return this.proposals;
  }
}
export default getModule(ProposalsModule);
