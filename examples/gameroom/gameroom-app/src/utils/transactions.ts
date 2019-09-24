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

import protos from '@/protobuf';
import { User } from '@/store/models';
import { signXOPayload } from '@/utils/crypto';
import { XO_FAMILY_NAME, XO_FAMILY_VERSION, XO_FAMILY_PREFIX } from '@/utils/addressing';
import { calculateNamespaceRegistryAddress, computeContractAddress, computeContractRegistryAddress } from '@/utils/addressing';

const crypto = require('crypto');
const { Transaction, TransactionHeader, Batch, BatchHeader, BatchList } = require('sawtooth-sdk/protobuf');

// The Sawtooth Sabre transaction family name (sabre)
const SABRE_FAMILY_NAME = 'sabre';
// The Sawtooth Sabre transaction family version (0.4)
const SABRE_FAMILY_VERSION = '0.4';


export function createTransaction(payloadBytes: Uint8Array, inputs: string[], outputs: string[], user: User) {
  const excuteTransactionAction = protos.ExecuteContractAction.create({
    name: 'xo',
    version: XO_FAMILY_VERSION,
    inputs,
    outputs,
    payload: payloadBytes,
  });

  const sabrePayload = protos.SabrePayload.encode({
    action: protos.SabrePayload.Action.EXECUTE_CONTRACT,
    executeContract: excuteTransactionAction,
  }).finish();

  const transactionHeaderBytes = TransactionHeader.encode({
    familyName: SABRE_FAMILY_NAME,
    familyVersion: SABRE_FAMILY_VERSION,
    inputs: prepare_inputs(inputs),
    outputs,
    signerPublicKey: user.publicKey,
    batcherPublicKey: user.publicKey,
    dependencies: [],
    payloadSha512: crypto.createHash('sha512').update(sabrePayload).digest('hex'),
  }).finish();

  const signature = signXOPayload(transactionHeaderBytes, user.privateKey);

  return Transaction.create({
    header: transactionHeaderBytes,
    headerSignature: signature,
    payload: sabrePayload,
  });
}


export function createBatch(transactions: any, user: User) {
  const transactionIds = transactions.map((txn: any) => txn.headerSignature);
  const batchHeaderBytes = BatchHeader.encode({
    signerPublicKey: user.publicKey,
    transactionIds,
  }).finish();

  const signature = signXOPayload(batchHeaderBytes, user.privateKey);

  const batch = Batch.create({
    header: batchHeaderBytes,
    headerSignature: signature,
    transactions,
  });

  const batchListBytes = BatchList.encode({
    batches: [batch],
  }).finish();

  return batchListBytes;
}

function prepare_inputs(contractAddresses: string[]) {
  const returnAddresses = [
    computeContractRegistryAddress(XO_FAMILY_NAME),
    computeContractAddress(XO_FAMILY_NAME, XO_FAMILY_VERSION),
    calculateNamespaceRegistryAddress(XO_FAMILY_PREFIX),
  ];

  return returnAddresses.concat(contractAddresses);

}
