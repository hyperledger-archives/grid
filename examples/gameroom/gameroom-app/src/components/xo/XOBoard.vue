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
  <table class="xo-board-wrapper">
    <tr>
      <xo-cell
        v-for="(cell, index) in boardArray.slice(0, 3)"
        :key="index"
        :value="cell"
        :class="cellStyles[index]"
        @click="markCell(index)" />
    </tr>
    <tr>
      <xo-cell v-for="(cell, index) in boardArray.slice(3, 6)"
        :key="index"
        :value="cell"
        :class="cellStyles[index + 3]"
        @click="markCell(index + 3)" />
    </tr>
    <tr>
      <xo-cell
        v-for="(cell, index) in boardArray.slice(6, 9)"
        :key="index"
        :value="cell"
        :class="cellStyles[index + 6]"
        @click="markCell(index + 6)" />
    </tr>
  </table>
</template>

<script lang="ts">
import { Vue, Component, Prop } from 'vue-property-decorator';
import XOCell from '@/components/xo/XOCell.vue';
import { Game } from '@/store/models';
@Component({
  components: { 'xo-cell': XOCell },
})
export default class XOBoard extends Vue {
  @Prop() game!: Game;
  @Prop({ default: false }) disabled!: boolean;

  submitting: boolean = false;

  get boardArray(): string[] {
    const boardArray = this.game.game_board.split('');
    return boardArray;
  }

  get winningCells(): number[] {
    if (this.game.game_status === 'P1-WIN') {
      return this.getWinState('X', this.boardArray);
    } else if (this.game.game_status === 'P2-WIN') {
      return this.getWinState('O', this.boardArray);
    }
    return [];
  }

  get cellStyles() {
    return this.boardArray.map((cell, index) => {
      return ({
        'unmarked': (cell === '-'),
        'can-select': this.canSelect(cell),
        'is-winning': this.winningCells.includes(index),
        'has-perspective': this.hasPerspective(cell),
      });
    });
  }

  getWinState(marker: string, board: string[]) {
    const winStates = [
      [0, 1, 2], [3, 4, 5], [6, 7, 8],
      [0, 3, 6], [1, 4, 7], [2, 5, 8],
      [0, 4, 8], [2, 4, 6],
    ];
    for (const win of winStates) {
      if ((board[win[0]] === marker)
        && (board[win[1]] === marker)
        && (board[win[2]] === marker)) {
        return win;
        break;
      }
    }
    return [];
  }

  /**
   * A cell marked 'X' has the 'has-perspective' class if the user is either
   * player one or a spectator. If the user is player two, 'O' marked cells
   * have perspective. If the user is able to become player two by marking a
   * cell, 'X' cells do not have perspective.
   */
  hasPerspective(cell: string) {
    if (cell === '-') {
      return false;
    }

    const publicKey = this.$store.getters['user/getPublicKey'];

    if (cell === 'O') {
      if (this.game.player_2.publicKey === publicKey) {
        return true;
      }
    } else if (cell === 'X') {
      if (this.game.player_1.publicKey === publicKey) {
        return true;
      }

      if (this.game.player_2) {
        if (this.game.player_2.publicKey !== publicKey) {
          return true;
        }
      }
    }
    return false;
  }

  canSelect(cell: string): boolean {
    if (this.disabled) {
      return false;
    }
    if (this.submitting) {
      return false;
    }

    const publicKey = this.$store.getters['user/getPublicKey'];
    if (!this.game.player_2 && this.game.player_1.publicKey === publicKey) {
      return false;
    }
    if (cell === '-') {
      if (this.game.game_status === 'P1-NEXT') {
        if ((this.game.player_1.publicKey === publicKey) || (!this.game.player_1)) {
          return true;
        }
      } else if (this.game.game_status === 'P2-NEXT') {
        if ((this.game.player_2.publicKey === publicKey) || (!this.game.player_2)) {
          return true;
        }
      }
    }
    return false;
  }

  async markCell(cellIndex: number) {
    if (this.canSelect(this.boardArray[cellIndex])) {
      this.submitting = true;
      try {
        const gameName = this.game.game_name;
        const circuitID = this.game.circuit_id;
        await this.$store.dispatch('games/take', { gameName, cellIndex, circuitID });
      } catch (e) {
        console.error(e);
        this.$emit('error', e.message);
      }
      this.submitting = false;
    }
  }
}
</script>

<style lang="scss" scoped>
  @import '@/scss/components/xo/_xo-board.scss';
</style>
