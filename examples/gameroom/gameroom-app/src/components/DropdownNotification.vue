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
    <div class="logo-wrapper">
      <img class="logo" :src="getLogo(notification.org)" alt="">
    </div>
    <div class="text-wrapper">
      <span class="text">
        <span class="bold">{{ notification.org }}</span>
        <span>{{ formatText(notification) }}</span>
        <span class="bold">{{ notification.target }}</span>
        !
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
import { GameroomNotification } from '@/store/models';


@Component
export default class DropdownNotification extends Vue {
  icons = {
    invite: 'share',
  };

  @Prop()
  notification!: GameroomNotification;

  getLogo(org: string) {
    const images = require.context('../assets/logos', false, /\.png$/);
    return images('./' + org + '.png');
  }

  formatText(notification: GameroomNotification) {
    if (notification.notification_type === 'invite') {
      return ' has invited you to a new gameroom: ';
    }
    return '';
  }

  fromNow(timestamp: number): string {
    return moment.unix(timestamp).fromNow();
  }

  markAsRead() {
    this.$store.dispatch('notifications/markReadMock', this.notification.id);
  }
}
</script>


<style lang="scss" scoped>
@import '@/scss/components/_dropdown-notification.scss';
</style>
