const { createMacro } = require('babel-plugin-macros');
const fs = require('fs');
const path = require('path');
const { parseExpression } = require('@babel/parser');
const yaml = require('js-yaml');

function loadYaml(yamlPath) {
  return yaml.safeLoad(
    fs.readFileSync(path.resolve(path.join(__dirname, yamlPath)), 'utf8')
  );
}

function replaceReferenceWithPromiseResolveExpression(resplacement) {
  return function doReplace(reference) {
    // In order to make the return signature align with the logical future state,
    // loading from an API call,
    // the macro returns a function that returns a promise.
    reference.replaceWith(
      parseExpression(`() => Promise.resolve(${JSON.stringify(resplacement)})`)
    );
  };
}

function saplingMacro({ references }) {
  const {
    getUserSaplings = [],
    getConfigSaplings = [],
    getSharedConfig = []
  } = references;

  const userSaplings = loadYaml('../saplings/userSaplings.yml');
  const configSaplings = loadYaml('../saplings/configSaplings.yml');
  const sharedConfig = loadYaml('../saplings/canopyConfig.yml');

  getUserSaplings.forEach(
    replaceReferenceWithPromiseResolveExpression(userSaplings)
  );

  getConfigSaplings.forEach(
    replaceReferenceWithPromiseResolveExpression(configSaplings)
  );

  getSharedConfig.forEach(
    replaceReferenceWithPromiseResolveExpression(sharedConfig)
  );
}

module.exports = createMacro(saplingMacro);
