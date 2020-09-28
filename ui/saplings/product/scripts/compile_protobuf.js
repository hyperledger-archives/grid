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

const protobuf = require('protobufjs');
const process = require('process');

const path = require('path');
const fs = require('fs');

const protoDir = process.argv[2];

const include = ['product_payload.proto', 'product_state.proto'];

let root = new protobuf.Root();

const files = fs
  .readdirSync(protoDir)
  .filter(f => include.includes(f))
  .map(f => path.resolve(protoDir, f));

try {
  root = root.loadSync(files);
} catch (e) {
  console.error(e);
  throw e;
}

const output = JSON.stringify(root, null, 2);

if (output !== '') {
  process.stdout.write(output, 'utf8');
}
