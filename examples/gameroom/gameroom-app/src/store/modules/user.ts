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

import { User, UserCredentials } from '@/store/models';
import { userAuthenticate, userCreate } from '@/store/api';

export interface UserState {
  user: User;
}

const userState = {
  user: {
    email: '',
    hashedPassword: '',
    publicKey: '',
    encryptedPrivateKey: '',
  },
};

const getters = {
  isLoggedIn(state: UserState) {
    return state.user.encryptedPrivateKey !== '';
  },
};

const actions = {
  async register({ commit }: any, userInfo: User) {
    const user = await userCreate(userInfo);
    commit('setUser', user);
  },
  async authenticate({ commit }: any, credentials: UserCredentials) {
    const user = await userAuthenticate(credentials);
    commit('setUser', user);
  },
};

const mutations = {
  setUser(state: UserState, user: User) {
    state.user = user;
  },
  clearUser(state: UserState) {
    state.user = {
      email: '',
      hashedPassword: '',
      publicKey: '',
      encryptedPrivateKey: '',
    };
  },
};

export default {
  namespaced: true,
  state: userState,
  getters,
  actions,
  mutations,
};
