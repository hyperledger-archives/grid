<!--
Copyright 2018-2020 Cargill Incorporated

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
  <div class="dashboard-container">
    <toast toast-type="error" :active="error" v-on:toast-action="clearError">
      {{ error }}
    </toast>
    <toast toast-type="success" :active="success" v-on:toast-action="clearSuccess">
      {{ success }}
    </toast>
    <modal v-if="displayModal" @close="closeNewGameroomModal">
      <h4 slot="title">New Gameroom</h4>
      <div slot="body">
        <form class="modal-form" @submit.prevent="createGameroom">
          <label class="form-label">
            <div class="multiselect-label">Other organization</div>
          </label>
          <multiselect
            class="multiselect-input"
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
            :allow-empty="false" />
          <label class="form-label">
            Gameroom name
            <input class="form-input" type="text" v-model="newGameroom.alias" />
          </label>
          <div class="flex-container button-container">
            <button class="btn-action outline small"
                    type="reset"
                    @click.prevent="closeNewGameroomModal">
              <div class="btn-text">Cancel</div>
            </button>
            <button class="btn-action small" type="submit" :disabled="!canSubmitNewGameroom">
              <div v-if="submitting" class="spinner" />
              <div class="btn-text" v-else>Send</div>
            </button>
          </div>
        </form>
      </div>
    </modal>
    <gameroom-sidebar
      v-on:show-new-gameroom-modal="showNewGameroomModal()"
      class="sidebar" />
    <div v-if="isPageLoading" class='dashboard-view'>
      <loading :message="pageLoadingMessage" />
    </div>
    <router-view v-else v-on:error="setError" v-on:success="setSuccess" class="dashboard-view" />
  </div>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';
import { mapGetters } from 'vuex';
import GameroomSidebar from '@/components/sidebar/GameroomSidebar.vue';
import Toast from '../components/Toast.vue';
import Multiselect from 'vue-multiselect';
import gamerooms from '@/store/modules/gamerooms';
import nodes from '@/store/modules/nodes';
import { Node } from '@/store/models';
import Modal from '@/components/Modal.vue';
import Loading from '@/components/Loading.vue';

interface NewGameroom {
  alias: string;
  member: Node | null;
}

@Component({
  components: { Modal, Multiselect, GameroomSidebar, Toast, Loading },
  computed: {
    ...mapGetters('nodes', {
      nodeList: 'nodeList',
    }),
    ...mapGetters('pageLoading', {
      isPageLoading: 'isPageLoading',
      pageLoadingMessage: 'pageLoadingMessage',
    }),
  },
})
export default class Dashboard extends Vue {
  nodes!: Node[];
  displayModal = false;
  submitting = false;
  error = '';
  success = '';

  newGameroom: NewGameroom = {
    alias: '',
    member: null,
  };

  mounted() {
    this.$store.dispatch('nodes/listNodes');
  }

  get canSubmitNewGameroom() {
    if (!this.submitting &&
        this.newGameroom.alias !== '' &&
        this.newGameroom.member !== null) {
      return true;
    }
    return false;
  }

  setError(message: string) {
    this.error = message;
    setTimeout(() => {
      this.clearError();
    }, 6000);
  }

  setSuccess(message: string) {
    this.success = message;
    setTimeout(() => {
      this.clearSuccess();
    }, 6000);
  }

  clearError() {
    this.error = '';
  }

  clearSuccess() {
    this.success = '';
  }

  async createGameroom() {
    if (this.canSubmitNewGameroom) {
        this.submitting = true;
        const member = this.newGameroom.member ? this.newGameroom.member.identity : '';
        try {
          this.$store.dispatch('gamerooms/proposeGameroom', {
            alias: this.newGameroom.alias,
            members: [member],
          });
          this.setSuccess('Your invitation has been sent!');
        } catch (e) {
          console.error(e);
          this.setError(e.message);
        }
        this.submitting = false;
        this.closeNewGameroomModal();
    }
  }

  getMemberLabel(node: Node) {
    let endpoint = node.endpoint;
    if (process.env.VUE_APP_BRAND
     && node.endpoint.includes(process.env.VUE_APP_BRAND)) {
      endpoint = 'local';
    }

    return `${node.metadata.organization} (${endpoint})`;
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
  @import '@/scss/components/_dashboard.scss';
</style>
