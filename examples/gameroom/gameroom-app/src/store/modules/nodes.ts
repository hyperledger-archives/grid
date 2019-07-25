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

import { VuexModule, Module, getModule, Action, Mutation } from 'vuex-module-decorators';
import store from '@/store';
import { Node } from '@/store/models';
import { listNodes } from '@/store/api';


@Module({
  namespaced: true,
  name: 'nodes',
  store,
  dynamic: true,
})
class NodesModule extends VuexModule {
  nodes: Node[] = [];

  @Mutation
  setNodes(nodes: Node[]) { this.nodes = nodes; }

  @Action({ commit: 'setNodes' })
  listNodesMock() {
    return ([
      {
        identity: '123asdf',
        metadata: {
          organization: 'bubba_bakery',
          endpoint: 'tcp://127.0.0.1:8046',
        },
      },
      {
        identity: '2456qwerty',
        metadata: {
          organization: 'anotherorg',
          endpoint: 'tcp://127.0.0.1:8049',
        },
      },
    ]);
  }

  @Action({ commit: 'setNodes' })
  async listNodes() {
    const nodes = await listNodes();
    return nodes;
  }

  get nodeList() {
    return this.nodes;
  }
}
export default getModule(NodesModule);
