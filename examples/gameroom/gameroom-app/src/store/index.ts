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

import Vue from 'vue';
import Vuex from 'vuex';
import userModule from '@/store/modules/user';
import notificationsModule from '@/store/modules/notifications';
import selectedGameroomModule from '@/store/modules/selectedGameroom';
import votesModule from '@/store/modules/votes';
import gamesModule from '@/store/modules/games';
import proposalsModule from '@/store/modules/proposals';
import pageLoadingModule from '@/store/modules/pageLoading';

import VuexPersistence from 'vuex-persist';

Vue.use(Vuex);

const vuexLocal = new VuexPersistence({
  storage: window.localStorage,
  reducer: (state: any) => ({ user: state.user }),
});

export default new Vuex.Store({
  modules: {
    user: userModule,
    notifications: notificationsModule,
    votes: votesModule,
    games: gamesModule,
    selectedGameroom: selectedGameroomModule,
    proposals: proposalsModule,
    pageLoading: pageLoadingModule,
  },
  plugins: [vuexLocal.plugin],
  state: {
    socket: {
      isConnected: false,
      message: '',
      reconnectError: false,
    },
  },
  mutations: {
    SOCKET_ONOPEN(state, event)  {
      Vue.prototype.$socket = event.currentTarget;
      state.socket.isConnected = true;
    },
    SOCKET_ONCLOSE(state, event)  {
      state.socket.isConnected = false;
    },
    SOCKET_ONERROR(state, event)  {
      console.error(state, event);
    },
    SOCKET_ONMESSAGE(state, message)  {
      state.socket.message = message;
    },
    SOCKET_RECONNECT(state, count) {
      console.info(state, count);
    },
    SOCKET_RECONNECT_ERROR(state) {
      state.socket.reconnectError = true;
    },
  },
});
