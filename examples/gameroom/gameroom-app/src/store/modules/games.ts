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
import { listGames, submitBatch } from '@/store/api';
import { createTransaction, createBatch } from '@/utils/transactions';
import { calculateGameAddress } from '@/utils/addressing';


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
  async createGame({ commit, rootGetters }: any, {gameName, circuitID}: any) {
     const user = rootGetters['user/getUser'];
     const payload = new Buffer(`${gameName},create,`, 'utf-8');
     const gameAdress = calculateGameAddress(gameName);
     const transaction = createTransaction(payload, [gameAdress], [gameAdress], user);
     const batchBytes = createBatch([transaction], user);
     try {
       const response = submitBatch(batchBytes, circuitID);
       return response;
     } catch (err) {
       throw err;
     }
  },
  async take({ commit, rootGetters }: any, {gameName, cellIndex, circuitID}: any) {
    const user = rootGetters['user/getUser'];
    const payload = new Buffer(`${gameName},take,${cellIndex + 1}`, 'utf-8');
    const gameAdress = calculateGameAddress(gameName);
    const transaction = createTransaction(payload, [gameAdress], [gameAdress], user);
    const batchBytes = createBatch([transaction], user);
    try {
      const response = await submitBatch(batchBytes, circuitID);
      commit('setPendingTake', {gameName, cellIndex});
      return response;
    } catch (err) {
      throw err;
    }
  },
};

const mutations = {
  setGames(state: GameState, games: Game[]) {
    state.games = games;
  },
  setPendingTake(state: GameState, {gameName, cellIndex}: any) {
    const index = state.games.findIndex((g) => g.game_name === gameName);
    if (index !== -1) {
      const update = state.games[index];
      const gameBoard = update.game_board;
      update.game_board = `${gameBoard.substr(0, cellIndex)}?${gameBoard.substr(cellIndex + 1)}`;
      state.games.splice(index, 1, update);
    }
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
