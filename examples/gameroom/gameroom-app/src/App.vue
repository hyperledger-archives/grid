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
  <div id="app">
    <AppNavbar></AppNavbar>
    <router-view/>
    <AppFooter></AppFooter>
  </div>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';
import AppFooter from '@/components/AppFooter.vue';
import AppNavbar from '@/components/AppNavbar.vue';

@Component({
  components: {
    AppFooter, AppNavbar,
  },
})
export default class App extends Vue {

}
</script>

<style lang="scss">

body {
  margin: 0;
  background-color: $color-background;
}

#{$a-tags} {
  color: $color-text-secondary;
  text-decoration: none;
}

#{$a-tags-hover} {
  color: $color-text;
}

.container {
  display: flex;
  padding-left: 1em;
}

#app {
  font-family: 'Avenir', Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: $color-text;
  background-color: $color-background;
}
</style>
