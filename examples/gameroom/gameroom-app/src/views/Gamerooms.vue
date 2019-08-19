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
  <div>
    <toast toast-type="error" :active="error" v-on:toast-action="clearError">
      {{ error }}
    </toast>
    <toast toast-type="success" :active="success" v-on:toast-action="clearSuccess">
      {{ success }}
    </toast>
    <modal v-if="displayModal" @close="closeNewGameroomModal">
      <h3 slot="title">New Gameroom</h3>
      <div slot="body">
        <form class="modal-form" @submit.prevent="createGameroom">
          <label class="form-label">
            Alias
            <input v-focus class="form-input" type="text" v-model="newGameroom.alias" />
          </label>
          <label class="form-label">
            <div class="multiselect-label">Member</div>
          </label>
          <multiselect
            v-model="newGameroom.member"
            track-by="identity"
            label="metadata"
            placeholder=""
            open-direction="bottom"
            :show-labels="false"
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
          <div class="flex-container button-container">
            <button class="btn-action outline small" @click.prevent="closeNewGameroomModal">
              <div class="btn-text">Cancel</div>
            </button>
            <button class="btn-action small" type="submit" :disabled="!canSubmitNewGameroom">
              <div v-if="submitting" class="spinner" />
              <div class="btn-text" v-else>Submit</div>
            </button>
          </div>
        </form>
      </div>
    </modal>
    <tabs v-on:show-new-gameroom-modal="showNewGameroomModal()" />
  </div>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';
import Modal from '@/components/Modal.vue';
import Tabs from '@/components/Tabs.vue';
import Multiselect from 'vue-multiselect';
import gamerooms from '@/store/modules/gamerooms';
import nodes from '@/store/modules/nodes';
import { Node } from '@/store/models';
import Toast from '../components/Toast.vue';

interface NewGameroom {
  alias: string;
  member: Node | null;
}

@Component({
  components: { Modal, Multiselect, Tabs, Toast },
})
export default class Gamerooms extends Vue {
  displayModal = false;
  submitting = false;
  error = '';
  success = '';

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
    if (!this.submitting &&
        this.newGameroom.alias !== '' &&
        this.newGameroom.member !== null) {
      return true;
    }
    return false;
  }

  clearError() {
    this.error = '';
  }

  clearSuccess() {
    this.success = '';
  }

  async createGameroom() {
    this.submitting = true;
    try {
      await gamerooms.proposeGameroom({
        alias: this.newGameroom.alias,
        member: [this.newGameroom.member as Node],
      });
      this.success = 'Your invitation has been sent!';
    } catch (e) {
      this.error = e.message;
    }
    this.submitting = false;
    this.closeNewGameroomModal();
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
