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
        <router-link to="/gamerooms">
          Gamerooms
        </router-link>
      </li>
      <li>
        <router-link to='/'>
          <span v-on:click="logout">Log out</span>
        </router-link>
      </li>
    </ul>
    <ul v-if="!isLoggedIn">
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
import user from '@/store/modules/user';

@Component
export default class AppNavbar extends Vue {
  get isLoggedIn() {
    return user.isLoggedIn;
  }

  logout() {
    user.clearUser();
  }
}
</script>

<style lang="scss" scoped>
.navbar {
  background-color: $color-base;
  border-bottom: 1px solid $color-border;
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap-reverse;
  ul {
    padding-left: 0;
    display: flex;
    list-style-type: none;
    li {
      padding-left: 1em;
      padding-right: 1em;
    }
  }
}
.navbar-brand {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: .5em;
  .brand-text {
    font-size: 2em;
  }
}

</style>
