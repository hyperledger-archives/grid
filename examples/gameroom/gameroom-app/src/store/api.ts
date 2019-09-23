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
  Gameroom,
  Ballot,
  Game,
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

export async function listGamerooms(): Promise<Gameroom[]> {
  const response = await gameroomAPI.get('/gamerooms');
  const gamerooms = response.data.data.map((gameroom: any) => {
    const members = gameroom.members.map(async (member: any) => {
      const node = await getNode(member.node_id);
      member.organization = node.metadata.organization;
      return member as Member;
    });
    Promise.all(members).then((m) => gameroom.members = m);
    return gameroom as Gameroom;
  });
  return response.data.data as Gameroom[];
}

export async function fetchGameroom(circuitID: string): Promise<Gameroom> {
  const response = await gameroomAPI.get(`/gamerooms/${circuitID}`);
  return response.data as Gameroom;
}

// Nodes
export async function listNodes(): Promise<Node[]> {
  const response = await gameroomAPI.get('/nodes');
  return response.data.data as Node[];
}

export async function listGames(circuitID: string): Promise<Game[]> {
  const response = await gameroomAPI.get(`/xo/${circuitID}/games`);
  return response.data.data as Game[];
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
    console.error(err.message);
    throw new Error('Failed to send request. Contact administrator for help.');
  });
}

export async function submitBatch(payload: Uint8Array, circuitID: string): Promise<void> {
  const options = {
    method: 'POST',
    url: `http://${window.location.host}/api/gamerooms/${circuitID}/batches`,
    body: payload,
    headers: {
      'Content-Type': 'application/octet-stream',
    },
  };

  await rp(options).then((body) => {
    return;
  })
  .catch((err) => {
    console.error(err.message);
    throw new Error('Failed to send request. Contact administrator for help.');
  });
}

// Proposals
export async function listProposals(): Promise<GameroomProposal[]> {
  const response = await gameroomAPI.get('/proposals');

  const getMembers = async (member: any) => {
    const node = await getNode(member.node_id);
    member.organization = node.metadata.organization;
    return member as Member;
  };

  const combineProposal = async (proposal: any) => {
    const gameroom = await fetchGameroom(proposal.circuit_id);
    proposal.status = gameroom.status;

    const requester = await getNode(proposal.requester_node_id);
    proposal.requester_org = requester.metadata.organization;

    const members = await Promise.all(
      proposal.members.map((member: any) => getMembers(member)));
    proposal.members = members;
    return proposal;
  };

  return await Promise.all(
    response.data.data.map((proposal: GameroomProposal) => combineProposal(proposal)));
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
const getOrgName = async (notif: any) => {
  const node = await getNode(notif.node_id);
  notif.requester_org = node.metadata.organization;
  return notif as GameroomNotification;
};

export async function listNotifications(publicKey: string): Promise<GameroomNotification[]> {
  const isDisplayed = (value: GameroomNotification): boolean => {
    const displayedNotifs = ['gameroom_proposal', 'circuit_active'];
    if (displayedNotifs.includes(value.notification_type)) {
      if (value.notification_type === 'gameroom_proposal' && value.requester === publicKey) {
        return false;
      }
      return true;
    } else { return false; }
  };

  const response = await gameroomAPI.get('/notifications');
  const notifications = response.data.data as GameroomNotification[];
  const filtered = notifications.filter(isDisplayed);
  return await Promise.all(filtered.map((notif: any) => getOrgName(notif)));
}

export async function markRead(id: string): Promise<GameroomNotification> {
  const response = await gameroomAPI.patch(`/notifications/${id}/read`);
  const notif = response.data.data;
  const node = await getNode(notif.node_id);
  notif.requester_org = node.metadata.organization;
  return notif as GameroomNotification;
}
