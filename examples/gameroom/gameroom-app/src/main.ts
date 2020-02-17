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

import './class-component-hooks';

import Vue from 'vue';
import App from '@/App.vue';
import router from '@/router';
import store from '@/store';
import VueNativeSock from 'vue-native-websocket';

Vue.config.productionTip = false;

new Vue({
  router,
  store,
  render: (h) => h(App),
}).$mount('#app');

const protocol = ('https:' === document.location.protocol ? 'wss' : 'ws');

Vue.use(VueNativeSock, `${protocol}://${window.location.host}/ws/subscribe`, {
  store,
  format: 'json',
  reconnection: true,
  reconnectionAttempts: 30,
  reconnectionDelay: 10,
});

// When the page loads, the element with this directive gains focus.
Vue.directive('focus', {
  inserted: (el) => {
    el.focus();
  },
});
