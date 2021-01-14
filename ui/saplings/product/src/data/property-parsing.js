/**
 * Copyright 2018-2021 Cargill Incorporated
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

/**
 * Fetch the value of a property.
 * @param {Object} property - The property object.
 * @return {string} The value of the property.
 */
export const getPropertyValue = property => {
  switch (property.data_type.toLowerCase()) {
    case 'string':
      return property.string_value;
    case 'number':
      return property.number_value;
    case 'boolean':
      return property.boolean_value;
    default:
      throw Error(`unsupported property type: ${property.data_type}`);
  }
};

/**
 * Fetch the value of a given property by name the property name.
 * @param {string} name - The name of the property.
 * @param {Array} propertyList - A list of product properties.
 * @return {string} The value of the property.
 */
export const getProperty = (name, propertyList) => {
  const property = propertyList.find(p => p.name === name);
  if (property === undefined) {
    return null;
  }

  return getPropertyValue(property);
};

/**
 * Remove underscores and uppercase a given string.
 * @param {string} name - The snake case name of a property.
 * @return {string} The uppercase name of a property.
 */
export const formatPropertyName = name => {
  return name.replace(/_/g, ' ').toUpperCase();
};
