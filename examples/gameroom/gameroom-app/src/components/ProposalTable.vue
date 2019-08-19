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
    <table v-if="proposals.length > 0" class="tbl">
      <thead>
        <tr class="tbl-row tbl-row-header">
          <th class="tbl-data tbl-data-header">
            name
          </th>
          <th class="tbl-data tbl-data-header">
            members
          </th>
          <th class="tbl-data tbl-data-header">
            invited by
          </th>
          <th class="tbl-data tbl-data-header">
            received
          </th>
          <th class="tbl-data tbl-data-header">
            actions
          </th>
        </tr>
      </thead>
      <tbody>
        <tr class="tbl-row tbl-row-body" v-for="(proposal, index) in proposals" :key="index">
          <td class="tbl-data tbl-data-body">
            {{ proposal.name }}
          </td>
          <td class="tbl-data tbl-data-body">
            <li v-for="(member, index) in proposal.members" :key="index">
              {{ member }}
            </li>
          </td>
          <td class="tbl-data tbl-data-body">
            {{ proposal.requester }}
          </td>
          <td class="tbl-data tbl-data-body">
            {{ fromNow(proposal.created_time) }}
          </td>
          <td class="tbl-data tbl-data-body">
            <div class="flex-container button-container">
              <button class="btn-action table outline">
                <div class="btn-text">Reject</div>
              </button>
              <button class="btn-action table">
                <div class="btn-text">Accept</div>
              </button>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
    <h3 class="tbl-placeholder" v-if="proposals.length === 0">No pending proposals</h3>
  </div>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';
import proposals from '@/store/modules/proposals';
import * as moment from 'moment';

@Component
export default class ProposalTable extends Vue {
  columns = ['name', 'members', 'sender', 'received', 'actions'];

  mounted() {
    proposals.listProposalsMock();
  }

  get proposals() {
    return proposals.proposalList;
  }

  fromNow(timestamp: number): string {
    return moment.unix(timestamp).fromNow();
  }
}
</script>

<style lang="scss" scoped>
@import '@/scss/components/_custom-table.scss';
</style>
