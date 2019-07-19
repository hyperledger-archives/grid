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

import * as sawtoothSDK from 'sawtooth-sdk';
import sjcl from 'sjcl';

const signing = sawtoothSDK.signing;

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
export function createKeyPair(password: string) {
  const privateKey = CRYPTO_CONTEXT.newRandomPrivateKey();
  const signer = CRYPTO_FACTORY.newSigner(privateKey);
  const encryptedPrivateKey = JSON.stringify(
    sjcl.encrypt(password, privateKey.asHex()));
  const publicKey = signer.getPublicKey().asHex();
  return({publicKey, encryptedPrivateKey});
}
