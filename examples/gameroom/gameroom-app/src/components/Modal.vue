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
  <transition name="modal">
    <div class="m-modal-overlay">
      <div class="m-modal-wrapper">
        <div class="m-modal-header">
          <slot name="title"></slot>
          <a href="#" class="m-modal-close" @click.stop.prevent="$emit('close')">
            <i class="material-icons-round">close</i>
          </a>
        </div>
        <div class="m-modal-body">
          <slot name="body"></slot>
        </div>
      </div>
    </div>
  </transition>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';

@Component
export default class Modal extends Vue {}
</script>

<style lang="scss" scoped>
  @import '@/scss/components/_modal.scss';
</style>
