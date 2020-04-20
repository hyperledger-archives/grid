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
    proposal_type: 'Create',
    circuit_id: 'uI2jb-JtA9s',
    circuit_hash:
      '8ce518770b962429a953b10220905ac9adf86a855f0b085695f444edf991b8ca',
    circuit: {
      circuit_id: 'uI2jb-JtA9s',
      members: [
        {
          node_id: 'alpha-node-000',
          endpoint: 'tls://splinterd-alpha:8044'
        },
        {
          node_id: 'beta-node-000',
          endpoint: 'tls://splinterd-beta:8044'
        }
      ],
      roster: [
        {
          service_id: 'FGHI',
          service_type: 'scabbard',
          allowed_nodes: ['alpha-node-000'],
          arguments: {
            peer_services: ['JKLM'],
            admin_keys: [
              '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
            ]
          }
        },
        {
          service_id: 'JKLM',
          service_type: 'scabbard',
          allowed_nodes: ['beta-node-000'],
          arguments: {
            peer_services: ['FGHI'],
            admin_keys: [
              '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
            ]
          }
        }
      ],
      management_type: 'gameroom',
      application_metadata:
        '7b2273636162626172645f61646d696e5f6b657973223a5b223',
      comments: 'Alpha/Beta Circuit'
    },
    votes: [
      {
        public_key:
          '026c889058c2d22558ead2c61b321634b74e705c42f890e6b7bc2c80abb4713118',
        vote: 'Accept',
        voter_node_id: 'alpha-node-000'
      }
    ],
    requester:
      '026c889058c2d22558ead2c61b321634b74e705c42f890e6b7bc2c80abb4713118',
    requester_node_id: 'alpha-node-000'
  },
  {
    proposal_type: 'Create',
    circuit_id: '6MNri-wRJ7B',
    circuit_hash:
      '8ce518770b962429a953b10220905ac9adf86a855f0b085695f444edf991b8ca',
    circuit: {
      circuit_id: '6MNri-wRJ7B',
      members: [
        {
          node_id: 'alpha-node-000',
          endpoint: 'tls://splinterd-alpha:8044'
        },
        {
          node_id: 'beta-node-000',
          endpoint: 'tls://splinterd-beta:8044'
        },
        {
          node_id: 'gamma-node-000',
          endpoint: 'tls://splinterd-alpha:8044'
        },
        {
          node_id: 'delta-node-000',
          endpoint: 'tls://splinterd-delta:8044'
        },
        {
          node_id: 'epsilon-node-000',
          endpoint: 'tls://splinterd-epsilon:8044'
        },
        {
          node_id: 'zeta-node-000',
          endpoint: 'tls://splinterd-zeta:8044'
        },
        {
          node_id: 'eta-node-000',
          endpoint: 'tls://splinterd-eta:8044'
        },
        {
          node_id: 'theta-node-000',
          endpoint: 'tls://splinterd-theta:8044'
        }
      ],
      roster: [
        {
          service_id: 'd7tp',
          service_type: 'scabbard',
          allowed_nodes: ['alpha-node-000'],
          arguments: {
            peer_services: ['OBzl, LAaH, xi7n, qXNp, hsq9, Lii6, wmCp'],
            admin_keys: [
              '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
            ]
          }
        },
        {
          service_id: 'OBzl',
          service_type: 'scabbard',
          allowed_nodes: ['beta-node-000'],
          arguments: {
            peer_services: ['d7tp, LAaH, xi7n, qXNp, hsq9, Lii6, wmCp'],
            admin_keys: [
              '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
            ]
          }
        },
        {
          service_id: 'LAaH',
          service_type: 'scabbard',
          allowed_nodes: ['gamma-node-000'],
          arguments: {
            peer_services: ['d7tp, OBzl, xi7n, qXNp, hsq9, Lii6, wmCp'],
            admin_keys: [
              '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
            ]
          }
        },
        {
          service_id: 'xi7n',
          service_type: 'scabbard',
          allowed_nodes: ['delta-node-000'],
          arguments: {
            peer_services: ['d7tp, OBzl, LAaH, qXNp, hsq9, Lii6, wmCp'],
            admin_keys: [
              '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
            ]
          }
        },
        {
          service_id: 'qXNp',
          service_type: 'scabbard',
          allowed_nodes: ['epsilon-node-000'],
          arguments: {
            peer_services: ['d7tp, OBzl, LAaH, xi7n, hsq9, Lii6, wmCp'],
            admin_keys: [
              '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
            ]
          }
        },
        {
          service_id: 'hsq9',
          service_type: 'scabbard',
          allowed_nodes: ['zeta-node-000'],
          arguments: {
            peer_services: ['d7tp, OBzl, LAaH, xi7n, qXNp, Lii6, wmCp'],
            admin_keys: [
              '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
            ]
          }
        },
        {
          service_id: 'Lii6',
          service_type: 'scabbard',
          allowed_nodes: ['eta-node-000'],
          arguments: {
            peer_services: ['d7tp, OBzl, LAaH, xi7n, qXNp, hsq9, wmCp'],
            admin_keys: [
              '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
            ]
          }
        },
        {
          service_id: 'wmCp',
          service_type: 'scabbard',
          allowed_nodes: ['theta-node-000'],
          arguments: {
            peer_services: ['d7tp, OBzl, LAaH, xi7n, qXNp, hsq9, Lii6'],
            admin_keys: [
              '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
            ]
          }
        },
        {
          service_id: 'DMWU',
          service_type: 'private-xo',
          allowed_nodes: ['alpha-node-000'],
          arguments: {}
        }
      ],
      management_type: 'grid',
      application_metadata:
        '7b2273636162626172645f61646d696e5f6b657973223a5b223',
      comments:
        'Greek Alphabet Consortium: \n This is a long comment describing this proposal. This is a long comment describing this proposal. This is a long comment describing this proposal. This is a long comment describing this proposal. This is a long comment describing this proposal. This is a long comment describing this proposal. This is a long comment describing this proposal. This is a long comment describing this proposal. This is a long comment describing this proposal. This is a long comment describing this proposal.'
    },
    votes: [
      {
        public_key:
          '026c889058c2d22558ead2c61b321634b74e705c42f890e6b7bc2c80abb4713118',
        vote: 'Accept',
        voter_node_id: 'beta-node-000'
      },
      {
        public_key:
          '026c889058c2d22558ead2c61b321634b74e705c42f890e6b7bc2c80abb4713118',
        vote: 'Accept',
        voter_node_id: 'gamma-node-000'
      },
      {
        public_key:
          '026c889058c2d22558ead2c61b321634b74e705c42f890e6b7bc2c80abb4713118',
        vote: 'Accept',
        voter_node_id: 'delta-node-000'
      }
    ],
    requester:
      '026c889058c2d22558ead2c61b321634b74e705c42f890e6b7bc2c80abb4713118',
    requester_node_id: 'beta-node-000'
  },
  {
    proposal_type: 'Create',
    circuit_id: 'S0cJ6-Z3Gb8',
    circuit_hash:
      '8ce518770b962429a953b10220905ac9adf86a855f0b085695f444edf991b8ca',
    circuit: {
      circuit_id: 'S0cJ6-Z3Gb8',
      members: [
        {
          node_id: 'alpha-node-000',
          endpoint: 'tls://splinterd-alpha:8044'
        },
        {
          node_id: 'beta-node-000',
          endpoint: 'tls://splinterd-beta:8044'
        }
      ],
      roster: [
        {
          service_id: 'LklR',
          service_type: 'scabbard',
          allowed_nodes: ['alpha-node-000'],
          arguments: {
            peer_services: ['GLnW'],
            admin_keys: [
              '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
            ]
          }
        },
        {
          service_id: 'GLnW',
          service_type: 'scabbard',
          allowed_nodes: ['beta-node-000'],
          arguments: {
            peer_services: ['LklR'],
            admin_keys: [
              '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
            ]
          }
        }
      ],
      management_type: 'gameroom',
      application_metadata:
        '7b2273636162626172645f61646d696e5f6b657973223a5b223',
      comments: 'Test Circuit'
    },
    votes: [
      {
        public_key:
          '026c889058c2d22558ead2c61b321634b74e705c42f890e6b7bc2c80abb4713118',
        vote: 'Accept',
        voter_node_id: 'beta-node-000'
      }
    ],
    requester:
      '026c889058c2d22558ead2c61b321634b74e705c42f890e6b7bc2c80abb4713118',
    requester_node_id: 'beta-node-000'
  },
  {
    proposal_type: 'Create',
    circuit_id: '2LeHR-rKeX7',
    circuit_hash:
      '8ce518770b962429a953b10220905ac9adf86a855f0b085695f444edf991b8ca',
    circuit: {
      circuit_id: '2LeHR-rKeX7',
      members: [
        {
          node_id: 'alpha-node-000',
          endpoint: 'tls://splinterd-alpha:8044'
        },
        {
          node_id: 'beta-node-000',
          endpoint: 'tls://splinterd-beta:8044'
        }
      ],
      roster: [
        {
          service_id: '59YP',
          service_type: 'scabbard',
          allowed_nodes: ['alpha-node-000'],
          arguments: {
            peer_services: ['m8l1'],
            admin_keys: [
              '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
            ]
          }
        },
        {
          service_id: 'm8l1',
          service_type: 'scabbard',
          allowed_nodes: ['beta-node-000'],
          arguments: {
            peer_services: ['59YP'],
            admin_keys: [
              '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
            ]
          }
        }
      ],
      management_type: 'grid',
      application_metadata:
        '7b2273636162626172645f61646d696e5f6b657973223a5b223',
      comments: 'Grid Test Circuit'
    },
    votes: [
      {
        public_key:
          '026c889058c2d22558ead2c61b321634b74e705c42f890e6b7bc2c80abb4713118',
        vote: 'Accept',
        voter_node_id: 'alpha-node-000'
      }
    ],
    requester:
      '026c889058c2d22558ead2c61b321634b74e705c42f890e6b7bc2c80abb4713118',
    requester_node_id: 'alpha-node-000'
  },
  {
    proposal_type: 'Create',
    circuit_id: 'IK6Nt-8vG49',
    circuit_hash:
      '8ce518770b962429a953b10220905ac9adf86a855f0b085695f444edf991b8ca',
    circuit: {
      circuit_id: 'IK6Nt-8vG49',
      members: [
        {
          node_id: 'alpha-node-000',
          endpoint: 'tls://splinterd-alpha:8044'
        },
        {
          node_id: 'beta-node-000',
          endpoint: 'tls://splinterd-beta:8044'
        }
      ],
      roster: [
        {
          service_id: 'yBuC',
          service_type: 'scabbard',
          allowed_nodes: ['alpha-node-000'],
          arguments: {
            peer_services: ['d6od'],
            admin_keys: [
              '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
            ]
          }
        },
        {
          service_id: 'd6od',
          service_type: 'scabbard',
          allowed_nodes: ['beta-node-000'],
          arguments: {
            peer_services: ['yBuC'],
            admin_keys: [
              '029150e180d57a8d5babde0ea6ae86193fcef7d40ae145b571b0654bf23071b169'
            ]
          }
        }
      ],
      management_type: 'defaut',
      application_metadata:
        '7b2273636162626172645f61646d696e5f6b657973223a5b223',
      comments: 'Circuit Test 001'
    },
    votes: [
      {
        public_key:
          '026c889058c2d22558ead2c61b321634b74e705c42f890e6b7bc2c80abb4713118',
        vote: 'Accept',
        voter_node_id: 'alpha-node-000'
      }
    ],
    requester:
      '026c889058c2d22558ead2c61b321634b74e705c42f890e6b7bc2c80abb4713118',
    requester_node_id: 'alpha-node-000'
  }
];
