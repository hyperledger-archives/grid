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

import axios from 'axios';
import {
  GameroomNotification,
  GameroomProposal,
  UserRegistration,
  UserCredentials,
  UserAuthResponse,
  NewGameroomProposal,
  Node,
} from './models';

export const gameroomAPI = axios.create({
  baseURL: '/api',
});

gameroomAPI.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response) {
      switch (error.response.status) {
        case 401:
          throw new Error('Incorrect username or password.');
        case 500:
          throw new Error(
            'The Gameroom server has encountered an error. Please contact the administrator.',
          );
        case 503:
          throw new Error('The Gameroom server is unavailable. Please contact the administrator.');
        default:
          throw new Error(error.response.data);
      }
    }
  },
);

// Users
export async function userCreate(
  user: UserRegistration,
): Promise<UserAuthResponse> {
  const response = await gameroomAPI.post('/users', user);
  return response.data as UserAuthResponse;
}

export async function userAuthenticate(
  userCredentials: UserCredentials,
): Promise<UserAuthResponse> {
    const response = await gameroomAPI.post('/users/authenticate', userCredentials);
    return response.data as UserAuthResponse;
}

// Gamerooms
export async function gameroomPropose(
  gameroomProposal: NewGameroomProposal,
): Promise<number> {
  const response = await gameroomAPI.post('/gamerooms/propose', gameroomProposal);
  return response.status;
}

// Nodes
export async function listNodes(): Promise<Node[]> {
  const response = await gameroomAPI.get('/nodes');
  return response.data.data as Node[];
}


// Proposals
export async function listProposals(): Promise<GameroomProposal[]> {
  const response = await gameroomAPI.get('/proposals');
  return response.data.data as GameroomProposal[];
}


// Notifications
export async function listNotifications(): Promise<GameroomNotification[]> {
  try {
    const response = await gameroomAPI.get('/notifications');
    return response.data.data as GameroomNotification[];
  } catch (e) {
    alert(e);
  }
  return [];
}

export async function markRead(id: string): Promise<GameroomNotification|undefined> {
  try {
    const response = await gameroomAPI.patch(`/notifications/${id}/read`);
    return response.data as GameroomNotification;
  } catch (e) {
    alert(e);
  }
}
