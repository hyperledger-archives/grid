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
  <div class="auth-container">
    <div class="auth-wrapper">
      <form class="auth-form" @submit.prevent="login">
        <label class= "form-label">
          Email
          <input
            class="form-input"
            type="email"
            v-model="email"
          />
        </label>
        <label class="form-label">
          Password
          <input
            class="form-input"
            type="password"
            v-model="password"
          />
        </label>
        <button class="btn-action form-button" type="submit" :disabled="!canSubmit">
          <div v-if="submitting"> Logging in... </div>
          <div v-else> Log In </div>
        </button>
        <span class="form-link">
          Don't have an account yet?
          <router-link to="/register">
            Click here to register.
          </router-link>
        </span>
      </form>
    </div>
  </div>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';
import * as crypto from '@/utils/crypto';

@Component
export default class Login extends Vue {
  email = '';
  password = '';
  submitting = false;

  get canSubmit() {
    if (!this.submitting &&
        this.email !== '' &&
        this.password !== '') {
      return true;
    }
    return false;
  }

  async login() {
    this.submitting = true;
    await this.$store.dispatch('user/authenticate', {
      email: this.email,
      hashedPassword: crypto.hashSHA256(this.email, this.password),
    });
    this.submitting = false;
  }
}
</script>
