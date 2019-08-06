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
  <nav class="navbar">
    <router-link class="navbar-brand" to="/">
      <img src="~brandAssets/logo_wide.png" height="60px">
    </router-link>
    <ul v-if="isLoggedIn">
      <li>
        <router-link to='/'>
          <span v-on:click="logout">Log out</span>
        </router-link>
      </li>
    </ul>
    <ul v-else>
      <li>
        <router-link to="/login">
          Log In
        </router-link>
      </li>
      <li>
        <router-link to="/register">
          Register
        </router-link>
      </li>
    </ul>
  </nav>
</template>

<script lang='ts'>
import { Vue, Component } from 'vue-property-decorator';

@Component
export default class AppNavbar extends Vue {

  get isLoggedIn() {
    return this.$store.getters['user/isLoggedIn'];
  }

  logout() {
    this.$store.commit('user/clearUser');
  }
}
</script>

<style lang="scss" scoped>
  @import '@/scss/components/_app-navbar.scss';
</style>
