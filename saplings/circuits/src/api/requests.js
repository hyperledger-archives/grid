/**
 * Copyright 2018-2021 Cargill Incorporated
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

/**
 * Returns the parsed JSON, if possible, `null` otherwise.
 */
function safeJSON(stringValue) {
  try {
    return JSON.parse(stringValue);
  } catch (e) {
    return null;
  }
}

function response(request, ok, overrideResponseText) {
  return {
    ok,
    status: request.status,
    statusText: request.statusText,
    headers: request.getAllResponseHeaders(),
    data: overrideResponseText || request.responseText,
    json: overrideResponseText
      ? safeJSON(overrideResponseText)
      : safeJSON(request.responseText)
  };
}

function httpRequest(method, url, data, headerFn) {
  return new Promise((resolve, reject) => {
    const request = new XMLHttpRequest();
    request.open(method, url, true);
    request.withCredentials = true;
    if (headerFn) {
      headerFn(request);
    }
    request.timeout = 5000;

    request.onload = () => {
      if (request.status >= 200 && request.status < 300) {
        resolve(response(request, true));
      } else {
        reject(response(request, false));
      }
    };

    request.onerror = () => {
      reject(response(request, false));
    };

    request.ontimeout = () => {
      reject(response(request, false, 'Request took longer than expected.'));
    };

    request.send(data);
  });
}

export function get(url, headerFn = null) {
  return httpRequest('GET', url, null, headerFn);
}

export function post(url, data, headerFn = null) {
  return httpRequest('POST', url, data, headerFn);
}
