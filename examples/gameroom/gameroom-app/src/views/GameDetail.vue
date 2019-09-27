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
  <div class="data-container">
    <div v-if="game" class="header">
      <h2 class="title">{{ game.game_name }}</h2>
      <router-link class="close" :to="gameroomLink">
        <i class="icon material-icons-round">close</i>
      </router-link>
    </div>
    <div v-if="game" class="xo-container">
      <xo-board :game="game" />
      <game-info-panel :gameInfo="gameInfo" />
    </div>
    <div v-else class="spinner large" />
  </div>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';
import { Game } from '@/store/models';
import XOBoard from '@/components/xo/XOBoard.vue';
import GameInfoPanel, { GameInfo, GameStatus } from '@/components/GameInfoPanel.vue';
import games from '@/store/modules/gamerooms';

@Component({
  components: { 'xo-board': XOBoard, GameInfoPanel },
})
export default class GameDetail extends Vue {
  mounted() {
    this.$store.dispatch('games/listGames', this.$route.params.id);
  }

  get game() {
    return this.$store.getters['games/getGames'].find(
      (game: Game) => game.game_name_hash === this.$route.params.gameNameHash);
  }

  get gameroomLink() {
    return `/dashboard/gamerooms/${this.$route.params.id}`;
  }

  get gameInfo() {
    return {
      gameType: 'XO',
      playerOne: this.game.player_1,
      playerTwo: this.game.player_2,
      status: this.getStatus(this.game.game_status),
      createdTimestamp: this.game.created_time,
      lastTurnTimestamp: this.game.updated_time,
    };
  }

  getStatus(status: string) {
    switch (status) {
      case('P1-NEXT'): return GameStatus.PlayerOneNext;
      case('P2-NEXT'): return GameStatus.PlayerTwoNext;
      case('P1-WIN'): return GameStatus.PlayerOneWin;
      case('P2-WIN'): return GameStatus.PlayerTwoWin;
      case('TIE'): return GameStatus.Tie;
    }
  }
}
</script>

<style lang="scss" scoped>
  .data-container {
    @include overlay(1);
    @include rounded-border;
    display: flex;
    flex-direction: column;
    margin: 2rem;
    padding: 2rem;

    .header {
      display: flex;
      width: 100%;

      .close {
        margin-left: auto;
      }
    }

    .xo-container {
      display: flex;
      flex-direction: row;
      width: 100%;
      height: 100%;
      padding: 1rem;

      @media screen and (max-width: 74rem) {
        flex-direction: column;
      }
    }
  }
</style>
