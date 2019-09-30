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

'use strict'

const protobuf = require('protobufjs');
const process = require('process');

const path = require('path');
const fs = require('fs');

const proto_dir = process.argv[2];

const include = [
  'admin.proto',
]

let root = new protobuf.Root();

let files = fs.readdirSync(proto_dir)
  .filter(f => include.includes(f))
  .map(f => path.resolve(proto_dir, f));

let sabre_proto = path.resolve('./sabre_proto/sabre_payload.proto');

files.push(sabre_proto);

try {
  root = root.loadSync(files);
} catch (e) {
  console.error(e);
  throw e;
}

let output = JSON.stringify(root, null, 2);

if (output !== '') {
  process.stdout.write(output, 'utf8');
}
