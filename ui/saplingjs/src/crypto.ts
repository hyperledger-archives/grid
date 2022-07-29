/**
 * Copyright 2018-2020 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
import sjcl from 'sjcl';

/**
 * Encrypts a private key.
 * @param password - Encryption key.
 * @param privateKey - Unencrypted private key.
 */
export function encryptKey(privateKey: string, password: string): string {
  return JSON.stringify(sjcl.encrypt(password, privateKey));
}

/**
 * Decrypts a private key.
 * @param password - Encryption key.
 * @param encryptedPrivateKey - Encrypted private key.
 */
export function decryptKey(
  encryptedPrivateKey: string,
  password: string
): string {
  return sjcl.decrypt(password, encryptedPrivateKey);
}
