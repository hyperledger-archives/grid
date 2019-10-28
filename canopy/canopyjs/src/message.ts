/**
 * Copyright 2019 Cargill Incorporated
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

import uuid from 'uuid/v4';

export interface ChildMessage {
  type: string;
  body: object | string;
}

export interface ParentMessage {
  status: string;
  id: string;
}

export function message(childMessage: ChildMessage): Promise<ParentMessage> {
  return new Promise(function WaitForMessage(resolve, reject) {
    const id = uuid();
    function handleMessage(e: MessageEvent): void {
      const { data }: { data: ParentMessage } = e;
      if (data.id === id) {
        window.removeEventListener('message', handleMessage, false);
        if (data.status === 'OK') {
          resolve(e.data);
        } else {
          reject(e.data);
        }
      }
    }
    window.addEventListener('message', handleMessage, false);
    window.parent.postMessage({ message: childMessage, id }, '*');
  });
}
