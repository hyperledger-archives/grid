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
  <div class="sidebar-container">
    <router-link to='/dashboard/home'>
      <sidebar-section :section="home" />
    </router-link>
    <sidebar-section v-on:action="$emit('show-new-gameroom-modal')" :section="gamerooms" />
    <router-link class="position-end" to='/dashboard/invitations'>
      <sidebar-section :section="invitations" />
    </router-link>
  </div>
</template>

<script lang="ts">
import { Vue, Prop, Component } from 'vue-property-decorator';
import SidebarSection from '@/components/sidebar/SidebarSection.vue';
import { Section } from '@/store/models';

@Component({
  components: { SidebarSection },
})
export default class GameroomSidebar extends Vue {
  @Prop() sections!: Section[];

  homeSection = {
    name: 'Home',
    icon: 'home',
    active: false,
    items: [],
    link: 'home',
    dropdown: false,
    action: false,
    actionIcon: '',
  };

  gameroom1 = {
    name: 'gameroom1',
  };

  gameroom2 = {
    name: 'gameroom2',
  };

  gameroomsSection = {
    name: 'My Gamerooms',
    icon: 'games',
    active: false,
    items: [this.gameroom1, this.gameroom2],
    link: '',
    dropdown: true,
    action: true,
    actionIcon: 'add_circle_outline',
  };

  invitationsSection = {
    name: 'Invitations',
    icon: 'email',
    active: false,
    items: [],
    link: '',
    dropdown: false,
    action: false,
    actionIcon: '',
  };

  get home() {
    this.homeSection.active = (this.$route.name === 'dashboard');
    return this.homeSection;
  }

  get gamerooms() {
    this.gameroomsSection.active = (this.$route.name === 'gamerooms');
    return this.gameroomsSection;
  }

  get invitations() {
    this.invitationsSection.active = (this.$route.name === 'invitations');
    return this.invitationsSection;
  }
}
</script>

<style lang="scss" scoped>
  @import '@/scss/components/sidebar/_sidebar-container.scss';
</style>
