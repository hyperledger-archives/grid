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

import { VuexModule, Module, getModule, Mutation, Action } from 'vuex-module-decorators';
import store from '@/store';
import { User, UserCredentials } from '@/store/models';
import { userAuthenticate, userCreate } from '@/store/api';

@Module({
  namespaced: true,
  name: 'user',
  store,
  dynamic: true,
})
class UserModule extends VuexModule {
  user: User | null = null;

  @Mutation
  setUser(user: User) { this.user = user; }

  @Mutation
  clearUser() { this.user = null; }

  @Action({commit: 'setUser'})
  async register(userInfo: User) {
    const user = await userCreate(userInfo);
    return user;
  }

  @Action({commit: 'setUser'})
  async authenticate(credentials: UserCredentials) {
    const user = await userAuthenticate(credentials);
    return user;
  }

  get isLoggedIn() {
    return this.user;
  }
}
export default getModule(UserModule);
