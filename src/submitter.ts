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
interface BatchStatus {
  statusType: string;
  message: BatchMessage[];
}

interface BatchMessage {
  transactionId: string;
  errorMessage: string;
  errorData: number[];
}

interface BatchInfo {
  id: string;
  status: BatchStatus;
}

const HTTPMethods = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE'];

/**
 * Wrapper function to set up XHR.
 * @param {string}      method    HTTP method for the request
 * @param {string}      url       endpoint to make the request to
 * @param {Uint8Array}  data      Byte array representation of the request body
 * @param {function}    headerFn  Function to set the correct request headers
 */
async function http(
  method: string,
  url: string,
  data: Uint8Array | null,
  headerFn: (request: XMLHttpRequest) => void
): Promise<string> {
  return new Promise((resolve, reject) => {
    if (!HTTPMethods.includes(method.toUpperCase())) {
      reject(Error('Invalid HTTP Method'));
    }

    const request = new XMLHttpRequest();
    request.open(method, url);
    if (headerFn) {
      headerFn(request);
    }
    request.onload = (): void => {
      if (request.status >= 200 && request.status < 300) {
        resolve(request.response);
      } else if (request.status >= 400 && request.status < 500) {
        reject(
          Error('Failed to send request. Contact the administrator for help.')
        );
      } else {
        reject(
          Error(
            'The server has encountered an error. Please contact the administrator.'
          )
        );
      }
    };
    request.onerror = (): void => {
      reject(
        Error(
          'The server has encountered an error. Please contact the administrator.'
        )
      );
    };
    request.send(data);
  });
}

/**
 * Submits a batch list of transaction batches
 * @param {string}      url       The endpoint to submit the batch list to
 * @param {Uint8Array}  batchList The serialized batch list
 */
export async function submitBatchList(
  url: string,
  batchList: Uint8Array
): Promise<BatchInfo[]> {
  return http('POST', url, batchList, (request: XMLHttpRequest) => {
    request.setRequestHeader('Content-Type', 'application/octet-stream');
  })
    .catch(err => {
      throw new Error(err);
    })
    .then(body => {
      return JSON.parse(body).data as BatchInfo[];
    });
}
