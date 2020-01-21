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

window.$CANOPY.registerConfigSapling('profile', () => {
  console.log('Registering Profile Sapling');

  if (window.location.pathname === '/profile') {
    window.$CANOPY.registerApp(function(domNode) {
      console.log('Rendering Profile JS App');
      domNode.innerHTML = `<h1>Profile<h1>`;
    });
  }
});
