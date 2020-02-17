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

import { NewGameroomProposal, Gameroom } from '@/store/models';
import { gameroomPropose, submitPayload, listGamerooms } from '@/store/api';
import { signPayload } from '@/utils/crypto';

export interface GameroomState {
  gamerooms: Gameroom[];
}

const gameroomState = {
  gamerooms: ([] as Gameroom[]),
};

const getters = {
  gameroomList(state: GameroomState): Gameroom[] {
    return state.gamerooms;
  },

  activeGameroomList(state: GameroomState): Gameroom[] {
    return state.gamerooms.filter((gameroom: Gameroom) => gameroom.status === 'Active');
  },
};

const actions = {
  async listGamerooms({ commit }: any) {
    const gamerooms = await listGamerooms();
    commit('setGamerooms', gamerooms);
  },

  async proposeGameroom({ rootGetters }: any, proposal: NewGameroomProposal) {
    const user = rootGetters['user/getUser'];
    try {
      const payload = await gameroomPropose(proposal);
      const signedPayload = signPayload(payload, user.privateKey);
      const response = await submitPayload(signedPayload);
      return response;
    } catch (err) {
      throw err;
    }
  },
};

const mutations = {
  setGamerooms(state: GameroomState, gamerooms: Gameroom[]) {
    state.gamerooms = gamerooms;
  },
};

export default {
  namespaced: true,
  name: 'gamerooms',
  state: gameroomState,
  getters,
  actions,
  mutations,
};
