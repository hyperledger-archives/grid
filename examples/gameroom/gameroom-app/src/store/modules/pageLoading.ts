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

export interface PageLoading {
  pageLoading: boolean;
  pageLoadingMessage: string;
}

const pageLoading = {
  pageLoading: false,
  pageLoadingMessage: '',
};

const getters = {
  isPageLoading(state: PageLoading): boolean {
    return state.pageLoading;
  },
  pageLoadingMessage(state: PageLoading): string {
    return state.pageLoadingMessage;
  },
};

const mutations = {
  setPageLoading(state: PageLoading, message: string = 'Loading') {
    state.pageLoading = true;
    state.pageLoadingMessage = message;
  },
  setPageLoadingComplete(state: PageLoading) {
    state.pageLoading = false;
    state.pageLoadingMessage = '';
  },
};

export default {
  namespaced: true,
  name: 'pageLoading',
  state: pageLoading,
  getters,
  mutations,
};
