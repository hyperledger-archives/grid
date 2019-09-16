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

import { Game } from '@/store/models';
import { listGames } from '@/store/api';

export interface GameState {
  games: Game[];
}

const gameState = {
  games: ([] as Game[]),
};

const getters = {
  getGames(state: GameState): Game[] {
    return state.games.sort(
      (a: Game, b: Game) => {
        return (b.updated_time - a.updated_time);  // Newest first
      },
    );
  },
};

const actions = {
  async listGames({ commit }: any, circuitID: string) {
     const games = await listGames(circuitID);
     commit('setGames', games);
  },
};

const mutations = {
  setGames(state: GameState, games: Game[]) {
    state.games = games;
  },
};

export default {
  namespaced: true,
  name: 'games',
  state: gameState,
  getters,
  actions,
  mutations,
};
