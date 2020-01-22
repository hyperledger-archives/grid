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

import { VuexModule, Module, getModule, Action, Mutation } from 'vuex-module-decorators';
import store from '@/store';
import { NewGameroomProposal, Gameroom } from '@/store/models';
import { gameroomPropose, submitPayload, listGamerooms } from '@/store/api';
import { signPayload } from '@/utils/crypto';

@Module({
  namespaced: true,
  name: 'gamerooms',
  store,
  dynamic: true,
})
class GameroomsModule extends VuexModule {
  gamerooms: Gameroom[] = [];

  @Mutation
  setGamerooms(gamerooms: Gameroom[]) { this.gamerooms = gamerooms; }

  get gameroomList(): Gameroom[] {
    return this.gamerooms;
  }

  get activeGameroomList(): Gameroom[] {
    return this.gamerooms.filter(
      (gameroom: Gameroom) => gameroom.status === 'Active');
  }

  @Action({ commit: 'setGamerooms' })
  async listGamerooms() {
    const gamerooms = await listGamerooms();
    return gamerooms;
  }

  @Action({ rawError: true })
  async proposeGameroom(proposal: NewGameroomProposal) {
    const user = this.context.rootGetters['user/getUser'];
    try {
      const payload = await gameroomPropose(proposal);
      const signedPayload = signPayload(payload, user.privateKey);
      const response = await submitPayload(signedPayload);
      return response;
    } catch (err) {
      throw err;
    }
  }
}
export default getModule(GameroomsModule);
