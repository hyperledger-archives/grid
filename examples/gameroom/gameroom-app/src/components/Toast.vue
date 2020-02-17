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
  <div class="toast-wrapper"
       :class="[{'active': active}, toastType]">
    <i class="material-icons-round type-icon">{{ this.icon() }}</i>
    <div class="toast-content">
      <slot></slot>
    </div>
    <i @click.prevent="toastAction" class="material-icons-round close-icon">close</i>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator';
@Component
export default class Toast extends Vue {
  @Prop({ default: false }) active!: boolean;
  @Prop() toastType!: string;

  icon() {
    switch (this.toastType) {
      case 'error': return 'error_outline';
      case 'success': return 'check_circle_outline';
      default: return '';
    }
  }

  toastAction() {
    this.$emit('toast-action');
  }
}
</script>

<style lang="scss" scoped>
  @import '@/scss/components/_toast.scss';
</style>
