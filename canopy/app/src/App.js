/**
 * Copyright 2019 Cargill Incorporated
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

import React, { useRef, useState, useEffect } from 'react';
import { useUserState, UserProvider } from 'UserStore';
import { loadAllSaplings } from './loadSaplings';
import 'App.scss';

window.$CANOPY = {};

function App() {
  const saplingDomNode = useRef(null);
  const [userSaplingManifests, setUserSaplingManifests] = useState([]);
  const [user, setUser] = useUserState();

  const appSapling = useRef(null);
  const configSaplings = useRef({});

  // Define implementaion of CanopyJS methods
  window.$CANOPY.registerApp = bootstrapFunction => {
    appSapling.current = bootstrapFunction;
  };

  window.$CANOPY.registerConfigSapling = (namespace, bootStrapFunction) => {
    configSaplings.current[namespace] = bootStrapFunction;
  };

  window.$CANOPY.setUser = setUser;
  window.$CANOPY.getUser = () => user;

  // This useEffect with zero dependencies will run only when the component first loads.
  useEffect(() => {
    (async () => {
      const { userSaplingsResponse } = await loadAllSaplings();
      setUserSaplingManifests(userSaplingsResponse);

      // Load the config saplings
      const configs = Object.values(configSaplings.current);
      if (configs.length === 0) {
        throw new Error('No Config Saplings registered');
      }
      configs.forEach(bootstrapConfigSapling => {
        bootstrapConfigSapling();
      });

      // Invoke the current sapling if one has been registered
      if (
        appSapling.current &&
        typeof appSapling.current === typeof Function.prototype
      ) {
        appSapling.current(saplingDomNode.current);
      }
    })();
  }, []);

  return (
    <>
      <nav>
        {userSaplingManifests.map(({ displayName, namespace }) => {
          return (
            <a href={`/${namespace}`} key={namespace}>
              {displayName}
            </a>
          );
        })}
      </nav>
      <div ref={saplingDomNode} />
    </>
  );
}

function AppWithProvider() {
  return (
    <UserProvider>
      <App />
    </UserProvider>
  );
}
export default AppWithProvider;
