<!--
Copyright 2019 Cargill Incorporated

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
-->

<template>
  <div class="gamerooms-container">
    <modal v-if="displayModal" @close="closeNewGameroomModal">
      <h3 slot="title">New Gameroom</h3>
      <div slot="body">
        <form class="modal-form" @submit.prevent="createGameroom">
          <label class="form-label">
            Alias
            <input class="form-input" type="text" v-model="newGameroom.alias" />
          </label>
          <label class="form-label">
            <div class="multiselect-label">Member</div>
            <multiselect
              v-model="newGameroom.member"
              track-by="identity"
              label="metadata"
              placeholder=""
              deselect-label=""
              open-direction="bottom"
              :close-on-select="true"
              :clear-on-select="false"
              :custom-label="getMemberLabel"
              :options="nodeList"
              :allow-empty="false"
            >
              <template slot="singleLabel" slot-scope="{ option }">
                <strong>{{ option.metadata.organization }}</strong>
              </template>
            </multiselect>
          </label>
          <div class="flex-container button-container">
            <button class="btn-action form-button btn-outline" @click.prevent="closeNewGameroomModal">
              Cancel
            </button>
            <button class="btn-action form-button" type="submit" :disabled="!canSubmitNewGameroom">
              Submit
            </button>
          </div>
        </form>
      </div>
    </modal>
    <div class="new-gameroom-button-container">
      <button class="icon-button" @click="showNewGameroomModal">
        <div class="button-content">
        <i class="material-icons" style="font-size:1.5em;">add</i>
        <span class="button-text">
          New gameroom
        </span>
        </div>
      </button>
    </div>
  </div>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';
import Modal from '@/components/Modal.vue';
import Multiselect from 'vue-multiselect';
import gamerooms from '@/store/modules/gamerooms';
import nodes from '@/store/modules/nodes';
import { Node } from '@/store/models';

interface NewGameroom {
  alias: string;
  member: Node | null;
}

@Component({
  components: { Modal, Multiselect },
})
export default class Gamerooms extends Vue {
  displayModal = false;

  newGameroom: NewGameroom = {
    alias: '',
    member: null,
  };

  mounted() {
    nodes.listNodes();
  }

  get nodeList() {
    return nodes.nodeList;
  }

  get canSubmitNewGameroom() {
    if (this.newGameroom.alias !== '' &&
        this.newGameroom.member !== null) {
      return true;
    }
    return false;
  }

  createGameroom() {
    gamerooms.proposeGameroom({
      alias: this.newGameroom.alias,
      member: [this.newGameroom.member as Node],
    });
  }

  getMemberLabel(node: Node) {
    return node.metadata.organization;
  }

  showNewGameroomModal() {
    this.displayModal = true;
  }

  closeNewGameroomModal() {
    this.displayModal = false;
    this.newGameroom.alias = '';
    this.newGameroom.member = null;
  }
}
</script>

<style lang="scss" scoped>
.gamerooms-container {
  margin: 1em;
  display: flex;

  .new-gameroom-button-container {
    margin-left: auto;
  }
}
</style>
