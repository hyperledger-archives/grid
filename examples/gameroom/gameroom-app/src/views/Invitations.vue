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
  <div class="invitations-container">
    <h2 class="title">Invitations</h2>
    <div v-if="proposals.length > 0" class="data-container">
      <invitation-card
        v-on:success="$emit('success', $event)"
        v-on:error="$emit('error', $event)"
        class="card-container"
        v-for="(proposal, index) in proposals"
        :key="index"
        :proposal="proposal" />
    </div>
    <h3 v-else class="tbl-placeholder">No pending proposals</h3>
  </div>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';
import InvitationCard from '@/components/InvitationCard.vue';
import proposals from '@/store/modules/proposals';

@Component({
  components: { InvitationCard },
})
export default class Invitations extends Vue {

  mounted() {
    proposals.listProposals();
  }

  get proposals() {
    return proposals.proposalList;
  }
}
</script>

<style lang="scss" scoped>
  .invitations-container {
    display: flex;
    flex-direction: column;
    padding: 2rem 4rem;

    .data-container {
      border-top: 1px solid $color-border;
      padding: 1rem;
      margin-top: 2rem;
      overflow: auto;
      align-items: center;
    }
  }
</style>
