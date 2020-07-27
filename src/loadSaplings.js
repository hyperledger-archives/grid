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

import promiseLoader from './promiseLoader';
import { styleLoader, unloadStylesByClassName } from './styleLoader';

export async function mountCurrentSapling(userSaplingsResponse) {
  // Saplings will be guaranteed to to have a collision-free namespace that
  // corresponds with the place entry in the URL's path

  // for instance /example and all of its child paths
  // (/example/first, /example/first/a, example/second)
  // would all let load the Sapling with `example` in its namespace attribute

  // saplings themselves can take over routing at that point to manage their
  // own routing (ie preventing full page refreshes, push on history)

  const topLevelPathRgx = /\/([^/]+)/i;
  const pathMatches = topLevelPathRgx.exec(window.location.pathname);
  const saplingNamespaceToLoad =
    pathMatches && pathMatches[1] ? pathMatches[1] : null;
  const currentSaplingManifest = userSaplingsResponse.find(
    ({ namespace }) => saplingNamespaceToLoad === namespace
  );

  if (currentSaplingManifest) {
    unloadStylesByClassName('user-sapling-stylesheet');
    await Promise.all(
      currentSaplingManifest.styleFiles.map(styleFile =>
        styleLoader(`http://${styleFile}`, 'user-sapling-stylesheet')
      )
    );
    await Promise.all(
      currentSaplingManifest.runtimeFiles.map(saplingFile =>
        promiseLoader(`http://${saplingFile}`)
      )
    );

    return true;
  }

  return false;
}

export async function mountConfigSaplingStyles(saplingResponse) {
  const saplingStyleFiles = saplingResponse
    .map(sapling => sapling.styleFiles)
    .flatMap(style => style)
    .filter(style => style !== undefined);

  if (saplingStyleFiles.length === 0) {
    return false;
  }

  await Promise.all(
    saplingStyleFiles.map(styleFile =>
      styleLoader(`http://${styleFile}`, 'config-sapling-stylesheet')
    )
  );
  return true;
}

export async function mountConfigSaplings(configSaplingResponse) {
  // Config Saplings need to be loaded with every page load.
  // An example of a Config Saplings would be a module to handle
  // user login/registration.
  const configSaplingRuntimeFiles = configSaplingResponse
    .map(s => s.runtimeFiles)
    .flatMap(r => r);

  if (configSaplingRuntimeFiles.length === 0) {
    return false;
  }

  await Promise.all(
    configSaplingRuntimeFiles.map(saplingFile =>
      promiseLoader(`http://${saplingFile}`)
    )
  );
  return true;
}
