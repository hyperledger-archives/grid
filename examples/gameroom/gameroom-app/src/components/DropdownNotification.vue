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
  <div :class="{ 'new' : !notification.read }"
       class="dropdown-notification"
       @click="markAsRead">
    <div class="text-wrapper">
      <span class="text">
        <span>{{ formatText(notification) }}</span>
      </span>
      <div class="meta-wrapper">
        <i class="icon material-icons-round">
          {{ icons[notification.notification_type] }}
        </i>
        <span class="timestamp">
          {{ fromNow(notification.timestamp) }}
        </span>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { Vue, Component, Prop } from 'vue-property-decorator';
import * as moment from 'moment';
import { GameroomNotification, Gameroom } from '@/store/models';


@Component
export default class DropdownNotification extends Vue {
  icons = {
    invite: 'share',
  };

  @Prop()
  notification!: GameroomNotification;

  get link(): any {
    const regex = RegExp('^new_game_created');
    const notification: string =
        regex.test(this.notification.notification_type) ? 'new_game_created' : this.notification.notification_type;
    switch (notification) {
      case('gameroom_proposal'): return {name: 'invitations'};
      case('circuit_active'): return {name: 'gamerooms', params: {id: `${this.notification.target}`}};
      case('new_game_created'): return {name: 'games', params: {id: `${this.notification.target}`, gameName: `${this.getGameName(this.notification.notification_type)}`}};
      default: return '';
    }
  }

  getName(): string {
    const gamerooms = this.$store.getters['gamerooms/gameroomList'];
    const gameroom = gamerooms.find((g: Gameroom) => g.circuit_id === this.notification.target);
    return gameroom.alias;
  }

  formatText(notification: GameroomNotification) {
    const regex = RegExp('new_game_created:');
    if (regex.test(notification.notification_type)) {
      const gameName = this.getGameName(notification.notification_type);
      return  `A new game is available in gameroom ${this.getName()}: ${gameName}`;
    }
    switch (notification.notification_type) {
      case('gameroom_proposal'): {
        return `${notification.requester_org} has invited you to a new gameroom: ${this.getName()}`;
      }
      case('circuit_active'): {
        return `A new gameroom has been created: ${this.getName()}`;
      }
      default: return '';
    }
  }

  getGameName(notificationType: string): string {
    const regex = RegExp('new_game_created:');
    if (regex.test(notificationType)) {
      const gameName = notificationType.split('new_game_created:')[1];
      return  gameName;
    } else {
      return '';
    }
  }

  fromNow(timestamp: number): string {
    return moment.unix(timestamp).fromNow();
  }

  markAsRead() {
    this.$store.dispatch('notifications/markRead', this.notification.id);
    this.$router.push(this.link);
    this.$emit('click');
  }
}
</script>


<style lang="scss" scoped>
@import '@/scss/components/_dropdown-notification.scss';
</style>
