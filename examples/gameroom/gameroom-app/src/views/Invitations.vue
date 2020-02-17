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
  <div class="invitations-container">
    <h2 class="title">Invitations</h2>
    <div class="data-container">
      <div class="tab-buttons">
        <button class="tab-button"
                @click="selectTab(1)"
                :class="{ 'is-active' : currentTab === 1 }">
          <div class="btn-text">received</div>
        </button >
        <button class="tab-button"
                @click="selectTab(2)"
                :class="{ 'is-active' : currentTab === 2 }">
          <div class="btn-text">sent</div>
        </button>
        <button class="tab-button"
                @click="selectTab(3)"
                :class="{ 'is-active' : currentTab === 3 }">
          <div class="btn-text">all</div>
        </button>
      </div>
      <div class="cards-container" v-if="proposals.length > 0">
        <invitation-card
          v-on:success="$emit('success', $event)"
          v-on:error="$emit('error', $event)"
          class="card-container"
          v-for="(proposal, index) in proposals"
          :key="index"
          :proposal="proposal" />
      </div>
      <div class="placeholder-wrapper" v-else>
        <h3  class="placeholder" >{{ this.placeholderText }}</h3>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';
import InvitationCard from '@/components/InvitationCard.vue';
import { GameroomProposal } from '@/store/models';
@Component({
  components: { InvitationCard },
})
export default class Invitations extends Vue {
  currentTab = 1;

  selectTab(tab: number) {
    this.currentTab = tab;
  }

  mounted() {
    this.$store.dispatch('proposals/listProposals');
  }

  get proposals() {
    const props = this.$store.getters['proposals/getProposalList'].filter(
      (p: GameroomProposal) => p.status === 'Pending');

    if (this.currentTab === 1) {
      return props.filter((p: GameroomProposal) => !this.isSelf(p.requester));
    } else if (this.currentTab === 2) {
      return props.filter((p: GameroomProposal) => this.isSelf(p.requester));
    } else {
      return props;
    }
  }

  get placeholderText(): string {
    if (this.currentTab === 1) {
      return 'You have no incoming invitations.';
    } else if (this.currentTab === 2) {
      return 'You have no outgoing invitations.';
    } else {
      return 'You have no invitations.';
    }
  }

  isSelf(key: string): boolean {
    const publicKey = this.$store.getters['user/getPublicKey'];
    return (key === publicKey);
  }
}
</script>

<style lang="scss" scoped>
  .invitations-container {
    display: flex;
    flex-direction: column;
    padding: 2rem 4rem;
    height: 100%;

    .data-container {
      @include overlay(1);
      @include rounded-border;
      display: flex;
      flex-direction: column;
      width: 100%;
      height: 100%;
      padding: 1rem;
      margin-top: 2rem;

      .tab-buttons {
        display: flex;
        margin-top: 1rem;
        padding-bottom: .4rem;

        .tab-button {
          @include rounded-border;
          display: flex;
          height: 1.5rem;
          background: none;
          border: none;
          margin-right: 1rem;
          color: $color-text-med;
          font-weight: $fw-bold;
          font-size: .9rem;

          &:hover {
            color: $color-text-high;
          }

          .btn-text {
            position: relative;
            text-decoration: none;
            text-transform: uppercase;
          }

          .btn-text:after {
              @include rounded-border;
              position: absolute;
              content: '';
              height: 3px;
              background: transparent;
              bottom: -.2rem;
              margin: 0 auto;
              left: 0;
              right: 0;
              width: 50%;

              -o-transition:.5s;
              -ms-transition:.5s;
              -moz-transition:.5s;
              -webkit-transition:.5s;
              transition:.5s;
          }

          .btn-text:hover:after {
            width: 80%;
            background: $color-primary-light;
          }

          &.is-active {
            color: $color-text-high;

            .btn-text:after {
              background: $color-primary;
              width: 50%;
            }
          }
        }
      }

      .cards-container {
        padding-top: 1rem;
        overflow: auto;
      }

      .placeholder-wrapper {
        display: flex;
        height: 100%;
        align-items: center;
        justify-content: center;
      }
    }

  }
</style>
