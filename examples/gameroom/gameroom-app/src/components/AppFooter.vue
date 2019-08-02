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
  <div class="footer">
    <span>
      <a href="/">Gameroom</a>
      - An example application built on Splinter. Copyright 2019 Cargill Incorporated. Licensed
      under the Apache License, Version 2.0.
    </span>
  </div>
</template>

<script lang="ts">
  import { Vue, Component } from 'vue-property-decorator';
  export default class AppFooter extends Vue {}
</script>

<style lang="scss" scoped>
  @import '@/scss/components/_app-footer.scss';
</style>
