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
 * Wrapper function to set up XHR.
 * @param {string}      method    HTTP method for the request
 * @param {string}      url       endpoint to make the request to
 * @param {object}      data      Byte array representation of the request body
 * @param {function}    headerFn  Function to set the correct request headers
 */
export async function http(method, url, data, headerFn) {
  return new Promise((resolve, reject) => {
    const request = new XMLHttpRequest();
    request.open(method, url);
    request.withCredentials = true;
    if (headerFn) {
      headerFn(request);
    }
    request.onload = () => {
      if (request.status >= 200 && request.status < 300) {
        resolve(request.response);
      } else {
        reject(request.response);
      }
    };
    request.onerror = () => {
      reject(
        Error(
          'The server has encountered an error. Please contact the administrator.'
        )
      );
    };
    request.send(data);
  });
}
