// Copyright 2018-2020 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { Game } from '@/store/models';

const crypto = require('crypto');

export function gameIsOver(gameStatus: string) {
    return gameStatus === 'P1-WIN' || gameStatus === 'P2-WIN' || gameStatus === 'TIE';
  }

export function userIsInGame(game: Game, publicKey: string) {
   return game.player_1.publicKey === publicKey || game.player_2.publicKey === publicKey;
 }

export function userCanJoinGame(game: Game, publicKey: string) {
    return !game.player_1 || (!game.player_2 && game.player_1.publicKey !== publicKey);
}

export function hashGameName(gameName: string) {
  return crypto.createHash('md5').update(gameName).digest('hex');
}

export function isUserTurn(game: Game, publicKey: string): boolean {
  if (userIsInGame(game, publicKey)) {
    if ((game.game_status === 'P1-NEXT' && game.player_1.publicKey === publicKey)
        || (game.game_status === 'P2-NEXT' && game.player_2.publicKey === publicKey)) {
      return true;
    }
  }
  return false;
}
