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
import React, { useRef } from 'react';
import { library } from '@fortawesome/fontawesome-svg-core';
import { faLeaf, faUserCircle } from '@fortawesome/free-solid-svg-icons';
import { SideNav } from 'canopy-design-system';
import './App.scss';

window.$CANOPY = {};

library.add(faLeaf);
library.add(faUserCircle);

function App() {
  const saplingDomNode = useRef(null);

  return (
    <div className="App">
      <SideNav userSaplingRoutes={[]} />
      <div className="view" ref={saplingDomNode} />
    </div>
  );
}

export default App;
