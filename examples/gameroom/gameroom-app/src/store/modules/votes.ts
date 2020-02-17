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

export interface VoteState {
  votes: { [key: string]: boolean};
}

const voteState = {
  votes: {},
};

const getters = {
  voteList(state: VoteState) {
    return state.votes;
  },
};

const actions = {
  vote({ commit }: any, id: number) {
    commit('setVote', id);
  },
};

const mutations = {
  setVote(state: VoteState, id: number) {
    state.votes[id] = true;
  },
};

export default {
  namespaced: true,
  name: 'votes',
  state: voteState,
  getters,
  actions,
  mutations,
};
