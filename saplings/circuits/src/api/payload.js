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

import { Secp256k1Signer, Secp256k1PrivateKey } from 'transact-sdk-javascript';

import crypto from 'crypto';
import protos from '../protobuf';

export const makeSignedPayload = (
  localNodeID,
  privateKey,
  action,
  actionType
) => {
  let actionBytes = null;
  let actionEnum = null;
  const payload = {};

  const secp256PrivateKey = Secp256k1PrivateKey.fromHex(privateKey);
  const signer = new Secp256k1Signer(secp256PrivateKey);

  switch (actionType) {
    case 'proposeCircuit': {
      actionBytes = protos.CircuitCreateRequest.encode(action).finish();
      payload.circuitCreateRequest = action;
      actionEnum =
        protos.CircuitManagementPayload.Action.CIRCUIT_CREATE_REQUEST;
      break;
    }
    case 'voteCircuitProposal': {
      actionBytes = protos.CircuitProposalVote.encode(action).finish();
      payload.circuitProposalVote = action;
      actionEnum = protos.CircuitManagementPayload.Action.CIRCUIT_PROPOSAL_VOTE;
      break;
    }
    default:
      throw new Error(`unhandled action type: ${action.type}`);
  }
  const hashedBytes = crypto.createHash('sha512').update(actionBytes);
  const header = protos.CircuitManagementPayload.Header.encode({
    action: actionEnum,
    payloadSha512: hashedBytes,
    requesterNodeId: localNodeID,
    requester: [...signer.getPublicKey().asBytes()]
  }).finish();
  const signature = signer.sign(header);

  const serializedPayload = protos.CircuitManagementPayload.encode({
    ...payload,
    header,
    signature: Buffer.from(signature, 'hex')
  }).finish();

  return serializedPayload;
};
