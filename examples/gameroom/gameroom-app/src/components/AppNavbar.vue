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
    <ul class="nav-links">
      <li class="nav-item">
        <router-link to="/">
          Home
        </router-link>
      </li>
      <li class="nav-item">
        <router-link to="/gamerooms">
          View Gamerooms
        </router-link>
      </li>
    </ul>
    <ul class="signin-links">
      <li class="nav-item">
        <router-link to="/login">
          Log in
        </router-link>
      </li>
      <li class="nav-item">
        <router-link to="/register">
          Register
        </router-link>
      </li>
    </ul>
  </nav>
</template>

<script>
export default {};
</script>

<style lang="scss" scoped>
.navbar {
  background-color: $color-base;
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

</style>
