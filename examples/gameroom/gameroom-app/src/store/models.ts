// Copyright 2019 Cargill Incorporated
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

export interface ApiError {
  status: number;
  message: string;
}

export interface User {
  email: string;
  publicKey: string;
  privateKey: string;
}

export interface UserRegistration {
  email: string;
  hashedPassword: string;
  publicKey: string;
  encryptedPrivateKey: string;
}

export interface UserCredentials {
  email: string;
  hashedPassword: string;
}

export interface UserAuthResponse {
  email: string;
  publicKey: string;
  encryptedPrivateKey: string;
}

export interface Node {
  identity: string;
  metadata: {
    organization: string;
    endpoint: string;
  };
}

export interface NewGameroomProposal {
  alias: string;
  member: [Node];
}

export interface Member {
  node_id: string;
  organization: string;
  endpoint: string;
}

export interface GameroomProposal {
  proposal_id: string;
  circuit_id: string;
  circuit_hash: string;
  members: Member[];
  requester: string;
  created_time: number;
  updated_time: number;
  requester_org: string;
  status: string;
}

export interface GameroomNotification {
  id: number;
  notification_type: string;
  org: string;
  requester: string;
  target: string;
  timestamp: number;
  read: boolean;
  node_id: string;
  requester_org: string;
}

export interface Section {
  name: string;
  icon: string;
  active: boolean;
  link: string;
  dropdown: boolean;
  action: boolean;
  actionIcon: string;
}

export interface Ballot {
  circuit_id: string;
  circuit_hash: string;
  vote: string;
}

export interface Gameroom {
  circuit_id: string;
  authorization_type: string;
  persistence: string;
  routes: string;
  members: Member[];
  circuit_management_type: string;
  alias: string;
  status: string;
}

export interface Game {
  id: number;
  game_name: string;
  game_name_hash: string;
  circuit_id: string;
  player_1: Player;
  player_2: Player;
  game_status: string;
  game_board: string;
  created_time: number;
  updated_time: number;
  commited: boolean;
}

export interface Player {
  name: string;
  publicKey: string;
  organization: string;
}
