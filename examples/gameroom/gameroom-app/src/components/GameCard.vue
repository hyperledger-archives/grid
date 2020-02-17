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
  <div class="card-container" @click="goToGame()" :class="[committed ? 'committed': 'not-committed']">
    <div class="game-preview-container">
      <xo-board v-if="committed" :game="game" class="small" :disabled='true'/>
      <span v-else class="spinner-gameroom spinner" />
    </div>
    <div class="game-details-containter">
      <h3 class="header">
          {{ game.game_name }}
      </h3>
      <div class="body">
        <div class="players-container">
         <div  class="player-container">
           <i class="material-icons player-icon">
             person
           </i>
           <div v-if="game.player_1" class="player-data-wrapper">
             <span>{{game.player_1.name}}</span>
             <span>{{game.player_1.organization}}</span>
          </div>
         </div>
         <div  class="vs-wrapper">
           <i class="material-icons">
            clear
           </i>
         </div>
         <div class="player-container">
           <i class="material-icons player-icon">
             person
           </i>
           <div v-if="game.player_2"  class="player-data-wrapper">
             <span>{{game.player_2.name}}</span>
             <span>{{game.player_2.organization}}</span>
          </div>
         </div>
       </div>
       </div>
       <div class="game-status" :class="[canUserTakeAction() ? 'take-action' : 'wait-action']">
         {{status_message}}
       </div>
     </div>
     <div class="footer">
       {{ lastUpdate() }}
     </div>
    </div>
</template>

<script lang="ts">
import { Vue, Prop, Component } from 'vue-property-decorator';
import * as moment from 'moment';
import { GameroomProposal, Game } from '../store/models';
import XOBoard from '@/components/xo/XOBoard.vue';
import { gameIsOver, userIsInGame, userCanJoinGame, isUserTurn} from '@/utils/xo-games';
import proposals from '@/store/modules/proposals';
import gamerooms from '@/store/modules/gamerooms';

@Component({
  components: { 'xo-board': XOBoard },
})
export default class GameCard extends Vue {
  @Prop() game!: Game;

  publicKey = this.$store.getters['user/getPublicKey'];

  get status_message(): string {
    const publicKey = this.$store.getters['user/getPublicKey'];
    if (!this.game.committed) {
      return 'creating game';
    }
    if (gameIsOver(this.game.game_status)) {
      return this.processGameOverStatus();
    } else if (!userIsInGame(this.game, publicKey) && userCanJoinGame(this.game, publicKey)) {
      return 'join game';
    } else if (userIsInGame(this.game, publicKey)) {
      if ((this.game.game_status === 'P1-NEXT' && this.game.player_1.publicKey === publicKey)
          || (this.game.game_status === 'P2-NEXT' && this.game.player_2.publicKey === publicKey)) {
        return 'your turn';
      } else if (this.game.game_status === 'P2-NEXT' && !this.game.player_2) {
        return 'waiting for player to join';
      } else {
        return 'their turn';
      }
    } else if (!userIsInGame(this.game, publicKey) && !userCanJoinGame(this.game, publicKey)) {
      return this.processWatchStatus();
    } else {
      return 'invalid status';
    }
  }

  processGameOverStatus(): string {
    if (this.game.game_status === 'TIE') {
      return 'Tie';
    }
    if (userIsInGame(this.game, this.publicKey)) {
      if ((this.game.player_1.publicKey === this.publicKey && this.game.game_status === 'P1-WIN') ||
          (this.game.player_2.publicKey === this.publicKey && this.game.game_status === 'P2-WIN')
    ) {
        return 'You won';
      } else {
        return 'You lost';
      }
    }

    if (this.game.game_status === 'P1-WIN') {
      return `${this.game.player_1.name} won`;
    }
    if (this.game.game_status === 'P2-WIN') {
      return `${this.game.player_2.name} won`;
    }
    return 'Archived';
  }

  processWatchStatus(): string {
    if (this.game.game_status === 'P1-NEXT') {
      return `${this.game.player_1.name}\'s turn`;
    }
    if (this.game.game_status === 'P2-NEXT') {
      return `${this.game.player_2.name}\'s turn`;
    }
    return 'Watch';
  }


  get committed(): boolean {
    return this.game.committed;
  }

  goToGame() {
    if (this.committed) {
      this.$router.push({name: 'games', params: {id: `${this.game.circuit_id}`, gameNameHash: `${this.game.game_name_hash}`}});
    }
  }

  lastUpdate(): string {
    if (this.game.created_time === this.game.updated_time) {
      return `Created: ${this.fromNow(this.game.created_time)}`;
    } else {
        return `Last move: ${this.fromNow(this.game.updated_time)}`;
    }
  }

  fromNow(timestamp: number): string {
    return moment.unix(timestamp).fromNow();
  }

  // Return true if the user can join a game or make a move
   canUserTakeAction(): boolean {
    if (!this.game.committed) {
      return false;
    }
    if ((!userIsInGame(this.game, this.publicKey) && userCanJoinGame(this.game, this.publicKey))
        || isUserTurn(this.game, this.publicKey)) {
      return true;
    } else {
      return false;
    }
  }
}
</script>

<style lang="scss" scoped>
  @import '@/scss/components/_game-card.scss';
</style>
