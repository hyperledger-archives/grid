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
  <td class="xo-cell" @click="$emit('click')">
    <div class="marker">
      <div v-if="submitting" class="spinner large" />
      <img v-else class="icon" :class="{visible : isVisible}" :src="getMarker()" />
    </div>
  </td>
</template>

<script lang="ts">
import { Vue, Component, Prop } from 'vue-property-decorator';
const whitelabel = require('@/../whitelabel.config')[process.env.VUE_APP_BRAND!];

@Component
export default class XOCell extends Vue {
  @Prop() value!: string;

  get isVisible(): boolean {
    if (this.value === '-') {
      return false;
    }
    return true;
  }

  get submitting() {
    return (this.value === '?');
  }

  getMarker(): string {
    if (this.value === 'X') {
      return require(`@/assets/${whitelabel.brand}/xo/xmark.svg`);
    }
    return require(`@/assets/${whitelabel.brand}/xo/omark.svg`);
  }
}
</script>

<style lang="scss" scoped>
  @import '@/scss/components/xo/_xo-cell.scss';
</style>
