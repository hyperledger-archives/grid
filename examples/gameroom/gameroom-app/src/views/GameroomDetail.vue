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
      <span> {{ gemeroomMembers }} </span>
    </div>
        <div class="data-container">
          <div class="tab-buttons">
          <button class="tab-button"
                  @click="selectTab(1)"
                  :disabled="gameroom.status !== 'Active'"
                  :class="{ 'is-active' : currentTab === 1 }">
            <div class="btn-text">all</div>
          </button >
          <button class="tab-button"
                  @click="selectTab(2)"
                  :disabled="gameroom.status !== 'Active'"
                  :class="{ 'is-active' : currentTab === 2 }">
            <div class="btn-text">your games</div>
          </button>
          <button class="tab-button"
                  @click="selectTab(3)"
                  :disabled="gameroom.status !== 'Active'"
                  :class="{ 'is-active' : currentTab === 3 }">
            <div class="btn-text">join</div>
          </button>
          <button class="tab-button"
                  @click="selectTab(4)"
                  :disabled="gameroom.status !== 'Active'"
                  :class="{ 'is-active' : currentTab === 4 }">
            <div class="btn-text">watch</div>
          </button>
          <button class="tab-button"
                  @click="selectTab(5)"
                  :disabled="gameroom.status !== 'Active'"
                  :class="{ 'is-active' : currentTab === 5 }">
            <div class="btn-text">archived</div>
          </button>
        </div>
       </div>
  </div>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';
import gamerooms from '@/store/modules/gamerooms';
import { Gameroom, Member, Game } from '@/store/models';

@Component
  export default class GameroomDetails extends Vue {
      games: Game[] = [];
      filteredGamesByState = this.games;
      currentTab = 1;

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

      get gemeroomMembers() {
        if (this.gameroom.members) {
          const organizations = this.gameroom.members.map((member) => member.organization);
          return organizations.join(', ');
        }
      }

       selectTab(tab: number) {
         this.currentTab = tab;
         this.filterGamesByState(tab);
       }

      filterGamesByState(tab: number) {
        const publicKey = this.$store.getters['user/getPublicKey'];
        let filteredGames: Game[] = [];
        switch (tab) {
          case 5:
             filteredGames = this.games.filter((game, index, array) => gameIsOver(game.game_status));
             break;
           case 3:
              filteredGames = this.games.filter((game, index, array) =>
                !userIsInGame(game, publicKey) && userCanJoinGame(game, publicKey));
              break;
           case 2:
              filteredGames = this.games.filter((game, index, array) =>
                !gameIsOver(game.game_status) && userIsInGame(game, publicKey));
              break;
           case 4:
              filteredGames = this.games.filter((game, index, array) =>
                !gameIsOver(game.game_status) && !userIsInGame(game, publicKey) && !userCanJoinGame(game, publicKey));
              break;
           default:
            filteredGames =  this.games;
        }
        this.filteredGamesByState = filteredGames;
    }
  }

function gameIsOver(gameStatus: string) {
  return gameStatus === 'P1-WIN' || gameStatus === 'P2-WIN' || gameStatus === 'TIE';
}

function userIsInGame(game: Game, publicKey: string) {
  return game.player_1 === publicKey || game.player_2 === publicKey;
}

function userCanJoinGame(game: Game, publicKey: string) {
  return game.player_1 === '' || (game.player_2 === '' && game.player_1 !== publicKey);
}

</script>

<style lang="scss" scoped>
@import '@/scss/components/_gameroom-details.scss';
</style>
