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
import { GameroomProposal } from '@/store/models';
import { listProposals } from '@/store/api';
import nodes from './nodes';

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

  @Action({ commit: 'setProposals' })
  async listProposals() {
    const proposals = await listProposals();
    return proposals;
  }

  @Action({ commit: 'setProposals' })
  listProposalsMock() {
    const proposals = [
      {
        name: 'acme_corp:bubba_bakery',
        members: [
          'bubba_bakery', 'acme_corp',
        ],
        requester: 'acme_corp',
        created_time: 1564772396,
        updated_time: 0,
      },
      {
        name: 'asdforg:bubba_bakery',
        members: [
          'asdforg', 'acme_corp',
        ],
        requester: 'asdforg',
        created_time: 1564772439,
        updated_time: 0,
      },
    ];
    return proposals;
  }

  get proposalList() {
    return this.proposals;
  }
}
export default getModule(ProposalsModule);
