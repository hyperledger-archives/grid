// Copyright 2018-2020 Cargill Incorporated
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
import protobuf from 'protobufjs';

// ignoring because this file is generated before deploying
// eslint-disable-next-line import/no-unresolved
const protoJSON = require('./compiled_protos.json');

const root = protobuf.Root.fromJSON(protoJSON);

export default Object.keys(root)
  .filter(key => /^[A-Z]/.test(key))
  .reduce((acc, key) => {
    acc[key] = root.get(key);
    return acc;
  }, {});
