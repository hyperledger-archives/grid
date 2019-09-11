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
    <sidebar-section
      v-on:action="$emit('show-new-gameroom-modal')"
      :section="gamerooms"
      :items="gameroomList" />
    <router-link class="position-end" to='/dashboard/invitations'>
      <sidebar-section :section="invitations" />
    </router-link>
  </div>
</template>

<script lang="ts">
import { Vue, Prop, Component } from 'vue-property-decorator';
import SidebarSection from '@/components/sidebar/SidebarSection.vue';
import { Section } from '@/store/models';
import gamerooms from '@/store/modules/gamerooms';

@Component({
  components: { SidebarSection },
})
export default class GameroomSidebar extends Vue {
  @Prop() sections!: Section[];

  homeSection = {
    name: 'Home',
    icon: 'home',
    active: false,
    link: 'home',
    dropdown: false,
    action: false,
    actionIcon: '',
  };

  gameroomsSection = {
    name: 'My Gamerooms',
    icon: 'games',
    active: false,
    link: '',
    dropdown: true,
    action: true,
    actionIcon: 'add_circle_outline',
  };

  invitationsSection = {
    name: 'Invitations',
    icon: 'email',
    active: false,
    link: '',
    dropdown: false,
    action: false,
    actionIcon: '',
  };

  mounted() {
    gamerooms.listGamerooms();
  }

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

  get gameroomList() {
    return gamerooms.activeGameroomList.map((gameroom) => {
      return {
        id: gameroom.circuit_id,
        name: gameroom.alias,
      };
    });
  }
}
</script>

<style lang="scss" scoped>
  @import '@/scss/components/sidebar/_sidebar-container.scss';
</style>
