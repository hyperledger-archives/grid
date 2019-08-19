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
  <div class="tab-wrapper">
    <div class="tab-sidebar">
        <tab-button
          v-on:select-tab="selectTab(1)"
          v-bind:is-active="currentTab == 1"
          class="btn-tab"
          text="Gamerooms"
          icon="games"/>
        <tab-button
          v-on:select-tab="selectTab(2)"
          v-bind:is-active="currentTab == 2"
          class="btn-tab"
          text="Invites"
          icon="share"/>
    </div>
    <div class="tab-container">
      <div class="tab-title">
        <h2 v-if="currentTab == 1">Gamerooms</h2>
        <h2 v-if="currentTab == 2">Invites</h2>
        <button class="btn-action" @click="$emit('show-new-gameroom-modal')">
            <div class="btn-text">New Gameroom</div>
        </button>
      </div>
      <div class="tab">
        <gameroom-table v-if="currentTab == 1">
          No gamerooms available
        </gameroom-table>
        <proposal-table v-if="currentTab == 2">
          No invites
        </proposal-table>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';
import TabButton from '@/components/TabButton.vue';
import ProposalTable from '@/components/ProposalTable.vue';
import GameroomTable from '@/components/GameroomTable.vue';

@Component({
  components: { GameroomTable, ProposalTable, TabButton },
})
export default class Tabs extends Vue {
  currentTab = 1;

  selectTab(tab: number) {
    this.currentTab = tab;
  }
}
</script>


<style lang="scss" scoped>
  @import '@/scss/components/_tabs.scss';
</style>
