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

import { GameroomNotification } from '@/store/models';
import { listNotifications, markRead } from '@/store/api';

export interface NotificationState {
  notifications: GameroomNotification[];
}

const mockNotifications: GameroomNotification[] = [
  {
    id: 1,
    notification_type: 'invite',
    org: 'ACME Corporation',
    target: 'acme:bubba',
    timestamp: 1565735940,
    read: false,
  },
  {
    id: 2,
    notification_type: 'invite',
    org: 'ACME Corporation',
    target: 'acme:asdforg',
    timestamp: 1565732000,
    read: false,
  },
  {
    id: 3,
    notification_type: 'invite',
    org: 'Bubba Bakery',
    target: 'bubba:asdforg',
    timestamp: 1465732000,
    read: false,
  },
];

const notificationState = {
  notifications: ([] as GameroomNotification[]),
};

const getters = {
  getNotifications(state: NotificationState): GameroomNotification[] {
    return state.notifications.sort(
      (a: GameroomNotification, b: GameroomNotification) => {
        return (b.timestamp - a.timestamp);  // Newest first
      },
    );
  },
  getNewNotificationCount(state: NotificationState) {
    const count = state.notifications.filter(
      (notification) => !notification.read).length;
    return count;
  },
};

const actions = {
  async listNotifications({ commit }: any) {
    const notifications = await listNotifications();
    commit('setNotifications', notifications);
  },
  async markRead({ commit }: any, id: string) {
    const update = await markRead(id);
    if (update) {
      commit('updateNotification', update);
    }
  },
  listNotificationsMock({ commit }: any) {
    commit('setNotifications', mockNotifications);
  },
  markReadMock({ commit }: any, id: number) {
    const update = mockNotifications.find(
      (notification) => notification.id === id);
    if (update) {
      update.read = true;
      commit('updateNotification', update);
    }
  },
};

const mutations = {
  setNotifications(state: NotificationState, notifications: GameroomNotification[]) {
    state.notifications = notifications;
  },
  addNotification(state: NotificationState, notification: GameroomNotification) {
    state.notifications.push(notification);
  },
  updateNotification(state: NotificationState, update: GameroomNotification) {
    const index = state.notifications.findIndex((notif) => notif.id === update.id);
    if (index !== -1) {
      state.notifications.splice(index, 1, update);
    }
  },
};

export default {
  namespaced: true,
  name: 'notifications',
  state: notificationState,
  getters,
  actions,
  mutations,
};
