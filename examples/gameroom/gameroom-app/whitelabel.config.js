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

const whitelabelConfig = {
  'bubba': {
    'brand': 'bubba',
    'scssVariables': './src/scss/modules/themes/bubba',
    'assets': './src/assets/bubba',
  },
  'acme': {
    'brand': 'acme',
    'scssVariables': './src/scss/modules/themes/acme',
    'assets': './src/assets/acme',
  },
  'generic': {
    'brand': 'generic',
    'scssVariables': './src/scss/modules/themes/generic',
    'assets': './src/assets/generic',
  },
}

module.exports = whitelabelConfig;
