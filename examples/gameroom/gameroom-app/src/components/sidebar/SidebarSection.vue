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
  <div class="section-container" :class="{ active: section.active }">
    <div class="highlight" />
    <div class="section" >
      <div class="title" @click="$emit('activate', section.name)">
        <div class="section-icon-wrapper">
           <i class="material-icons-round">{{ section.icon }}</i>
        </div>
        <div class="text">{{ section.name }}</div>
        <button class="action" v-if="section.action" @click="$emit('action')">
          <i class="material-icons-round">{{ section.actionIcon }}</i>
        </button>
      </div>
      <div class="items-container" v-if="section.dropdown">
        <router-link
          class="link"
          v-for="(item, index) in items"
          :key="index"
          :class="{active: item.id === $route.params.id}"
          :to="`/dashboard/gamerooms/${item.id}`">
          {{ item.name }}
        </router-link>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { Vue, Prop, Component } from 'vue-property-decorator';
import { Section } from '@/store/models';

interface SectionItem {
  id: string;
  name: string;
}

@Component
export default class SidebarSection extends Vue {
  @Prop() section!: Section;
  @Prop() items!: SectionItem[];
}
</script>

<style lang="scss" scoped>
  @import '@/scss/components/sidebar/_sidebar-section.scss';
</style>
