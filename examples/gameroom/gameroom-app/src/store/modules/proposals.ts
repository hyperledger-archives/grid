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

  @Action({ commit: 'setProposals' })
  listProposalsMock() {
    const proposals: GameroomProposal[] = [
      {
        proposal_id: 'proposal1',
        circuit_id: 'gameroom::acme-node-000::bubba-node-000::a57747f7-6a54-4d74-8a7a-7029b71b59f4',
        circuit_hash: '8e066d41911817a42ab098eda35a2a2b11e93c753bc5ecc3ffb3e99ed99ada0d',
        members: [
          {
            node_id: 'acme-node-000',
            endpoint: 'tls://splinterd-node-acme:8044',
            organization: 'ACME Corporation'
          },
          {
            node_id: 'bubba-node-000',
            endpoint: 'tls://splinterd-node-bubba:8044',
            organization: 'Bubba Bakery'
          },
        ],
        requester: '395acb89a89835ffd4ecaf92baeb83b74eea6e5ade10a5c570debfd12a772baa87',
        created_time: 1565732000,
        updated_time: 1565732000,
      },
      {
        proposal_id: 'proposal2',
        circuit_id: 'gameroom::acme-node-000::bubba-node-000::a57747f7-6a54-4d74-8a7a-7029b71b59f4',
        circuit_hash: '8e066d41911817a42ab098eda35a2a2b11e93c753bc5ecc3ffb3e99ed99ada0d',
        members: [
          {
            node_id: 'acme-node-000',
            endpoint: 'tls://splinterd-node-acme:8044',
            organization: 'ACME Corporation'
          },
          {
            node_id: 'bubba-node-000',
            endpoint: 'tls://splinterd-node-bubba:8044',
            organization: 'Bubba Bakery'
          },
        ],
        requester: '03473bfa98097f3e09b1d929c7830419ba372638af4c67ea23ac3e1f616fd85e9d',
        created_time: 1565732000,
        updated_time: 1565732000,
      },
      {
        proposal_id: 'proposal3',
        circuit_id: 'gameroom::acme-node-000::bubba-node-000::a57747f7-6a54-4d74-8a7a-7029b71b59f4',
        circuit_hash: '8e066d41911817a42ab098eda35a2a2b11e93c753bc5ecc3ffb3e99ed99ada0d',
        members: [
          {
            node_id: 'acme-node-000',
            endpoint: 'tls://splinterd-node-acme:8044',
            organization: 'ACME Corporation'
          },
          {
            node_id: 'bubba-node-000',
            endpoint: 'tls://splinterd-node-bubba:8044',
            organization: 'Bubba Bakery'
          },
        ],
        requester: '395acb89a89835ffd4ecaf92baeb83b74eea6e5ade10a5c570debfd12a772baa87',
        created_time: 1565732000,
        updated_time: 1565732000,
      },
    ];
    return proposals;
  }

  get proposalList() {
    return this.proposals;
  }
}
export default getModule(ProposalsModule);
