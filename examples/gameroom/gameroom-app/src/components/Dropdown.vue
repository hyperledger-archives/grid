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
  <div class="dropdown-wrapper">
    <div class="dropdown-container">
      <button
        ref="button"
        @click="toggleDropdown"
        class="dropdown-button"
      >
        <i :class="{ 'icon-active' : dropdownVisible }" class="icon material-icons-round">
          {{ icon }}
        </i>
        <span v-if="!dropdownVisible && badgeCount > 0" class="badge">
          {{ badgeCount }}
        </span>
      </button>
      <div
        v-on-clickaway="toggleDropdown"
        v-if="dropdownVisible"
        class="dropdown-menu"
      >
        <div class="dropdown-header">
          <span class="title">{{ title }}</span>
        </div>
        <div class="dropdown-body">
          <dropdown-notification
            v-for="notification in notifications"
            :key="notification.id"
            :notification="notification"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';
import { mixin as clickaway } from 'vue-clickaway';
import DropdownNotification from '@/components/DropdownNotification.vue';

@Component({
  mixins: [ clickaway ],
  props: ['icon', 'badgeCount', 'dropdownItems', 'title'],
  components: { DropdownNotification },
})
export default class Dropdown extends Vue {
  dropdownVisible = false;

  mounted() {
    this.$store.dispatch('notifications/listNotificationsMock');
  }

  get notifications() {
    return this.$store.getters['notifications/getNotifications'];
  }

  toggleDropdown() {
    this.dropdownVisible = !this.dropdownVisible;
  }
}
</script>

<style lang="scss" scoped>
@import '@/scss/components/_dropdown.scss';
</style>
