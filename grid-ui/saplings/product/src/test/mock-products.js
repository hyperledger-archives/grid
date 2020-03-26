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
    service_id: 'cargill-target0',
    product_id: '1',
    owner: 'Target',
    properties: [
      {
        name: 'product_name',
        data_type: 'STRING',
        string_value: 'Canola Oil'
      },
      {
        name: 'gtin',
        data_type: 'NUMBER',
        number_value: '00012345600012'
      },
      {
        name: 'image_url',
        data_type: 'STRING',
        string_value:
          'https://target.scene7.com/is/image/Target/GUEST_e8106dc8-b312-49d3-b9f5-ae4f01553452?fmt=webp&wid=1400&qlt=80'
      },
      {
        name: 'brand_name',
        data_type: 'STRING',
        string_value: 'Market Pantry'
      },
      {
        name: 'product_description',
        data_type: 'STRING',
        string_value: 'Market Pantry Canola Oil 48oz'
      },
      {
        name: 'gpc',
        data_type: 'NUMBER',
        number_value: '30002914'
      },
      {
        name: 'net_content',
        data_type: 'STRING',
        string_value: '48oz'
      },
      {
        name: 'target_market',
        data_type: 'NUMBER',
        number_value: '840'
      }
    ]
  },
  {
    service_id: 'cargill-target1',
    product_id: '2',
    owner: 'Cargill',
    properties: [
      {
        name: 'product_name',
        data_type: 'STRING',
        string_value: 'Truvia 80 ct.'
      },
      {
        name: 'gtin',
        data_type: 'NUMBER',
        number_value: '00012345600099'
      },
      {
        name: 'image_url',
        data_type: 'STRING',
        string_value:
          'https://target.scene7.com/is/image/Target/GUEST_b7a6e983-b391-40a5-ad89-2f906bce5743?fmt=webp&wid=1400&qlt=80'
      },
      {
        name: 'brand_name',
        data_type: 'STRING',
        string_value: 'Truvia'
      },
      {
        name: 'product_description',
        data_type: 'STRING',
        string_value: 'Truvia Sugar 80CT'
      },
      {
        name: 'gpc',
        data_type: 'NUMBER',
        number_value: '30016951'
      },
      {
        name: 'net_content',
        data_type: 'STRING',
        string_value: '80CT'
      },
      {
        name: 'target_market',
        data_type: 'NUMBER',
        number_value: '840'
      }
    ]
  },
  {
    service_id: 'cargill-target5',
    product_id: '3',
    owner: 'Cargill',
    properties: [
      {
        name: 'product_name',
        data_type: 'STRING',
        string_value: 'Anonymous Product'
      },
      {
        name: 'gtin',
        data_type: 'NUMBER',
        number_value: '00000000000001'
      }
    ]
  }
];
