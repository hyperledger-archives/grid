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
  <div class="game-info-wrapper">
    <div class="header">
      <div class="game-icon">
        XO
      </div>
      <div class="info">
        <div>
          Game started: {{ fromNow(gameInfo.createdTimestamp) }}
        </div>
        <div v-if="gameInfo.playerOne">
          Last move: {{ fromNow(gameInfo.lastTurnTimestamp) }}
        </div>
      </div>
    </div>
    <div class="players">
      <div class="player">
        <i class="icon material-icons-round">{{ playerOneIcon }}</i>
        {{ formatPlayerName(gameInfo.playerOne) }}
      </div>
      <div class="player">
        <i class="icon material-icons-round">{{ playerTwoIcon }}</i>
        {{ formatPlayerName(gameInfo.playerTwo) }}
      </div>
    </div>
    <div class="status">
      {{ status }}
    </div>
  </div>
</template>

<script lang="ts">
import { Vue, Component, Prop } from 'vue-property-decorator';
import * as moment from 'moment';
import { Player } from '@/store/models';

export enum GameStatus {
  PlayerOneNext,
  PlayerTwoNext,
  PlayerOneWin,
  PlayerTwoWin,
  Tie,
}

export interface GameInfo {
  gameType: string;
  playerOne: Player;
  playerTwo: Player;
  status: GameStatus;
  createdTimestamp: number;
  lastTurnTimestamp: number;
}

@Component
export default class GameInfoPanel extends Vue {
  @Prop() gameInfo!: GameInfo;

  get status(): string {
    if (!this.gameInfo.playerOne) {
      return 'Take a space to join the game as X';
    } else if (!this.gameInfo.playerTwo) {
      return 'Take a space to join the game as O';
    }

    switch (this.gameInfo.status) {
      case(GameStatus.PlayerOneWin): return `${this.gameInfo.playerOne.name} was victorious`;
      case(GameStatus.PlayerTwoWin): return `${this.gameInfo.playerTwo.name} was victorious`;
      case(GameStatus.Tie): return 'Game resulted in a draw';
      default: return 'Game in progress';
    }
  }

  get playerOneIcon(): string {
    if (this.gameInfo.status === GameStatus.PlayerOneNext) {
      return 'radio_button_checked';
    } else { return 'radio_button_unchecked'; }
  }

  get playerTwoIcon(): string {
    if (this.gameInfo.status === GameStatus.PlayerTwoNext) {
      return 'radio_button_checked';
    } else { return 'radio_button_unchecked'; }
  }

  fromNow(timestamp: number): string {
    return moment.unix(timestamp).fromNow();
  }

  formatPlayerName(player: Player) {
    if (!player) {
      return 'Waiting for player to join';
    }
    return `${player.name} (${player.organization})`;
  }
}

</script>

<style lang="scss">
  @import '@/scss/components/_game-info-panel.scss';
</style>
