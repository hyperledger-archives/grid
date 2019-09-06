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
  <div class="card-container">
    <div class="header">
      <div class="title">
        Name placeholder
      </div>
    </div>
    <div class="body">
      <div class="data">
        <div class="key">{{ getTimestampLabel(proposal.requester) }}</div>
        <div class="value">
          {{ fromNow(proposal.created_time) }}
        </div>
      </div>
      <div v-if="!isSelf(proposal.requester)" class="data">
        <div class="key">from:</div>
        <div class="value">
          {{ proposal.requester_org }}
        </div>
      </div>
      <div class="data">
        <div class="key">members:</div>
        <div class="value">
          <li class="list-value"
              v-for="(member, index) in proposal.members"
              :key="index">
            {{ member.organization }}
          </li>
        </div>
      </div>
    </div>
    <div v-if="!isSelf(proposal.requester)" class="actions">
      <button :disabled="!canSubmit"
              class="btn-action table"
              @click="acceptInvitation">
        <div v-if="acceptSubmitting" class="spinner" />
        <div v-else class="btn-text">Accept</div>
      </button>
      <button :disabled="!canSubmit"
              class="btn-action table outline"
              @click="rejectInvitation">
        <div v-if="rejectSubmitting" class="spinner" />
        <div class="btn-text">Reject</div>
      </button>
    </div>
  </div>
</template>

<script lang="ts">
import { Vue, Prop, Component } from 'vue-property-decorator';
import * as moment from 'moment';
import { GameroomProposal } from '../store/models';
import proposals from '@/store/modules/proposals';

@Component
export default class InvitationCard extends Vue {
  @Prop() proposal!: GameroomProposal;

  acceptSubmitting = false;
  rejectSubmitting = false;

  get canSubmit() {
    return !this.acceptSubmitting && !this.rejectSubmitting;
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

  async acceptInvitation() {
    this.acceptSubmitting = true;
    try {
      await proposals.vote({
        proposalID: this.proposal.proposal_id,
        ballot: {
          circuit_id: this.proposal.circuit_id,
          circuit_hash: this.proposal.circuit_hash,
          vote: 'Accept',
        },
      });
      this.$emit('success', 'You have accepted the invitation.');
    } catch (e) {
      console.error(e);
      this.$emit('error', e.message);
    }
    this.acceptSubmitting = false;
  }

  async rejectInvitation() {
    this.rejectSubmitting = true;
    try {
      await proposals.vote({
        proposalID: this.proposal.proposal_id,
        ballot: {
          circuit_id: this.proposal.circuit_id,
          circuit_hash: this.proposal.circuit_hash,
          vote: 'Reject',
        },
      });
      this.$emit('success', 'You have rejected the invitation.');
    } catch (e) {
      console.error(e);
      this.$emit('error', e.message);
    }
    this.rejectSubmitting = false;
  }
}
</script>

<style lang="scss" scoped>
  @import '@/scss/components/_invitation-card.scss';
</style>
