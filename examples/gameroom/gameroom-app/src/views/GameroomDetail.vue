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
          <div class="header">
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
        <button  :disabled="gameroom.status !== 'Active'" class="btn-action right">
          <div class="btn-text">New Game</div>
        </button>
        </div>
        <div class="filter-container">
          <input class="form-input form-filter"
                  :disabled="gameroom.status !== 'Active'"
                  v-model="gameNameFilter" type="text"
                  placeholder="Filter name..."
                  @input="filterGamesByName" />
        </div>
        <div class="cards-container" v-if="filteredGames.length > 0">
          <ul id="example-1">
            <li v-for="(index, game) in filteredGames" :key="index">
              {{ game.game_name }}
            </li>
          </ul>
         </div>
         <div class="placeholder-wrapper" v-else>
           <h3 class="tbl-placeholder"> {{ placeholderText }} </h3>
           <div v-if="gameroom.status !== 'Active'" class="spinner-gameroom spinner" />
         </div>
       </div>
  </div>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';
import gamerooms from '@/store/modules/gamerooms';
import games from '@/store/modules/gamerooms';
import { gameIsOver, userIsInGame, userCanJoinGame} from '@/utils/xo-games';
import { Gameroom, Member, Game } from '@/store/models';

@Component
  export default class GameroomDetails extends Vue {
      gameNameFilter = '';
      currentTab = 1;

      mounted() {
        gamerooms.listGamerooms();
        this.$store.dispatch('games/listGames', this.$route.params.id);
      }

      get gameroom(): Gameroom {
        return gamerooms.gameroomList.find(
              (gameroom) => gameroom.circuit_id ===  this.$route.params.id) || {} as Gameroom;
      }

      get games(): Game[] {
        return this.$store.getters['games/getGames'];
      }

      get gemeroomMembers() {
        if (this.gameroom.members) {
          const organizations = this.gameroom.members.map((member) => member.organization);
          return organizations.join(', ');
        }
      }

      get placeholderText(): string {
       if (this.gameroom.status === 'Active') {
         return 'No games to show.';
       } else {
         return 'Please wait while your gameroom finishes setting up.';
       }
     }

      // intersection of filteredGamesByName and filteredGamesByState
      get filteredGames() {
        const filteredGamesByState = this.filterGamesByState;
        return this.filterGamesByName.filter((game, index, array) => filteredGamesByState.indexOf(game) !== -1);
      }

      get filterGamesByName() {
        return this.games.filter((game, index, array) =>
          game.game_name.toUpperCase().indexOf(this.gameNameFilter.toUpperCase()) > -1);
      }


      get filterGamesByState() {
        const publicKey = this.$store.getters['user/getPublicKey'];
        switch (this.currentTab) {
          case 5:
             return this.games.filter((game, index, array) => gameIsOver(game.game_status));
           case 3:
              return this.games.filter((game, index, array) =>
                !userIsInGame(game, publicKey) && userCanJoinGame(game, publicKey));
           case 2:
              return this.games.filter((game, index, array) =>
                !gameIsOver(game.game_status) && userIsInGame(game, publicKey));
           case 4:
              return this.games.filter((game, index, array) =>
                !gameIsOver(game.game_status) && !userIsInGame(game, publicKey) && !userCanJoinGame(game, publicKey));
           default:
            return this.games;
        }
    }

    selectTab(tab: number) {
      this.currentTab = tab;
    }
  }

</script>

<style lang="scss" scoped>
@import '@/scss/components/_gameroom-details.scss';
</style>
