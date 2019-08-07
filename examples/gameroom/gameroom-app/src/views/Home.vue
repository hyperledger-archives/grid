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
  <gamerooms v-if="isLoggedIn" />
  <div v-else>
    <h1>Welcome to Gameroom</h1>
  </div>
</template>

<script>
import { Vue, Component } from 'vue-property-decorator';
import Gamerooms from '@/views/Gamerooms';

@Component({
  components: { Gamerooms },
})
export default class Home extends Vue {
  get isLoggedIn() {
    return this.$store.getters['user/isLoggedIn'];
  }
}
</script>
