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
  <button
    :class="{ 'btn-tab-active' : isActive }"
    @click="$emit('select-tab')"
    class="btn-tab">
    <div class="highlight"></div>
    <div class="icon"> <i class="material-icons-round">{{ icon }}</i></div>
    <div class="text">{{ text }}</div>
  </button>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';

@Component({
  props: {
    isActive: Boolean,
    text: String,
    icon: String,
  },
})
export default class TabButton extends Vue {

}
</script>

<style lang="scss" scoped>
  @import '@/scss/components/_tab-button.scss';
</style>
