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

export default [
  {
    id: 'WBh1C-MIcIK',
    members: ['alpha-node-000', 'beta-node-000'],
    roster: [
      {
        service_id: 'JWrS',
        service_type: 'scabbard',
        allowed_nodes: ['alpha-node-000'],
        arguments: {
          peer_services: ['rOF6'],
          admin_keys: [
            '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
          ]
        }
      },
      {
        service_id: 'rOF6',
        service_type: 'scabbard',
        allowed_nodes: ['beta-node-000'],
        arguments: {
          peer_services: ['JWrS'],
          admin_keys: [
            '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
          ]
        }
      }
    ],
    management_type: 'gameroom',
    application_metadata: ''
  },
  {
    id: 'j64jw-QUi5K',
    members: [
      'alpha-node-000',
      'beta-node-000',
      'gamma-node-000',
      'delta-node-000',
      'epsilon-node-000',
      'zeta-node-000',
      'eta-node-000',
      'theta-node-000'
    ],
    roster: [
      {
        service_id: 'b5ED',
        service_type: 'scabbard',
        allowed_nodes: ['alpha-node-000'],
        arguments: {
          peer_services: ['DI91, ruP2, oUwo, RNM4, 9dw0, yHE4, zmgU'],
          admin_keys: [
            '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
          ]
        }
      },
      {
        service_id: 'DI91',
        service_type: 'scabbard',
        allowed_nodes: ['beta-node-000'],
        arguments: {
          peer_services: ['b5ED, ruP2, oUwo, RNM4, 9dw0, yHE4, zmgU'],
          admin_keys: [
            '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
          ]
        }
      },
      {
        service_id: 'ruP2',
        service_type: 'scabbard',
        allowed_nodes: ['gamma-node-000'],
        arguments: {
          peer_services: ['b5ED, DI91, oUwo, RNM4, 9dw0, yHE4, zmgU'],
          admin_keys: [
            '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
          ]
        }
      },
      {
        service_id: 'oUwo',
        service_type: 'scabbard',
        allowed_nodes: ['delta-node-000'],
        arguments: {
          peer_services: ['b5ED, DI91, ruP2, RNM4, 9dw0, yHE4, zmgU'],
          admin_keys: [
            '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
          ]
        }
      },
      {
        service_id: 'RNM4',
        service_type: 'scabbard',
        allowed_nodes: ['epsilon-node-000'],
        arguments: {
          peer_services: ['b5ED, DI91, ruP2, oUwo, 9dw0, yHE4, zmgU'],
          admin_keys: [
            '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
          ]
        }
      },
      {
        service_id: '9dw0',
        service_type: 'scabbard',
        allowed_nodes: ['zeta-node-000'],
        arguments: {
          peer_services: ['b5ED, DI91, ruP2, oUwo, RNM4, yHE4, zmgU'],
          admin_keys: [
            '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
          ]
        }
      },
      {
        service_id: 'yHE4',
        service_type: 'scabbard',
        allowed_nodes: ['eta-node-000'],
        arguments: {
          peer_services: ['b5ED, DI91, ruP2, oUwo, RNM4, 9dw0, zmgU'],
          admin_keys: [
            '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
          ]
        }
      },
      {
        service_id: 'zmgU',
        service_type: 'scabbard',
        allowed_nodes: ['theta-node-000'],
        arguments: {
          peer_services: ['b5ED, DI91, ruP2, oUwo, RNM4, 9dw0, yHE4'],
          admin_keys: [
            '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
          ]
        }
      },
      {
        service_id: 'S0Pi',
        service_type: 'private-xo',
        allowed_nodes: ['alpha-node-000'],
        arguments: {}
      }
    ],
    management_type: 'grid',
    application_metadata: ''
  },
  {
    id: 'vwXDB-aHBpR',
    members: ['alpha-node-000', 'beta-node-000'],
    roster: [
      {
        service_id: '4k0f',
        service_type: 'scabbard',
        allowed_nodes: ['alpha-node-000'],
        arguments: {
          peer_services: ['d4He'],
          admin_keys: [
            '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
          ]
        }
      },
      {
        service_id: 'd4He',
        service_type: 'scabbard',
        allowed_nodes: ['beta-node-000'],
        arguments: {
          peer_services: ['4k0f'],
          admin_keys: [
            '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
          ]
        }
      }
    ],
    management_type: 'default',
    application_metadata: ''
  },
  {
    id: 'pqGUt-8rSS9',
    members: ['alpha-node-000', 'beta-node-000'],
    roster: [
      {
        service_id: 'jYQt',
        service_type: 'scabbard',
        allowed_nodes: ['alpha-node-000'],
        arguments: {
          peer_services: ['qQ5k'],
          admin_keys: [
            '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
          ]
        }
      },
      {
        service_id: 'qQ5k',
        service_type: 'scabbard',
        allowed_nodes: ['beta-node-000'],
        arguments: {
          peer_services: ['jYQt'],
          admin_keys: [
            '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
          ]
        }
      }
    ],
    management_type: 'gameroom',
    application_metadata: ''
  }
];
