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
    <toast toast-type="error" :active="error" v-on:toast-action="clearError">
      {{ error }}
    </toast>
    <div class="auth-wrapper">
      <form class="auth-form" @submit.prevent="register">
        <label class= "form-label">
          <div>Email</div>
          <input
            class="form-input"
            type="email"
            v-model="email"
            v-focus
          />
        </label>
        <label class="form-label">
          <div>Password</div>
          <input
            class="form-input"
            type="password"
            v-model="password"
          />
        </label>
        <label class="form-label">
          <div>Confirm Password</div>
          <input
            class="form-input"
            type="password"
            v-model="confirmPassword"
          />
        </label>
        <div class="submit-container">
          <button class="btn-action large" type="submit" :disabled="!canSubmit">
            <div v-if="submitting" class="spinner" />
            <div v-else> Register </div>
          </button>
          <div class="form-link">
            Already have an account?
            <router-link to="/login">
              Click here to log in.
            </router-link>
          </div>
        </div>
      </form>
    </div>
  </div>
</template>

<script lang="ts">
import { Vue, Component } from 'vue-property-decorator';
import * as crypto from '@/utils/crypto';
import Toast from '../components/Toast.vue';

@Component({
  components: { Toast },
})
export default class Register extends Vue {
  email = '';
  password = '';
  confirmPassword = '';
  submitting = false;
  error = '';

  get canSubmit() {
    if (!this.submitting &&
        this.email !== '' &&
        this.password !== '' &&
        this.confirmPassword !== '') {
      return true;
    }
    return false;
  }

  clearError() {
    this.error = '';
  }

  async register() {
    if (this.password !== this.confirmPassword) {
      this.error = 'Passwords do not match.';
      return;
    }
    const keys = crypto.createKeyPair(this.password);
    this.submitting = true;
    try {
      await this.$store.dispatch('user/register', {
        email: this.email,
        hashedPassword: crypto.hashSHA256(this.email, this.password),
        publicKey: keys.publicKey,
        encryptedPrivateKey: keys.encryptedPrivateKey,
      });
    } catch (e) {
      this.error = e.message;
    }
    this.submitting = false;
  }
}
</script>
