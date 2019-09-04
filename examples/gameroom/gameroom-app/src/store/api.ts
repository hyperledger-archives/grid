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
import rp from 'request-promise';
import {
  GameroomNotification,
  GameroomProposal,
  UserRegistration,
  UserCredentials,
  UserAuthResponse,
  NewGameroomProposal,
  Member,
  Node,
  Ballot,
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
          throw new Error(error.response.data.message);
      }
    }
  },
);

// Users
export async function userCreate(
  user: UserRegistration,
): Promise<UserAuthResponse> {
  const response = await gameroomAPI.post('/users', user);
  return response.data.data as UserAuthResponse;
}

export async function userAuthenticate(
  userCredentials: UserCredentials,
): Promise<UserAuthResponse> {
    const response = await gameroomAPI.post('/users/authenticate', userCredentials);
    return response.data.data as UserAuthResponse;
}

// Gamerooms
export async function gameroomPropose(
  gameroomProposal: NewGameroomProposal,
): Promise<Uint8Array> {
  const response = await gameroomAPI.post('/gamerooms/propose', gameroomProposal);
  return response.data.data.payload_bytes as Uint8Array;
}

// Nodes
export async function listNodes(): Promise<Node[]> {
  const response = await gameroomAPI.get('/nodes');
  return response.data.data as Node[];
}

// Payloads
export async function submitPayload(payload: Uint8Array): Promise<void> {
  const options = {
    method: 'POST',
    url: `http://${window.location.host}/api/submit`,
    body: payload,
    headers: {
      'Content-Type': 'application/octet-stream',
    },
  };

  await rp(options).then((body) => {
    return;
  })
  .catch((err) => {
    console.log(err.message);
    throw new Error("Failed to create gameroom. Contact administrator for help.");
  });
}

// Proposals
export async function listProposals(): Promise<GameroomProposal[]> {
  const response = await gameroomAPI.get('/proposals');

  const proposals = response.data.data.map((proposal: any) => {
    const members = proposal.members.map(async (member: any) => {
      const node = await getNode(member.identity);
      member.organization = node.metadata.organization;
      return member as Member;
    });
    proposal.members = members;
    return proposal as GameroomProposal;
  });

  return proposals as GameroomProposal[];
}

async function getNode(id: string): Promise<Node> {
    const response = await gameroomAPI.get(`/nodes/${id}`);
    return response.data.data as Node;
}

export async function proposalVote(ballot: Ballot, proposalID: string,
): Promise<Uint8Array> {
  const response = await gameroomAPI.post(`/proposals/${proposalID}/vote`, ballot);
  return response.data.data.payload_bytes as Uint8Array;
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
