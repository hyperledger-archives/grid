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

import { User } from '@/store/models';
import { userAuthenticate, userCreate } from '@/store/api';
import * as crypto from '@/utils/crypto';

export interface UserState {
  user: User;
}

const userState = {
  user: {
    email: '',
    publicKey: '',
    privateKey: '',
  },
};

interface Creds {
  email: string;
  password: string;
}

const getters = {
  getUser(state: UserState) {
    return state.user;
  },
  getPublicKey(state: UserState) {
    return state.user.publicKey;
  },
  isLoggedIn(state: UserState) {
    return state.user.privateKey !== '';
  },
};

const actions = {
  async register({ commit }: any, creds: Creds) {
    const keys = crypto.createKeyPair();
    await userCreate({
      email: creds.email,
      hashedPassword: crypto.hashSHA256(creds.email, creds.password),
      publicKey: keys.publicKey,
      encryptedPrivateKey: crypto.encrypt(creds.password, keys.privateKey),
    });
    commit('setUser', {
      email: creds.email,
      publicKey: keys.publicKey,
      privateKey: keys.privateKey,
    });
  },
  async authenticate({ commit }: any, creds: Creds) {
    const hashedPassword = crypto.hashSHA256(creds.email, creds.password);
    const user = await userAuthenticate({email: creds.email, hashedPassword});
    const privateKey = crypto.decrypt(creds.password, user.encryptedPrivateKey);
    commit('setUser', {
      email: creds.email,
      publicKey: user.publicKey,
      privateKey,
    });
    return user;
  },
};

const mutations = {
  setUser(state: UserState, user: User) {
    state.user = user;
  },
  clearUser(state: UserState) {
    state.user = {
      email: '',
      publicKey: '',
      privateKey: '',
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
