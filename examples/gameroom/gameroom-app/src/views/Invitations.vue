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
      <div class="card-container" v-for="(proposal, index) in proposals" :key="index">
        <div class="header">
          <div class="title">
            <h2>{{ proposal.proposal_id }}</h2>
          </div>
        </div>
        <div class="body">
          <div class="data">
            <div class="key">{{ getTimestampLabel(proposal.requester) }}</div>
            <div class="value">
              {{ fromNow(proposal.created_time) }}
            </div>
          </div>
          <div class="data">
            <div class="key">from:</div>
            <div class="value breakable-value">
              {{ proposal.requester }}
            </div>
          </div>
          <div class="data">
            <div class="key">members:</div>
            <div class="value">
              <li class="list-value" v-for="(member, index) in proposal.members" :key="index">
                {{ member.node_id }}
              </li>
            </div>
          </div>
        </div>
        <div v-if="!isSelf(proposal.requester)" class="actions">
          <button class="btn-action table">
            <div class="btn-text">Accept</div>
          </button>
          <button class="btn-action table outline">
            <div class="btn-text">Reject</div>
          </button>
        </div>
      </div>
    </div>
    <h3 v-else class="tbl-placeholder" >No pending proposals</h3>
  </div>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';
import proposals from '@/store/modules/proposals';
import * as moment from 'moment';

@Component
export default class Invitations extends Vue {

  mounted() {
    proposals.listProposalsMock();
  }

  get proposals() {
    return proposals.proposalList;
  }

  fromNow(timestamp: number): string {
    return moment.unix(timestamp).fromNow();
  }

  isSelf(key: string): boolean {
    const publicKey = this.$store.getters['user/getPublicKey'];
    return (key === publicKey);
  }

  getTimestampLabel(key: string) {
    if (this.isSelf(key)) {
      return 'sent:';
    } else {
      return 'received:';
    }
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
    }
  }
</style>
