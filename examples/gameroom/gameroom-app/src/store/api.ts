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
  User,
  UserCredentials,
  UserAuthResponse,
  NewGameroomProposal,
  Node,
} from './models';
import router from '@/router';

export const gameroomAPI = axios.create({
  baseURL: '/api',
});

// Users
export async function userCreate(
  user: User,
): Promise<UserAuthResponse|undefined> {
  try {
    const response = await gameroomAPI.post('/users', user);
    router.push('/');
    return response.data as UserAuthResponse;
  } catch (e) {
    alert(e);
  }
}

export async function userAuthenticate(
  userCredentials: UserCredentials,
): Promise<UserAuthResponse|undefined> {
  try {
    const response = await gameroomAPI.post('/users/authenticate', userCredentials);
    router.push('/');
    return response.data as UserAuthResponse;
  } catch (e) {
    alert(e);
  }
}

// Gamerooms
export async function gameroomPropose(
  gameroomProposal: NewGameroomProposal,
): Promise<number|undefined> {
  try {
    const response = await gameroomAPI.post('/gamerooms/propose', gameroomProposal);
    return response.status;
  } catch (e) {
    alert(e);
  }
}

// Nodes
export async function listNodes(): Promise<Node[]> {
  try {
    const response = await gameroomAPI.get('/nodes');
    return response.data.data as Node[];
  } catch (e) {
    alert(e);
  }
  return [];
}


// Proposals
export async function listProposals(): Promise<GameroomProposal[]> {
  try {
    const response = await gameroomAPI.get('/proposals');
    return response.data.data as GameroomProposal[];
  } catch (e) {
    alert(e);
  }
  return [];
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
