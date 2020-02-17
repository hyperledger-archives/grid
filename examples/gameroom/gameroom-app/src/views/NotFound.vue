<!--
Copyright 2018-2020 Cargill Incorporated
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
  <div class="not-found-container">
    <h2> 404 Error</h2>
    <div class="message-containter">
      <i class="material-icons error-icon"> error_outline </i>
      <h3>The page you requested does not exist</h3>
    </div>
  </div>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';
@Component
export default class NotFoundPage extends Vue {}
</script>

<style lang="scss" scoped>
  .not-found-container {
    display: flex;
    flex-direction: column;
    padding: 2rem 4rem;
    height: 100%;
  }

  .message-containter {
    @include overlay(1);
    @include rounded-border;
    display: flex;
    width: 100%;
    height: 100%;
    padding: 1rem;
    margin-top: 2rem;
    align-items: center;
    justify-content: center;

  }
  .error-icon {
    color: $color-text-primary;
    font-size: 5rem;
    margin: 1rem;
  }

</style>
