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

import sjcl from 'sjcl';
import protos from '@/protobuf';

const signing = require('sawtooth-sdk/signing');
const { Secp256k1PrivateKey } = require('sawtooth-sdk/signing/secp256k1');

const CRYPTO_CONTEXT = signing.createContext('secp256k1');
const CRYPTO_FACTORY = new signing.CryptoFactory(CRYPTO_CONTEXT);

/**
 * Returns the SHA-256 hash of the provided salt and data.
 *
 * @param salt - The salt used in the creation of the hash
 * @param data - The data to be hashed
 * @returns The SHA-256 hash of the salt and data
 */
export function hashSHA256(salt: string, data: string): string {
  const out = sjcl.hash.sha256.hash(salt + data);
  return sjcl.codec.hex.fromBits(out);
}

/**
 * Creates a new secp256k1 key pair and encrypts the private key using the
 * provided password.
 *
 * @param password - The password or key.
 * @returns An object containing the public key as hex and the ciphertext of
 *  the encrypted private key.
 */
export function createKeyPair() {
  const privateKey = CRYPTO_CONTEXT.newRandomPrivateKey();
  const signer = CRYPTO_FACTORY.newSigner(privateKey);
  const publicKey = signer.getPublicKey().asHex();
  const privateKeyHex = privateKey.asHex();
  return({publicKey, privateKey: privateKeyHex});
}

/**
 * Encrypts a private key.
 * @param password - Encryption key.
 * @param privateKey - Unencrypted private key.
 */
export function encrypt(password: string, privateKey: string): string {
  return JSON.stringify(sjcl.encrypt(password, privateKey));
}

/**
 * Decrypts a private key.
 * @param password - Encryption key.
 * @param encryptedPrivateKey - Encrypted private key.
 */
export function decrypt(password: string, encryptedPrivateKey: string): string {
  return sjcl.decrypt(password, JSON.parse(encryptedPrivateKey));
}

/**
 * Fills out, signs, and encodes an incomplete CircuitManagementPayload.
 *
 * @param payload - The incomplete CircuitManagementPayload.
 * @param signer - Wrapper containing the user's keys.
 */
export function signPayload(payload: Uint8Array, privateKey: string): Uint8Array {
  const pkey = Secp256k1PrivateKey.fromHex(privateKey);
  const signer = CRYPTO_FACTORY.newSigner(pkey);

  const message = protos.CircuitManagementPayload.decode(payload);
  const header = protos.CircuitManagementPayload.Header.decode(message.header);

  const pubKey = signer.getPublicKey().asBytes();
  header.requester = pubKey;
  message.signature = signer.sign(header);
  message.header = protos.CircuitManagementPayload.Header.encode(header).finish();
  const signedPayload = protos.CircuitManagementPayload.encode(message).finish();
  return signedPayload;
}
