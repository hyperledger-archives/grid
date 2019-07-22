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
          	<i class="material-icons">close</i>
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

<style lang="scss">
.m-modal-overlay {
  display: flex;
  align-items: center;
  justify-content: center;
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 10;
  background-color: rgba(0, 0, 0, 0.4);
  transition: opacity .3s ease;
}

.m-modal-wrapper {
  display: flex;
  flex-direction: column;
  width: 500px;
  background-color: $color-background;
  transition: all .3s ease;
  @include rounded-border;
}

.m-modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 15px 20px;
  border-bottom: 1px solid $color-white;
  h3 {
    color: $color-text;
    line-height: 1.4;
    font-size: 1.2em;
    font-weight: 600;
    margin: 0;
  }
}

.m-modal-close {
  color: #c8ccd1;
  text-transform: uppercase;
  text-decoration: none;
  i {
    color: inherit;
    font-size: 1.5em;
  }
}

.m-modal-body {
  flex: 1;
  font-size: 14px;
  padding: 20px;
  overflow-y: visible;
  div {
    margin: 0;
  }
}

.modal-enter, .modal-leave-active {
  opacity: 0;
}

.modal-enter .m-modal-wrapper,
.modal-leave-active .m-modal-wrapper {
  transform: scale(1.1);
}
</style>
