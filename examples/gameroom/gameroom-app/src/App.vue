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
  @import '@/scss/multiselect.scss';
</style>

<style lang="scss">

*, *:before, *:after {
  box-sizing: border-box;
}

html, body {
  margin: 0;
  background-color: $color-background;
  height: 100%;
}

form, button, input {
  font-family: inherit;
}

button {
  border: none;
  background: $color-primary;
  padding: .5em .5em .5em;
  font-size: 1em;
  font-weight: 600;
  clear: both;
  @include rounded-border;
  outline: none;
  cursor: pointer;
  margin-left: 1em;
  width: auto;
  border: 2px solid $color-primary;

  .button-content {
    display: inline-flex;
    align-items: center;

    .button-text {
      padding: 0 .25em 0;
    }
  }
}

button:disabled {
  background: $color-gray;
  border: 2px solid $color-gray;
  color: $color-text-dark;
}

.outline-button {
  color: $color-primary;
  border: 2px solid $color-primary;
  background-color: $color-transparent;
  @include rounded-border;
}

.form-button {
  width: 6em;
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

.auth-container {
  height: 100%;
  display: flex;
  flex: center;
  justify-content: center;
  align-items: center;
  align-self: center;
}

.auth-wrapper {
  max-width: 500px;
  width: 100%
}

.auth-form {
  background: $color-base;
  padding: 2em 1em;
  @include rounded-border;

  .form-label {
    color: $color-white;
    margin: 0 3%;

    .form-input {
      width: 94%;
      margin: .25em 3% 1em;
      padding: .5em .25em;
      font-size: 1.2em;
      border: 2px solid $color-white;
      outline: none;
      @include rounded-border;
    }

    .form-input:focus {
      border: 2px solid $color-secondary;
    }
  }

  .form-button {
    width: 94%;
    margin: .5em 3% .5em;
    padding: 1em 0;
    font-size: 1.25em;
    clear: both;
    @include rounded-border;
    outline: none;
    cursor: pointer;
  }

  .form-link {
    margin: 1em 3%;
    padding: 1em 0;
  }
}

#app {
  font-family: 'Raleway', sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: $color-text;
  background-color: $color-background;
  display: flex;
  flex-flow: column;
  height: 100%;
}
</style>
