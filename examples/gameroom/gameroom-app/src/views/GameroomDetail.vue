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
  <div class="gameroom-detail-container">
    <div class="gameroom-information">
      <h2 class="gameroom-name">{{ gameroom.alias }}</h2>
    </div>
  </div>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';
import gamerooms from '@/store/modules/gamerooms';
import { Gameroom, Member, Game } from '@/store/models';

@Component
  export default class GameroomDetails extends Vue {
      cachedGameroom: Gameroom = {} as Gameroom;

      mounted() {
        gamerooms.listGamerooms();
      }

      get gameroom(): Gameroom {
        if (!this.cachedGameroom.circuit_id) {
            this.cachedGameroom = gamerooms.gameroomList.find(
              (gameroom) => gameroom.circuit_id ===  this.$route.params.id) || {} as Gameroom;
        }
        return this.cachedGameroom;
      }

  }
</script>

<style lang="scss" scoped>
@import '@/scss/components/_gameroom-details.scss';
</style>
