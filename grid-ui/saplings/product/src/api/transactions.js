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

import {
  Secp256k1Signer,
  SabreTransactionBuilder,
  BatchBuilder,
  Secp256k1PrivateKey
} from 'transact-sdk-javascript';
import crypto from 'crypto';
import { submitBatchList, getSharedConfig } from 'splinter-saplingjs';
import protos from '../protobuf';

const { gridURL } = getSharedConfig().appConfig;

const ProductNamespace = '621dee';
const ProductVersion = '1.0';

const GridNamespace = 'cad11d';

const productTypeNamespaces = {
  GS1: `${ProductNamespace}0201`
};

const parseValue = (stringValue, conversionType) => {
  switch (conversionType) {
    case 'BOOLEAN':
      return Boolean(stringValue);
    case 'NUMBER':
      return Number(stringValue);
    default:
      return stringValue;
  }
};

function buildProductAddress(productId, productType) {
  let prefix;
  switch (productType) {
    case 'gs1':
      prefix = productTypeNamespaces.GS1;
      break;
    default:
      prefix = productTypeNamespaces.GS1;
  }
  return `${prefix}00000000000000000000000000000000000000000000${productId}00`;
}

function buildOrganizationAddess(orgName) {
  const hash = crypto
    .createHash('sha512')
    .update(orgName)
    .digest('hex')
    .substring(0, 62);
  return `${GridNamespace}01${hash}`;
}

function buildAgentAddress(publicKey) {
  const hash = crypto
    .createHash('sha512')
    .update(publicKey)
    .digest('hex')
    .substring(0, 62);
  return `${GridNamespace}00${hash}`;
}

function buildSchemaAddress(name) {
  const hash = crypto
    .createHash('sha512')
    .update(name)
    .digest('hex')
    .substring(0, 62);
  return `${ProductNamespace}01${hash}`;
}

export async function editProduct(data, keys, callbackFn) {
  const privateKey = Secp256k1PrivateKey.fromHex(keys.privateKey);
  const signer = new Secp256k1Signer(privateKey);

  const dataTypes = {
    STRING: 'stringValue',
    BOOLEAN: 'booleanValue',
    NUMBER: 'numberValue'
  };

  const propertiesList = data.properties.map(property => {
    const value = parseValue(property.value, property.type);
    return protos.PropertyValue.create({
      name: property.name,
      dataType: protos.PropertyDefinition.DataType[property.type],
      [dataTypes[property.type.toUpperCase()]]: value
    });
  });

  const product = protos.ProductUpdateAction.create({
    productType: protos.Product.ProductType.GS1,
    productId: data.productId,
    owner: data.orgName,
    properties: propertiesList
  });

  const payloadBytes = protos.ProductPayload.encode({
    action: protos.ProductPayload.Action.PRODUCT_UPDATE,
    timestamp: Date.now(),
    productUpdate: product
  }).finish();

  const txn = new SabreTransactionBuilder({
    name: 'grid_product',
    version: ProductVersion,
    prefix: ProductNamespace
  })
    .withBatcherPublicKey(signer.getPublicKey())
    .withFamilyName('grid_product')
    .withFamilyVersion(ProductVersion)
    .withInputs([
      buildProductAddress(data.productId, 'gs1'),
      buildOrganizationAddess(data.orgName),
      buildAgentAddress(keys.publicKey),
      buildSchemaAddress('product')
    ])
    .withOutputs([buildProductAddress(data.productId, 'gs1')])
    .withPayload(payloadBytes)
    .build(signer);

  const batch = new BatchBuilder().withTransactions([txn]).build(signer);

  data.services.forEach(async service => {
    await submitBatchList(`${gridURL}/batches?service_id=${service}`, batch);
  });
  callbackFn();
}

export async function addProduct(data, keys, callbackFn) {
  const privateKey = Secp256k1PrivateKey.fromHex(keys.privateKey);
  const signer = new Secp256k1Signer(privateKey);

  const dataTypes = {
    STRING: 'stringValue',
    BOOLEAN: 'booleanValue',
    NUMBER: 'numberValue'
  };

  const propertiesList = data.properties.map(property => {
    const value = parseValue(property.value, property.type);
    return protos.PropertyValue.create({
      name: property.name,
      dataType: protos.PropertyDefinition.DataType[property.type],
      [dataTypes[property.type.toUpperCase()]]: value
    });
  });

  const product = protos.ProductCreateAction.create({
    productType: protos.Product.ProductType.GS1,
    productId: data.productId,
    owner: data.orgName,
    properties: propertiesList
  });

  const payloadBytes = protos.ProductPayload.encode({
    action: protos.ProductPayload.Action.PRODUCT_CREATE,
    timestamp: Date.now(),
    productCreate: product
  }).finish();

  const txn = new SabreTransactionBuilder({
    name: 'grid_product',
    version: ProductVersion,
    prefix: ProductNamespace
  })
    .withBatcherPublicKey(signer.getPublicKey())
    .withFamilyName('grid_product')
    .withFamilyVersion(ProductVersion)
    .withInputs([
      buildProductAddress(data.productId, 'gs1'),
      buildOrganizationAddess(data.orgName),
      buildAgentAddress(keys.publicKey),
      buildSchemaAddress('product')
    ])
    .withOutputs([buildProductAddress(data.productId, 'gs1')])
    .withPayload(payloadBytes)
    .build(signer);

  const batch = new BatchBuilder().withTransactions([txn]).build(signer);

  data.services.forEach(async service => {
    await submitBatchList(`${gridURL}/batches?service_id=${service}`, batch);
  });
  callbackFn();
}

export async function deleteProduct(data, keys, callbackFn) {
  const privateKey = Secp256k1PrivateKey.fromHex(keys.privateKey);
  const signer = new Secp256k1Signer(privateKey);

  const product = protos.ProductDeleteAction.create({
    productType: protos.Product.ProductType.GS1,
    productId: data.productId
  });

  const payloadBytes = protos.ProductPayload.encode({
    action: protos.ProductPayload.Action.PRODUCT_DELETE,
    timestamp: Date.now(),
    productDelete: product
  }).finish();

  const txn = new SabreTransactionBuilder({
    name: 'grid_product',
    version: ProductVersion,
    prefix: ProductNamespace
  })
    .withBatcherPublicKey(signer.getPublicKey())
    .withFamilyName('grid_product')
    .withFamilyVersion(ProductVersion)
    .withInputs([
      buildProductAddress(data.productId, 'gs1'),
      buildOrganizationAddess(data.orgName),
      buildAgentAddress(keys.publicKey),
      buildSchemaAddress('product')
    ])
    .withOutputs([buildProductAddress(data.productId, 'gs1')])
    .withPayload(payloadBytes)
    .build(signer);

  const batch = new BatchBuilder().withTransactions([txn]).build(signer);

  data.services.forEach(async service => {
    await submitBatchList(`${gridURL}/batches?service_id=${service}`, batch);
  });
  callbackFn();
}
