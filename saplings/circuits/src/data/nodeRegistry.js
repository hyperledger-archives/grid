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

function Node(data) {
  this.identity = data.identity;
  this.endpoints = data.endpoints;
  this.displayName = data.display_name;
  this.metadata = data.metadata;
}

function NodeRegistryResponse(reponseData) {
  this.data = reponseData.data.map(node => {
    return new Node(node);
  });
  this.paging = {
    current: reponseData.paging.current,
    offset: reponseData.paging.offset,
    limit: reponseData.paging.limit,
    total: reponseData.paging.total,
    first: reponseData.paging.first,
    prev: reponseData.paging.prev,
    next: reponseData.paging.next,
    last: reponseData.paging.last
  };
}

export { NodeRegistryResponse, Node };
